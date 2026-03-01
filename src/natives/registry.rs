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

// Global thread-safe registry
// Instead of lazy_static we'll use a const Mutex with an Option since lazy_static might not be available
static COUNTER_REGISTRY: Mutex<Option<HashMap<usize, RegistryEntry>>> = Mutex::new(None);
static COUNTER_NEXT_ID: Mutex<usize> = Mutex::new(1);

fn with_registry<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<usize, RegistryEntry>) -> R,
{
    let mut option_guard = COUNTER_REGISTRY.lock().unwrap();
    if option_guard.is_none() {
        *option_guard = Some(HashMap::new());
    }
    f(option_guard.as_mut().unwrap())
}

// ── Lifecycle FFI Implementations ─────────────────────────────────

pub fn registry_retain(handle_id: i64) {
    let id = handle_id as usize;
    with_registry(|registry| {
        if let Some(entry) = registry.get_mut(&id) {
            entry.ref_count += 1;
        }
    });
}

pub fn registry_release(handle_id: i64) {
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
    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap();
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
    let id = handle_id as usize;
    with_registry(|registry| {
        if registry.remove(&id).is_some() {
            // Memory freed natively
        } else {
            eprintln!(
                "[KnotenCore Registry] Warning: Double free or invalid handle {}.",
                handle_id
            );
        }
    });
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
    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap();
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
    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap();
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

    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap();
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
