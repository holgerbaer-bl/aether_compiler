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
                
                // Setup proper 3D pipeline
                let camera_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Camera BGL"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                        count: None,
                    }],
                });

                let material_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Material BGL"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture { multisampled: false, view_dimension: wgpu::TextureViewDimension::D2, sample_type: wgpu::TextureSampleType::Float { filterable: true } },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

                let model_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Model BGL"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                        count: None,
                    }],
                });

                let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Main Pipeline Layout"),
                    bind_group_layouts: &[&camera_bgl, &material_bgl, &model_bgl],
                    push_constant_ranges: &[],
                });

                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("3D Shader"),
                    source: wgpu::ShaderSource::Wgsl("
                        struct CameraUniform { view_proj: mat4x4<f32>, camera_pos: vec4<f32> };
                        @group(0) @binding(0) var<uniform> camera: CameraUniform;

                        struct ModelUniform { transform: mat4x4<f32> };
                        @group(2) @binding(0) var<uniform> model: ModelUniform;

                        struct VertexInput {
                            @location(0) position: vec3<f32>,
                            @location(1) tex_coords: vec2<f32>,
                        };
                        struct VertexOutput {
                            @builtin(position) clip_position: vec4<f32>,
                            @location(0) tex_coords: vec2<f32>,
                        };

                        @vertex fn vs_main(input: VertexInput) -> VertexOutput {
                            var out: VertexOutput;
                            out.clip_position = camera.view_proj * model.transform * vec4<f32>(input.position, 1.0);
                            out.tex_coords = input.tex_coords;
                            return out;
                        }

                        @group(1) @binding(0) var t_diffuse: texture_2d<f32>;
                        @group(1) @binding(1) var s_diffuse: sampler;

                        @fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                            return textureSample(t_diffuse, s_diffuse, in.tex_coords);
                        }
                    ".into()),
                });

                let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("3D Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: 20, // 5 * 4 bytes
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 },
                                wgpu::VertexAttribute { offset: 12, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
                            ],
                        }],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
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
                    size: 80, // Mat4 (64) + Vec4 (16)
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Camera Bind Group"),
                    layout: &camera_bgl,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }],
                });

                let model_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Model Buffer"),
                    size: 64, // Mat4
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Model Bind Group"),
                    layout: &model_bgl,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: model_buffer.as_entire_binding(),
                    }],
                });

                let default_texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Default Texture"),
                    size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                    mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                queue.write_texture(
                    wgpu::ImageCopyTexture { texture: &default_texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                    &[255, 255, 255, 255],
                    wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
                    wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                );
                let default_view = default_texture.create_view(&wgpu::TextureViewDescriptor::default());
                let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

                let default_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Default Material Bind Group"),
                    layout: &material_bgl,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&default_view) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&default_sampler) },
                    ],
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
                    model_buffer,
                    model_bind_group,
                    geometry_cache: HashMap::new(),
                    texture_cache: HashMap::new(),
                    default_texture_bind_group,
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
            RenderCommand::AddMesh { name, vertices, indices } => {
                for state in self.windows.values_mut() {
                    use wgpu::util::DeviceExt;
                    let vertex_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Mesh {} VBO", name)),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                    let index_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Mesh {} IBO", name)),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });
                    state.geometry_cache.insert(name.clone(), CachedMesh {
                        vertex_buffer,
                        index_buffer,
                        index_count: indices.len() as u32,
                    });
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
                        let (mesh_name, texture_id, transform) = match &cmd {
                            RenderCommand::DrawSphere { rings, sectors, texture_id, x, y, z, radius, .. } => {
                                let t = glam::Mat4::from_translation(glam::Vec3::new(*x, *y, *z));
                                let s = glam::Mat4::from_scale(glam::Vec3::splat(*radius));
                                (format!("sphere_{}_{}", rings, sectors), *texture_id, t * s)
                            }
                            RenderCommand::DrawCube { texture_id, x, y, z, w, h, d, .. } => {
                                let t = glam::Mat4::from_translation(glam::Vec3::new(*x, *y, *z));
                                let s = glam::Mat4::from_scale(glam::Vec3::new(*w, *h, *d));
                                ("cube".to_string(), *texture_id, t * s)
                            }
                            RenderCommand::DrawCylinder { segments, texture_id, x, y, z, radius, height, .. } => {
                                let t = glam::Mat4::from_translation(glam::Vec3::new(*x, *y, *z));
                                let s = glam::Mat4::from_scale(glam::Vec3::new(*radius, *height, *radius));
                                (format!("cylinder_{}", segments), *texture_id, t * s)
                            }
                            _ => continue,
                        };

                        if let Some(mesh) = state.geometry_cache.get(&mesh_name) {
                            // Update model matrix
                            state.queue.write_buffer(&state.model_buffer, 0, bytemuck::cast_slice(&transform.to_cols_array()));

                            rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                            rpass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                            
                            let mat_bg = state.texture_cache.get(&texture_id).unwrap_or(&state.default_texture_bind_group);
                            rpass.set_bind_group(1, mat_bg, &[]);
                            rpass.set_bind_group(2, &state.model_bind_group, &[]);
                            
                            rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
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
