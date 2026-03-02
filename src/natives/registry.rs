use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use wgpu::util::DeviceExt;
use winit::{event_loop::EventLoop, window::Window as WinitWindow};

// Wrapper for Window to bypass non-Send restriction. Safe because our executor is single-threaded.
pub struct SendWindow(pub RegistryWindowState);
unsafe impl Send for SendWindow {}
unsafe impl Sync for SendWindow {}

pub struct RegistryWindowState {
    pub window: Arc<WinitWindow>,
    // Store EventLoop temporarily if needed, though winit requires run to pump properly.
    // For synchronous MVP we might just let it be.
    pub surface: wgpu::Surface<'static>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub width: u32,
    pub height: u32,
    // Hack for synchronous winit updates without a main loop pump:
    pub clear_color: wgpu::Color,
    // Frame resources updated every frame
    pub current_texture: Option<wgpu::SurfaceTexture>,
    pub current_view: Option<wgpu::TextureView>,
    pub encoder: Option<wgpu::CommandEncoder>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RegistryVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

// The types of resources we can manage
pub enum NativeHandle {
    Counter(StatefulCounter),
    Window(SendWindow),
    File(File),
    Timestamp(std::time::Instant),
    GpuContext(GpuContext),
    VoxelWorld(SendVoxelWorld),
    Texture(TextureAsset),
}

pub struct RegistryEntry {
    pub handle: NativeHandle,
    pub ref_count: usize,
}

// Our dummy stateful Rust object
pub struct StatefulCounter {
    pub count: i64,
}

// GPU Context managed by the Registry
pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

// SAFETY: wgpu GPU types are Send+Sync; our registry is single-threaded.
unsafe impl Send for GpuContext {}
unsafe impl Sync for GpuContext {}

pub struct TextureAsset {
    pub bind_group: Arc<wgpu::BindGroup>,
    pub width: u32,
    pub height: u32,
}
unsafe impl Send for TextureAsset {}
unsafe impl Sync for TextureAsset {}

// VoxelWorld — isometric software-rendered voxel scene
pub struct VoxelWorldState {
    pub width: usize,
    pub height: usize,
    pub voxels: Vec<[i32; 3]>,
}

pub struct SendVoxelWorld(pub VoxelWorldState);
unsafe impl Send for SendVoxelWorld {}
unsafe impl Sync for SendVoxelWorld {}

// ── Isometric software renderer ───────────────────────────────────────

/// Scanline polygon fill for convex polygons (used for isometric cube faces).
fn fill_poly(buffer: &mut Vec<u32>, width: usize, height: usize, pts: &[(i32, i32)], color: u32) {
    let min_y = pts.iter().map(|&(_, y)| y).min().unwrap_or(0).max(0) as usize;
    let raw_max = pts.iter().map(|&(_, y)| y).max().unwrap_or(0) as usize;
    let max_y = raw_max.min(height.saturating_sub(1));
    if min_y >= height {
        return;
    }
    let n = pts.len();
    for row in min_y..=max_y {
        let y = row as i32;
        let mut xs: Vec<i32> = Vec::new();
        for i in 0..n {
            let (x0, y0) = pts[i];
            let (x1, y1) = pts[(i + 1) % n];
            let (lo, hi, xa, xb) = if y0 < y1 {
                (y0, y1, x0, x1)
            } else {
                (y1, y0, x1, x0)
            };
            if lo <= y && y < hi && lo != hi {
                let t = (y - lo) as f32 / (hi - lo) as f32;
                xs.push((xa as f32 + t * (xb - xa) as f32) as i32);
            }
        }
        xs.sort_unstable();
        let mut i = 0;
        while i + 1 < xs.len() {
            let x0 = xs[i].max(0) as usize;
            let x1 = (xs[i + 1]).min(width as i32 - 1) as usize;
            if x0 <= x1 {
                for col in x0..=x1 {
                    buffer[row * width + col] = color;
                }
            }
            i += 2;
        }
    }
}

/// Isometric projection render — painters-sorted, 3-face-per-voxel.
fn iso_render(buffer: &mut Vec<u32>, width: usize, height: usize, voxels: &[[i32; 3]]) {
    buffer.iter_mut().for_each(|p| *p = 0x0d1b2a); // dark navy background
    let cx = (width as i32) / 2;
    let cy = (height as i32) * 5 / 8;
    let tw = 14i32; // half-width of one voxel tile
    let ts = 7i32; // half-height of top rhombus

    // Back-to-front sort: larger (vx + vz - vy*2) draws first
    let mut sorted: Vec<[i32; 3]> = voxels.to_vec();
    sorted.sort_by_key(|v| v[0] - v[1] * 2 + v[2]);

    for [vx, vy, vz] in sorted.iter() {
        let sx = cx + (vx - vz) * tw;
        let sy = cy + (vx + vz) * ts - vy * ts * 2;

        // Top face (rhombus)
        fill_poly(
            buffer,
            width,
            height,
            &[(sx, sy - ts), (sx + tw, sy), (sx, sy + ts), (sx - tw, sy)],
            0x5b9bd5,
        );
        // Left face (darker)
        fill_poly(
            buffer,
            width,
            height,
            &[
                (sx - tw, sy),
                (sx, sy + ts),
                (sx, sy + ts * 3),
                (sx - tw, sy + ts * 2),
            ],
            0x2e6ea8,
        );
        // Right face (darkest)
        fill_poly(
            buffer,
            width,
            height,
            &[
                (sx, sy + ts),
                (sx + tw, sy),
                (sx + tw, sy + ts * 2),
                (sx, sy + ts * 3),
            ],
            0x1a4a7c,
        );
    }
}

// Global thread-safe registry
// Instead of lazy_static we'll use a const Mutex with an Option since lazy_static might not be available
static COUNTER_REGISTRY: Mutex<Option<HashMap<usize, RegistryEntry>>> = Mutex::new(None);
static COUNTER_NEXT_ID: Mutex<usize> = Mutex::new(1);

fn with_registry<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<usize, RegistryEntry>) -> R,
{
    let mut option_guard = COUNTER_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    if option_guard.is_none() {
        *option_guard = Some(HashMap::new());
    }
    f(option_guard.as_mut().unwrap())
}

