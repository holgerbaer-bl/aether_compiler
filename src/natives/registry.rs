use glam::{Mat4, Vec3};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use wgpu::util::DeviceExt;
use winit::window::Window as WinitWindow;

use std::collections::HashSet;
#[cfg(not(target_os = "windows"))]
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct InputState {
    pub keys: HashSet<KeyCode>,
    pub mouse_dx: f32,
    pub mouse_dy: f32,
    pub last_char: u32,
}

pub enum RenderCommand {
    CreateWindow {
        id: usize,
        title: String,
        width: u32,
        height: u32,
    },
    DrawSphere {
        window_id: usize,
        texture_id: usize,
        x: f32, y: f32, z: f32,
        radius: f32,
        rings: u32,
        sectors: u32,
    },
    DrawCube {
        window_id: usize,
        texture_id: usize,
        x: f32, y: f32, z: f32,
        w: f32, h: f32, d: f32,
    },
    DrawCylinder {
        window_id: usize,
        texture_id: usize,
        x: f32, y: f32, z: f32,
        radius: f32, height: f32, segments: u32,
    },
    DrawQuad3D {
        window_id: usize,
        texture_id: usize,
        x: f32, y: f32, z: f32,
        scale_x: f32, scale_y: f32,
    },
    UpdateWindow(usize),
    CloseWindow(usize),
    AddMesh {
        name: String,
        vertices: Vec<RegistryVertex>,
        indices: Vec<u32>,
    },
}

static RENDER_TX: Mutex<Option<std::sync::mpsc::Sender<RenderCommand>>> = Mutex::new(None);
static SENT_MESHES: Mutex<Option<HashSet<String>>> = Mutex::new(None);

pub fn set_render_channel(tx: std::sync::mpsc::Sender<RenderCommand>) {
    let mut guard = RENDER_TX.lock().unwrap();
    *guard = Some(tx);
}

fn send_render_command(cmd: RenderCommand) {
    let guard = RENDER_TX.lock().unwrap();
    if let Some(tx) = guard.as_ref() {
        let _ = tx.send(cmd);
    }
}

// Proxy for a Window to be used by the background executor.
pub struct WindowProxy {
    pub id: usize,
    pub input: Arc<Mutex<InputState>>,
}

unsafe impl Send for WindowProxy {}
unsafe impl Sync for WindowProxy {}

pub struct RegistryWindowState {
    pub window: Arc<WinitWindow>,
    pub input: Arc<Mutex<InputState>>,
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

    // 3D Resources
    pub depth_texture_view: wgpu::TextureView,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub model_buffer: wgpu::Buffer,
    pub model_bind_group: wgpu::BindGroup,
    pub geometry_cache: HashMap<String, CachedMesh>,
    pub texture_cache: HashMap<usize, wgpu::BindGroup>,
    pub default_texture_bind_group: wgpu::BindGroup,
    pub commands: Vec<RenderCommand>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RegistryVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

pub struct CachedMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

// The types of resources we can manage
pub enum NativeHandle {
    Counter(StatefulCounter),
    Window(WindowProxy),
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
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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

    send_render_command(RenderCommand::CreateWindow {
        id,
        title,
        width: w,
        height: h,
    });

    let input = Arc::new(Mutex::new(InputState {
        keys: HashSet::new(),
        mouse_dx: 0.0,
        mouse_dy: 0.0,
        last_char: 0,
    }));

