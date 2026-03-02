use minifb::{Window, WindowOptions};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

// Wrapper for Window to bypass non-Send restriction. Safe because our executor is single-threaded.
pub struct SendWindow(pub RegistryWindowState);
unsafe impl Send for SendWindow {}
unsafe impl Sync for SendWindow {}

pub struct RegistryWindowState {
    pub window: Window,
    pub buffer: Vec<u32>,
    pub width: usize,
    pub height: usize,
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

// Texture loaded into memory for rendering
pub struct TextureAsset {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>, // RGBA packed as 0x00RRGGBB for minifb
}
unsafe impl Send for TextureAsset {}
unsafe impl Sync for TextureAsset {}

// VoxelWorld — isometric software-rendered voxel scene
pub struct VoxelWorldState {
    pub window: minifb::Window,
    pub buffer: Vec<u32>,
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
    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap();
    let id = *id_guard;
    *id_guard += 1;

    let w = width as usize;
    let h = height as usize;

    // Create an initial framebuffer (solid color so we see something)
    let buffer = vec![0x333333; w * h];

    if let Ok(mut window) = Window::new(&title, w, h, WindowOptions::default()) {
        window.set_target_fps(60);
        let state = RegistryWindowState {
            window,
            buffer,
            width: w,
            height: h,
        };
        with_registry(|registry| {
            registry.insert(
                id,
                RegistryEntry {
                    handle: NativeHandle::Window(SendWindow(state)),
                    ref_count: 1, // RC starts at 1
                },
            );
        });
        id as i64
    } else {
        eprintln!("[KnotenCore Registry] Failed to create window.");
        -1
    }
}

pub fn registry_window_update(handle_id: i64) -> bool {
    if handle_id < 0 {
        return false;
    }
    let id = handle_id as usize;
    with_registry(|registry| {
        if let Some(entry) = registry.get_mut(&id) {
            if let NativeHandle::Window(SendWindow(state)) = &mut entry.handle {
                // Update the window with its internal buffer. Returns true if open.
                state
                    .window
                    .update_with_buffer(&state.buffer, state.width, state.height)
                    .is_ok()
                    && state.window.is_open()
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
    // Pack RGB into the 0x00RRGGBB format that minifb expects
    let color: u32 = ((r.max(0).min(255) as u32) << 16)
        | ((g.max(0).min(255) as u32) << 8)
        | (b.max(0).min(255) as u32);
    with_registry(|registry| {
        if let Some(entry) = registry.get_mut(&id) {
            if let NativeHandle::Window(SendWindow(state)) = &mut entry.handle {
                state.buffer.iter_mut().for_each(|px| *px = color);
            } else {
                eprintln!("[KnotenCore GPU] Handle {} is not a Window.", window_handle);
            }
        } else {
            eprintln!(
                "[KnotenCore GPU] Window handle {} not found.",
                window_handle
            );
        }
    });
}

// ── Voxel World Orchestration ─────────────────────────────────────────

pub fn registry_voxel_world_create(width: i64, height: i64, title: String) -> i64 {
    let w = width as usize;
    let h = height as usize;
    let buffer = vec![0x0d1b2au32; w * h];
    match minifb::Window::new(
        &title,
        w,
        h,
        minifb::WindowOptions {
            resize: false,
            ..minifb::WindowOptions::default()
        },
    ) {
        Ok(mut win) => {
            win.set_target_fps(60);
            let mut id_guard = COUNTER_NEXT_ID.lock().unwrap_or_else(|e| e.into_inner());
            let id = *id_guard;
            *id_guard += 1;
            with_registry(|registry| {
                registry.insert(
                    id,
                    RegistryEntry {
                        handle: NativeHandle::VoxelWorld(SendVoxelWorld(VoxelWorldState {
                            window: win,
                            buffer,
                            width: w,
                            height: h,
                            voxels: Vec::new(),
                        })),
                        ref_count: 1,
                    },
                );
            });
            id as i64
        }
        Err(e) => {
            eprintln!("[KnotenCore Voxel] Failed to create window: {}", e);
            -1
        }
    }
}

pub fn registry_voxel_add_block(world_handle: i64, x: i64, y: i64, z: i64) {
    if world_handle < 0 {
        return;
    }
    let id = world_handle as usize;
    with_registry(|registry| {
        if let Some(entry) = registry.get_mut(&id) {
            if let NativeHandle::VoxelWorld(SendVoxelWorld(state)) = &mut entry.handle {
                state.voxels.push([x as i32, y as i32, z as i32]);
            }
        }
    });
}

/// Renders one frame of the voxel scene. Returns true while the window is open.
pub fn registry_voxel_render_frame(world_handle: i64) -> bool {
    if world_handle < 0 {
        return false;
    }
    let id = world_handle as usize;
    with_registry(|registry| {
        if let Some(entry) = registry.get_mut(&id) {
            if let NativeHandle::VoxelWorld(SendVoxelWorld(state)) = &mut entry.handle {
                iso_render(&mut state.buffer, state.width, state.height, &state.voxels);
                return state
                    .window
                    .update_with_buffer(&state.buffer, state.width, state.height)
                    .is_ok()
                    && state.window.is_open();
            }
        }
        false
    })
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

    let width = img.width();
    let height = img.height();
    let mut pixels = Vec::with_capacity((width * height) as usize);
    for pixel in img.pixels() {
        let r = pixel[0] as u32;
        let g = pixel[1] as u32;
        let b = pixel[2] as u32;
        pixels.push((r << 16) | (g << 8) | b);
    }

    println!(
        "[KnotenCore Texture] Loaded '{}' ({}x{})",
        path, width, height
    );

    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap_or_else(|e| e.into_inner());
    let id = *id_guard;
    *id_guard += 1;

    with_registry(|registry| {
        registry.insert(
            id,
            RegistryEntry {
                handle: NativeHandle::Texture(TextureAsset {
                    width,
                    height,
                    pixels,
                }),
                ref_count: 1,
            },
        );
    });

    id as i64
}

/// Draw a textured quad into a minifb Window at pixel position (x,y).
/// This is a software blit — suitable for a Doom-style rasteriser.
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

    // First, grab texture data (clone out to avoid nested lock)
    let tex_data: Option<(u32, u32, Vec<u32>)> = with_registry(|registry| {
        if let Some(entry) = registry.get(&tex_id) {
            if let NativeHandle::Texture(tex) = &entry.handle {
                return Some((tex.width, tex.height, tex.pixels.clone()));
            }
        }
        None
    });

    let (tw, th, tpx) = match tex_data {
        Some(d) => d,
        None => {
            eprintln!(
                "[KnotenCore DrawQuad] Texture handle {} not found.",
                texture_handle
            );
            return;
        }
    };

    // Blit into window buffer
    with_registry(|registry| {
        if let Some(entry) = registry.get_mut(&win_id) {
            if let NativeHandle::Window(SendWindow(state)) = &mut entry.handle {
                let dx = x.max(0) as usize;
                let dy = y.max(0) as usize;
                for ty in 0..th as usize {
                    let screen_y = dy + ty;
                    if screen_y >= state.height {
                        break;
                    }
                    for tx in 0..tw as usize {
                        let screen_x = dx + tx;
                        if screen_x >= state.width {
                            break;
                        }
                        state.buffer[screen_y * state.width + screen_x] =
                            tpx[ty * tw as usize + tx];
                    }
                }
            } else {
                eprintln!(
                    "[KnotenCore DrawQuad] Handle {} is not a Window.",
                    window_handle
                );
            }
        } else {
            eprintln!(
                "[KnotenCore DrawQuad] Window handle {} not found.",
                window_handle
            );
        }
    });
}