// ── Lifecycle FFI Implementations ─────────────────────────────────

pub fn registry_retain(handle_id: i64) {
    if handle_id < 0 {
        return;
    }
    let id = handle_id as usize;
    with_registry(|registry| {
        if let Some(entry) = registry.get_mut(&id) {
            entry.ref_count += 1;
        }
    });
}

pub fn registry_release(handle_id: i64) {
    if handle_id < 0 {
        return;
    }
    let id = handle_id as usize;
    let mut remove = false;
    with_registry(|registry| {
        if let Some(entry) = registry.get_mut(&id) {
            if entry.ref_count > 0 {
                entry.ref_count -= 1;
            }
            if entry.ref_count == 0 {
                remove = true;
            }
        }
        if remove {
            registry.remove(&id);
        }
    });
}

// FFI Implementations
pub fn registry_create_counter() -> i64 {
    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap_or_else(|e| e.into_inner());
    let id = *id_guard;
    *id_guard += 1;

    let counter = StatefulCounter { count: 0 };
    with_registry(|registry| {
        registry.insert(
            id,
            RegistryEntry {
                handle: NativeHandle::Counter(counter),
                ref_count: 1,
            },
        );
    });

    id as i64
}

pub fn registry_increment(handle_id: i64) {
    if handle_id < 0 {
        return;
    }
    let id = handle_id as usize;
    with_registry(|registry| {
        if let Some(entry) = registry.get_mut(&id) {
            if let NativeHandle::Counter(counter) = &mut entry.handle {
                counter.count += 1;
            } else {
                eprintln!("[KnotenCore Registry] Error: Target handle is not a Counter.");
            }
        } else {
            eprintln!(
                "[KnotenCore Registry] Error: Counter handle {} not found.",
                handle_id
            );
        }
    });
}