    with_registry(|registry| {
        registry.insert(
            id,
            RegistryEntry {
                handle: NativeHandle::Window(WindowProxy { id, input }),
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
    send_render_command(RenderCommand::UpdateWindow(id));
    
    // We assume the window is open unless we receive a message back or have a way to check.
    // For now, we return true. The main loop will handle window closure.
    true
}

pub fn registry_window_close(handle_id: i64) {
    if handle_id < 0 {
        return;
    }
    let id = handle_id as usize;
    send_render_command(RenderCommand::CloseWindow(id));
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
    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap_or_else(|e| e.into_inner());
    let id = *id_guard;
    *id_guard += 1;

    // This is synchronous and can be slow, but it's called once.
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("Failed to find WGPU adapter");

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
    // Note: We could send a Command for this too.
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
        permissions: &crate::executor::AgentPermissions,
    ) -> Option<crate::executor::ExecResult> {
        use crate::natives::bridge::BridgeModule;
        crate::natives::bridge::CoreBridge.handle("registry", func_name, args, permissions)
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
            if let NativeHandle::GpuContext(ctx) = &entry.handle {
                return Some((ctx.device.clone(), ctx.queue.clone()));
            }
        }
        None
    })
    .expect("Cannot load texture without an active WGPU context. Call registry_gpu_init or create a window first.");

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
    x: f32,
    y: f32,
    z: f32,
    scale_x: f32,
    scale_y: f32,
) {
    if window_handle < 0 || texture_handle < 0 {
        return;
    }
    send_render_command(RenderCommand::DrawQuad3D {
        window_id: window_handle as usize,
        texture_id: texture_handle as usize,
        x, y, z,
        scale_x, scale_y,
    });
}

pub fn registry_draw_sphere(
    window_handle: i64,
    texture_handle: i64,
    radius: f32,
    rings: i32,
    sectors: i32,
    x: f32,
    y: f32,
    z: f32,
) {
    if window_handle < 0 || texture_handle < 0 {
        return;
    }
    let rings = rings.max(3) as u32;
    let sectors = sectors.max(3) as u32;
    let mesh_name = format!("sphere_{}_{}", rings, sectors);

    let mut guard = SENT_MESHES.lock().unwrap();
    let sent = guard.get_or_insert_with(HashSet::new);
    if !sent.contains(&mesh_name) {
        let (vertices, indices) = generate_uv_sphere(rings, sectors);
        send_render_command(RenderCommand::AddMesh {
            name: mesh_name.clone(),
            vertices,
            indices,
        });
        sent.insert(mesh_name.clone());
    }

    send_render_command(RenderCommand::DrawSphere {
        window_id: window_handle as usize,
        texture_id: texture_handle as usize,
        x, y, z,
        radius,
        rings,
        sectors,
    });
}

fn generate_uv_sphere(rings: u32, sectors: u32) -> (Vec<RegistryVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for r in 0..=rings {
        let phi = std::f32::consts::PI * (r as f32 / rings as f32);
        for s in 0..=sectors {
            let theta = 2.0 * std::f32::consts::PI * (s as f32 / sectors as f32);
            let x = phi.sin() * theta.cos();
            let y = phi.cos();
            let z = phi.sin() * theta.sin();
            let u = s as f32 / sectors as f32;
            let v = r as f32 / rings as f32;
            vertices.push(RegistryVertex {
                position: [x, y, z],
                tex_coords: [u, v],
            });
        }
    }

    for r in 0..rings {
        for s in 0..sectors {
            let first = r * (sectors + 1) + s;
            let second = first + sectors + 1;
            indices.push(first);
            indices.push(second);
            indices.push(first + 1);
            indices.push(second);
            indices.push(second + 1);
            indices.push(first + 1);
        }
    }
    (vertices, indices)
}

pub fn registry_draw_cube(
    window_handle: i64,
    texture_handle: i64,
    w: f32,
    h: f32,
    d: f32,
    x: f32,
    y: f32,
    z: f32,
) {
    if window_handle < 0 || texture_handle < 0 {
        return;
    }
    send_render_command(RenderCommand::DrawCube {
        window_id: window_handle as usize,
        texture_id: texture_handle as usize,
        x, y, z,
        w, h, d,
    });
}

pub fn registry_draw_cylinder(
    window_handle: i64,
    texture_handle: i64,
    radius: f32,
    height: f32,
    segments: i32,
    x: f32,
    y: f32,
    z: f32,
) {
    if window_handle < 0 || texture_handle < 0 {
        return;
    }
    let segments = segments.max(3) as u32;
    let mesh_name = format!("cylinder_{}", segments);

    let mut guard = SENT_MESHES.lock().unwrap();
    let sent = guard.get_or_insert_with(HashSet::new);
    if !sent.contains(&mesh_name) {
        let (vertices, indices) = generate_cylinder(segments);
        send_render_command(RenderCommand::AddMesh {
            name: mesh_name.clone(),
            vertices,
            indices,
        });
        sent.insert(mesh_name.clone());
    }

    send_render_command(RenderCommand::DrawCylinder {
        window_id: window_handle as usize,
        texture_id: texture_handle as usize,
        x, y, z,
        radius,
        height,
        segments,
    });
}

fn generate_cylinder(segments: u32) -> (Vec<RegistryVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Top center
    vertices.push(RegistryVertex { position: [0.0, 0.5, 0.0], tex_coords: [0.5, 0.5] });
    // Bottom center
    vertices.push(RegistryVertex { position: [0.0, -0.5, 0.0], tex_coords: [0.5, 0.5] });

    let base_idx_top = 0;
    let base_idx_bottom = 1;

    // Cycle through segments
    for i in 0..=segments {
        let theta = 2.0 * std::f32::consts::PI * (i as f32 / segments as f32);
        let x = theta.cos();
        let z = theta.sin();
        let u = i as f32 / segments as f32;

        // Top cap vertex
        vertices.push(RegistryVertex { position: [x, 0.5, z], tex_coords: [u, 0.0] });
        // Bottom cap vertex
        vertices.push(RegistryVertex { position: [x, -0.5, z], tex_coords: [u, 1.0] });
    }

    for i in 0..segments {
        let top0 = 2 + i * 2;
        let bot0 = top0 + 1;
        let top1 = top0 + 2;
        let bot1 = top1 + 1;

        // Side faces
        indices.push(top0);
        indices.push(bot0);
        indices.push(top1);
        indices.push(bot0);
        indices.push(bot1);
        indices.push(top1);

        // Top cap
        indices.push(base_idx_top);
        indices.push(top1);
        indices.push(top0);

        // Bottom cap
        indices.push(base_idx_bottom);
        indices.push(bot0);
        indices.push(bot1);
    }
    (vertices, indices)
}

pub fn registry_set_camera(fov_degrees: f32, cam_x: f32, cam_y: f32, cam_z: f32) {
    // Note: Camera setting is now more complex.
    // For now, we omit the implementation here as it requires a RenderCommand or shared state update.
}

pub fn registry_is_key_pressed(keycode: i64) -> f32 {
    let mut pressed = false;
    with_registry(|registry| {
        for entry in registry.values() {
            if let NativeHandle::Window(proxy) = &entry.handle {
                let input = proxy.input.lock().unwrap_or_else(|e| e.into_inner());
                for k in &input.keys {
                    if *k as i64 == keycode {
                        pressed = true;
                        break;
                    }
                }
            }
        }
    });
    if pressed { 1.0 } else { 0.0 }
}

pub fn registry_get_mouse_delta_x() -> f32 {
    let mut acc = 0.0;
    with_registry(|registry| {
        for entry in registry.values() {
            if let NativeHandle::Window(proxy) = &entry.handle {
                let input = proxy.input.lock().unwrap_or_else(|e| e.into_inner());
                acc += input.mouse_dx;
            }
        }
    });
    acc
}

pub fn registry_get_mouse_delta_y() -> f32 {
    let mut acc = 0.0;
    with_registry(|registry| {
        for entry in registry.values() {
            if let NativeHandle::Window(proxy) = &entry.handle {
                let input = proxy.input.lock().unwrap_or_else(|e| e.into_inner());
                acc += input.mouse_dy;
            }
        }
    });
    acc
}

pub fn registry_get_last_char() -> i64 {
    let mut last = 0;
    with_registry(|registry| {
        for entry in registry.values() {
            if let NativeHandle::Window(proxy) = &entry.handle {
                let input = proxy.input.lock().unwrap_or_else(|e| e.into_inner());
                if input.last_char != 0 {
                    last = input.last_char as i64;
                }
            }
        }
    });
    last
}

pub fn registry_read_file(path: String) -> String {
    std::fs::read_to_string(&path).unwrap_or_else(|_| "".to_string())
}

pub fn registry_write_file(path: String, content: String) -> bool {
    std::fs::write(&path, content).is_ok()
}

pub fn registry_get_ultimate_answer() -> i64 {
    42
}
