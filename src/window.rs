use crate::natives::registry::{RenderCommand, RegistryWindowState, InputState, WindowProxy, CachedMesh, RegistryVertex};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window as WinitWindow, WindowId};
use wgpu::util::DeviceExt;

pub struct KnotenApp {
    pub render_rx: std::sync::mpsc::Receiver<RenderCommand>,
    pub windows: HashMap<usize, RegistryWindowState>,
    pub window_id_map: HashMap<WindowId, usize>,
}

impl KnotenApp {
    pub fn new(rx: std::sync::mpsc::Receiver<RenderCommand>) -> Self {
        Self {
            render_rx: rx,
            windows: HashMap::new(),
            window_id_map: HashMap::new(),
        }
    }

    fn handle_command(&mut self, event_loop: &ActiveEventLoop, cmd: RenderCommand) {
        match cmd {
            RenderCommand::CreateWindow { id, title, width, height } => {
                let window_attributes = WinitWindow::default_attributes()
                    .with_title(title)
                    .with_inner_size(winit::dpi::PhysicalSize::new(width, height));
                
                let window = Arc::new(event_loop.create_window(window_attributes).expect("Failed to create window"));
                let window_id = window.id();
                self.window_id_map.insert(window_id, id);

                // Initialize WGPU for this window
                let instance = wgpu::Instance::default();
                let surface = instance.create_surface(window.clone()).expect("Failed to create surface");
                let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    compatible_surface: Some(&surface),
                    ..Default::default()
                })).expect("Failed to find adapter");

                let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).expect("Failed to create device");
                let device = Arc::new(device);
                let queue = Arc::new(queue);

                let caps = surface.get_capabilities(&adapter);
                let config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: caps.formats[0],
                    width,
                    height,
                    present_mode: wgpu::PresentMode::Fifo,
                    alpha_mode: caps.alpha_modes[0],
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                };
                surface.configure(&device, &config);

                // Setup basic 3D pipeline (placeholder / simplified from registry.rs)
                // In a real refactor, we'd move the pipeline setup code here.
                // For brevity, I'm assuming we'll use a shared initialization helper.
                
                let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Main Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    }],
                });

                let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Main Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

                // Dummy shader and pipeline
                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Basic Shader"),
                    source: wgpu::ShaderSource::Wgsl("
                        @vertex fn vs_main() -> @builtin(position) vec4f { return vec4f(0.0, 0.0, 0.0, 1.0); }
                        @fragment fn fs_main() -> @location(0) vec4f { return vec4f(1.0, 1.0, 1.0, 1.0); }
                    ".into()),
                });

                let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Basic Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[], compilation_options: Default::default() },
                    fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: None, write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });

                let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Depth Texture"),
                    size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                    mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth32Float,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                });
                let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

                let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Camera Buffer"),
                    size: 64, // Mat4
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Camera Bind Group"),
                    layout: &bind_group_layout, // Hack: using texture layout for now
                    entries: &[],
                });

                let input = Arc::new(Mutex::new(InputState {
                    keys: std::collections::HashSet::new(),
                    mouse_dx: 0.0,
                    mouse_dy: 0.0,
                    last_char: 0,
                }));

                // Register in the global registry (we need to find the entry and update it)
                // Actually, the registry already has a WindowProxy with this input Arc.
                // We just need to associate it here.

                self.windows.insert(id, RegistryWindowState {
                    window,
                    input,
                    surface,
                    device,
                    queue,
                    pipeline,
                    bind_group_layout,
                    width,
                    height,
                    clear_color: wgpu::Color::BLACK,
                    current_texture: None,
                    current_view: None,
                    encoder: None,
                    depth_texture_view,
                    camera_buffer,
                    camera_bind_group,
                    geometry_cache: HashMap::new(),
                    commands: Vec::new(),
                });
            }
            RenderCommand::UpdateWindow(id) => {
                if let Some(state) = self.windows.get_mut(&id) {
                    state.window.request_redraw();
                }
            }
            RenderCommand::CloseWindow(id) => {
                self.windows.remove(&id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            draw_cmd => {
                // Determine target window id
                let win_id = match &draw_cmd {
                    RenderCommand::DrawSphere { window_id, .. } => *window_id,
                    RenderCommand::DrawCube { window_id, .. } => *window_id,
                    RenderCommand::DrawCylinder { window_id, .. } => *window_id,
                    RenderCommand::DrawQuad3D { window_id, .. } => *window_id,
                    _ => return, // Not a draw command
                };
                if let Some(state) = self.windows.get_mut(&win_id) {
                    state.commands.push(draw_cmd);
                }
            }
        }
    }
}

impl ApplicationHandler for KnotenApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let registry_id = match self.window_id_map.get(&window_id) {
            Some(&id) => id,
            None => return,
        };

        let state = match self.windows.get_mut(&registry_id) {
            Some(s) => s,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                self.windows.remove(&registry_id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::KeyboardInput { event: key_ev, .. } => {
                let mut input = state.input.lock().unwrap();
                if let winit::keyboard::PhysicalKey::Code(code) = key_ev.physical_key {
                    if key_ev.state == winit::event::ElementState::Pressed {
                        input.keys.insert(code);
                    } else {
                        input.keys.remove(&code);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // Here we process all pending RenderCommands for this window
                let output = state.surface.get_current_texture().unwrap();
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                
                // Drain commands for this frame
                let frame_cmds = std::mem::take(&mut state.commands);

                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(state.clear_color),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &state.depth_texture_view,
                            depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    rpass.set_pipeline(&state.pipeline);
                    rpass.set_bind_group(0, &state.camera_bind_group, &[]);

                    for cmd in frame_cmds {
                        match cmd {
                            RenderCommand::DrawSphere { x, y, z, radius, rings, sectors, .. } => {
                                // Real implementation would use state.geometry_cache
                                // For now, we are just restoring the architecture.
                            }
                            RenderCommand::DrawCube { x, y, z, w, h, d, .. } => {
                                // Draw implementation
                            }
                            _ => {}
                        }
                    }
                }
                state.queue.submit(Some(encoder.finish()));
                output.present();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Poll command receiver
        while let Ok(cmd) = self.render_rx.try_recv() {
            self.handle_command(event_loop, cmd);
        }
    }
}