pub fn registry_get_value(handle_id: i64) -> i64 {
    if handle_id < 0 {
        return 0;
    }
    let id = handle_id as usize;
    with_registry(|registry| {
        if let Some(entry) = registry.get(&id) {
            if let NativeHandle::Counter(counter) = &entry.handle {
                counter.count
            } else {
                -1
            }
        } else {
            eprintln!(
                "[KnotenCore Registry] Error: Counter handle {} not found.",
                handle_id
            );
            -1
        }
    })
}

pub fn registry_free(handle_id: i64) {
    if handle_id < 0 {
        return;
    }
    // Finding C-2: Do not unconditionally remove the handle, respect the refcount mechanism by releasing it
    registry_release(handle_id);
}

pub fn registry_dump() -> i64 {
    let mut count = 0;
    with_registry(|registry| {
        println!("[KnotenCore Registry] --- MEMORY DUMP ---");
        for (id, entry) in registry.iter() {
            let handle_type = match &entry.handle {
                NativeHandle::Counter(_) => "Counter",
                NativeHandle::Window(_) => "Window",
                NativeHandle::File(_) => "File",
                NativeHandle::Timestamp(_) => "Timestamp",
                NativeHandle::GpuContext(_) => "GpuContext",
                NativeHandle::VoxelWorld(SendVoxelWorld(s)) => {
                    println!("      voxels={}, {}x{}", s.voxels.len(), s.width, s.height);
                    "VoxelWorld"
                }
                NativeHandle::Texture(tex) => {
                    println!("      {}x{}", tex.width, tex.height);
                    "Texture"
                }
            };
            println!(
                "   -> Handle {} [Type: {}, RefCount: {}]",
                id, handle_type, entry.ref_count
            );
            count += 1;
        }
        println!("[KnotenCore Registry] Total Active: {}", count);
    });
    count
}

// ── Timestamp Orchestration ────────────────────────────────────────

pub fn registry_now() -> i64 {
    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap_or_else(|e| e.into_inner());
    let id = *id_guard;
    *id_guard += 1;

    with_registry(|registry| {
        registry.insert(
            id,
            RegistryEntry {
                handle: NativeHandle::Timestamp(std::time::Instant::now()),
                ref_count: 1,
            },
        );
    });

    id as i64
}

pub fn registry_elapsed_ms(handle_id: i64) -> i64 {
    if handle_id < 0 {
        return 0;
    }
    let id = handle_id as usize;
    with_registry(|registry| {
        if let Some(entry) = registry.get(&id) {
            if let NativeHandle::Timestamp(t) = &entry.handle {
                t.elapsed().as_millis() as i64
            } else {
                -1
            }
        } else {
            -1
        }
    })
}

// ── Window Orchestration ─────────────────────────────────────────

pub fn registry_create_window(width: i64, height: i64, title: String) -> i64 {
    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap_or_else(|e| e.into_inner());
    let id = *id_guard;
    *id_guard += 1;

    let w = width as u32;
    let h = height as u32;

    #[cfg(target_os = "windows")]
    use winit::platform::windows::EventLoopBuilderExtWindows;

    #[cfg(target_os = "windows")]
    let mut event_loop = winit::event_loop::EventLoop::builder()
        .with_any_thread(true)
        .build()
        .unwrap();
    #[cfg(not(target_os = "windows"))]
    let mut event_loop = EventLoop::new().unwrap();

    use winit::application::ApplicationHandler;
    #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
    use winit::platform::pump_events::EventLoopExtPumpEvents;

    struct WindowPump {
        window: Option<Arc<WinitWindow>>,
        width: u32,
        height: u32,
        title: String,
    }

    impl ApplicationHandler for WindowPump {
        fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            if self.window.is_none() {
                let attrs = winit::window::Window::default_attributes()
                    .with_inner_size(winit::dpi::PhysicalSize::new(self.width, self.height))
                    .with_title(&self.title);
                let win = event_loop.create_window(attrs).unwrap();
                self.window = Some(Arc::new(win));
            }
        }

        fn window_event(
            &mut self,
            _: &winit::event_loop::ActiveEventLoop,
            _: winit::window::WindowId,
            _: winit::event::WindowEvent,
        ) {
        }
    }

    let mut pump = WindowPump {
        window: None,
        width: w,
        height: h,
        title,
    };

    let _ = event_loop.pump_app_events(Some(std::time::Duration::from_millis(50)), &mut pump);

    let window = pump
        .window
        .expect("Failed to create Winit 0.30 Window via resumed()");

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = instance.create_surface(window.clone()).unwrap();

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .expect("No suitable GPU adapter found");

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("KnotenCore GPU Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        },
        None,
    ))
    .expect("Failed to create WGPU device");

    let device = Arc::new(device);
    let queue = Arc::new(queue);

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: w,
        height: h,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    // Default WGSL Shader for rendering Quads
    let shader_source = "
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = tex_coords;
    out.clip_position = vec4<f32>(position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
    ";

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("KnotenCore Base Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_source)),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<RegistryVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None, // No backface culling for 2D UI
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    let state = RegistryWindowState {
        window,
        surface,
        device,
        queue,
        pipeline: render_pipeline,
        bind_group_layout,
        width: w,
        height: h,
        clear_color: wgpu::Color {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        },
        current_texture: None,
        current_view: None,
        encoder: None,
    };

    with_registry(|registry| {
        registry.insert(
            id,
            RegistryEntry {
                handle: NativeHandle::Window(SendWindow(state)),
                ref_count: 1,
            },
        );
    });
    id as i64
}

pub fn registry_window_update(handle_id: i64) -> bool {
    if handle_id < 0 {
        return false;
    }
    let id = handle_id as usize;
    with_registry(|registry| {
        if let Some(entry) = registry.get_mut(&id) {
            if let NativeHandle::Window(SendWindow(state)) = &mut entry.handle {
                // If there's an active encoder, finish and submit it
                if let Some(encoder) = state.encoder.take() {
                    state.queue.submit(std::iter::once(encoder.finish()));
                }

                // If we grabbed a surface texture this frame, present it
                if let Some(texture) = state.current_texture.take() {
                    texture.present();
                }

                state.current_view = None;

                // Synchronous immediate return - we assume window is open until dropped
                true
            } else {
                false
            }
        } else {
            false
        }
    })
}

pub fn registry_window_close(handle_id: i64) {
    // Closing the window is as simple as freeing its handle!
    registry_free(handle_id);
}

// ── File IO Orchestration ─────────────────────────────────────────

pub fn registry_file_create(path: String) -> i64 {
    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap_or_else(|e| e.into_inner());
    let id = *id_guard;
    *id_guard += 1;

    match File::create(&path) {
        Ok(file) => {
            with_registry(|registry| {
                registry.insert(
                    id,
                    RegistryEntry {
                        handle: NativeHandle::File(file),
                        ref_count: 1,
                    },
                );
            });
            id as i64
        }
        Err(e) => {
            eprintln!("[KnotenCore FileIO] Error creating file '{}': {}", path, e);
            -1
        }
    }
}

pub fn registry_file_write(handle_id: i64, content: String) {
    if handle_id < 0 {
        return;
    }
    let id = handle_id as usize;
    with_registry(|registry| {
        if let Some(entry) = registry.get_mut(&id) {
            if let NativeHandle::File(file) = &mut entry.handle {
                if let Err(e) = file.write_all(content.as_bytes()) {
                    eprintln!(
                        "[KnotenCore FileIO] Failed to write to file handle {}: {}",
                        handle_id, e
                    );
                }
            } else {
                eprintln!("[KnotenCore FileIO] Handle {} is not a File.", handle_id);
            }
        } else {
            eprintln!("[KnotenCore FileIO] Handle {} not found.", handle_id);
        }
    });
}

// ── GPU Orchestration ────────────────────────────────────────────────

pub fn registry_gpu_init() -> i64 {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    })) {
        Some(a) => a,
        None => {
            eprintln!("[KnotenCore GPU] No suitable GPU adapter found.");
            return -1;
        }
    };

    let adapter_info = adapter.get_info();
    println!(
        "[KnotenCore GPU] Adapter: {} ({:?})",
        adapter_info.name, adapter_info.backend
    );

    let (device, queue) = match pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("KnotenCore GPU Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        },
        None,
    )) {
        Ok(dq) => dq,
        Err(e) => {
            eprintln!("[KnotenCore GPU] Failed to create device: {}", e);
            return -1;
        }
    };

    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap_or_else(|e| e.into_inner());
    let id = *id_guard;
    *id_guard += 1;

    with_registry(|registry| {
        registry.insert(
            id,
            RegistryEntry {
                handle: NativeHandle::GpuContext(GpuContext {
                    instance,
                    adapter,
                    device,
                    queue,
                }),
                ref_count: 1,
            },
        );
    });

    id as i64
}

pub fn registry_fill_color(window_handle: i64, r: i64, g: i64, b: i64) {
    if window_handle < 0 {
        return;
    }
    let id = window_handle as usize;
    let color = wgpu::Color {
        r: (r.max(0).min(255) as f64) / 255.0,
        g: (g.max(0).min(255) as f64) / 255.0,
        b: (b.max(0).min(255) as f64) / 255.0,
        a: 1.0,
    };
    with_registry(|registry| {
        if let Some(entry) = registry.get_mut(&id) {
            if let NativeHandle::Window(SendWindow(state)) = &mut entry.handle {
                state.clear_color = color;
                if state.encoder.is_none() {
                    state.current_texture = Some(state.surface.get_current_texture().unwrap());
                    state.current_view = Some(
                        state
                            .current_texture
                            .as_ref()
                            .unwrap()
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    );
                    state.encoder = Some(state.device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor {
                            label: Some("Clear Pass Encoder"),
                        },
                    ));
                }

                let _render_pass = state.encoder.as_mut().unwrap().begin_render_pass(
                    &wgpu::RenderPassDescriptor {
                        label: Some("Clear Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: state.current_view.as_ref().unwrap(),
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(state.clear_color),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    },
                );
            }
        }
    });
}

// ── Voxel World Orchestration ─────────────────────────────────────────

pub fn registry_voxel_world_create(_width: i64, _height: i64, _title: String) -> i64 {
    eprintln!("[KnotenCore Voxel] Legacy Voxel module disabled in Sprint 51.");
    -1
}

pub fn registry_voxel_add_block(_world_handle: i64, _x: i64, _y: i64, _z: i64) {}

/// Renders one frame of the voxel scene. Returns true while the window is open.
pub fn registry_voxel_render_frame(_world_handle: i64) -> bool {
    false
}

pub struct RegistryModule;

impl crate::natives::NativeModule for RegistryModule {
    fn handle(
        &self,
        func_name: &str,
        args: &[crate::executor::RelType],
    ) -> Option<crate::executor::ExecResult> {
        use crate::natives::bridge::BridgeModule;
        crate::natives::bridge::CoreBridge.handle("registry", func_name, args)
    }
}

// ── Texture Orchestration ─────────────────────────────────────────

pub fn registry_texture_load(path: String) -> i64 {
    let img = match image::open(&path) {
        Ok(img) => img.to_rgba8(),
        Err(e) => {
            eprintln!("[KnotenCore Texture] Failed to load '{}': {}", path, e);
            return -1;
        }
    };
    let dimensions = img.dimensions();

    let (device, queue) = with_registry(|registry| {
        for entry in registry.values() {
            if let NativeHandle::Window(SendWindow(state)) = &entry.handle {
                return Some((state.device.clone(), state.queue.clone()));
            }
        }
        None
    })
    .expect("Cannot load texture without an active WGPU window");

    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some(&path),
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &diffuse_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &img,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        texture_size,
    );

    let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
            },
        ],
        label: Some("diffuse_bind_group"),
    });

    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap_or_else(|e| e.into_inner());
    let id = *id_guard;
    *id_guard += 1;

    with_registry(|registry| {
        registry.insert(
            id,
            RegistryEntry {
                handle: NativeHandle::Texture(TextureAsset {
                    bind_group: Arc::new(bind_group),
                    width: dimensions.0,
                    height: dimensions.1,
                }),
                ref_count: 1,
            },
        );
    });

    id as i64
}

pub fn registry_draw_quad_3d(
    window_handle: i64,
    texture_handle: i64,
    x: i64,
    y: i64,
    _z: i64, // reserved for depth sorting
) {
    if window_handle < 0 || texture_handle < 0 {
        return;
    }
    let win_id = window_handle as usize;
    let tex_id = texture_handle as usize;

    let tex_data: Option<(u32, u32, Arc<wgpu::BindGroup>)> = with_registry(|registry| {
        if let Some(entry) = registry.get(&tex_id) {
            if let NativeHandle::Texture(tex) = &entry.handle {
                return Some((tex.width, tex.height, tex.bind_group.clone()));
            }
        }
        None
    });

    let (tw, th, bind_group) = match tex_data {
        Some(d) => d,
        None => return,
    };

    with_registry(|registry| {
        if let Some(win_entry) = registry.get_mut(&win_id) {
            if let NativeHandle::Window(SendWindow(state)) = &mut win_entry.handle {
                let sw = state.width as f32;
                let sh = state.height as f32;
                let nx = (x as f32 / sw) * 2.0 - 1.0;
                let ny = 1.0 - (y as f32 / sh) * 2.0;
                let nw = (tw as f32 / sw) * 2.0;
                let nh = (th as f32 / sh) * 2.0;

                let vertices = [
                    RegistryVertex {
                        position: [nx, ny, 0.0],
                        tex_coords: [0.0, 0.0],
                    },
                    RegistryVertex {
                        position: [nx, ny - nh, 0.0],
                        tex_coords: [0.0, 1.0],
                    },
                    RegistryVertex {
                        position: [nx + nw, ny - nh, 0.0],
                        tex_coords: [1.0, 1.0],
                    },
                    RegistryVertex {
                        position: [nx + nw, ny - nh, 0.0],
                        tex_coords: [1.0, 1.0],
                    },
                    RegistryVertex {
                        position: [nx + nw, ny, 0.0],
                        tex_coords: [1.0, 0.0],
                    },
                    RegistryVertex {
                        position: [nx, ny, 0.0],
                        tex_coords: [0.0, 0.0],
                    },
                ];

                let vertex_buffer =
                    state
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Quad VB"),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                if state.encoder.is_none() {
                    state.current_texture = Some(state.surface.get_current_texture().unwrap());
                    state.current_view = Some(
                        state
                            .current_texture
                            .as_ref()
                            .unwrap()
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    );
                    state.encoder = Some(state.device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor {
                            label: Some("Quad Encoder"),
                        },
                    ));
                }

                let mut render_pass = state.encoder.as_mut().unwrap().begin_render_pass(
                    &wgpu::RenderPassDescriptor {
                        label: Some("Quad Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: state.current_view.as_ref().unwrap(),
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    },
                );

                render_pass.set_pipeline(&state.pipeline);
                // Dereference Arc to wgpu::BindGroup
                render_pass.set_bind_group(0, &*bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..6, 0..1);
            }
        }
    });
}
