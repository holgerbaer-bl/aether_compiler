use crate::executor::{ExecutionEngine, VoxelVertex, InstanceData, PointLightStruct, MeshUniforms};
use wgpu::util::DeviceExt;
use std::sync::Arc;

impl ExecutionEngine {
    pub fn ensure_canvas_mesh_pipeline(&mut self) {
        if self.canvas_mesh_pipeline.is_some() { return; }
        let (device, config) = match (&self.device, &self.config) {
            (Some(d), Some(c)) => (d, c),
            _ => return,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Mesh3D Blinn-Phong Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../assets/mesh3d.wgsl").into()),
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Mesh3D BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { multisampled: false, view_dimension: wgpu::TextureViewDimension::D2, sample_type: wgpu::TextureSampleType::Float { filterable: true } },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { multisampled: false, view_dimension: wgpu::TextureViewDimension::D2, sample_type: wgpu::TextureSampleType::Float { filterable: true } },
                    count: None,
                },
            ],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mesh3D Pipeline Layout"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Mesh3D Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<VoxelVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 },
                            wgpu::VertexAttribute { offset: 12, shader_location: 1, format: wgpu::VertexFormat::Float32x3 },
                            wgpu::VertexAttribute { offset: 24, shader_location: 2, format: wgpu::VertexFormat::Float32x2 },
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute { offset: 0, shader_location: 3, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 16, shader_location: 4, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 32, shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 48, shader_location: 6, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 64, shader_location: 7, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 80, shader_location: 8, format: wgpu::VertexFormat::Float32x4 },
                        ],
                    },
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
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

        self.canvas_mesh_pipeline = Some(pipeline);
    }

    pub fn get_or_create_mesh(&mut self, prim_name: &str) -> (Arc<wgpu::Buffer>, Arc<wgpu::Buffer>, u32) {
        if let Some(m) = self.mesh_cache.get(prim_name) {
            return (m.vbo.clone(), m.ibo.clone(), m.index_count);
        }

        let (verts, indices) = match prim_name {
            "plane" => {
                let v = vec![
                    VoxelVertex { position: [-1.0, 0.0, 1.0], normal: [0.0,1.0,0.0], uv:[0.0,0.0] },
                    VoxelVertex { position: [1.0, 0.0, 1.0], normal: [0.0,1.0,0.0], uv:[1.0,0.0] },
                    VoxelVertex { position: [1.0, 0.0, -1.0], normal: [0.0,1.0,0.0], uv:[1.0,1.0] },
                    VoxelVertex { position: [-1.0, 0.0, -1.0], normal: [0.0,1.0,0.0], uv:[0.0,1.0] },
                ];
                (v, vec![0,1,2, 2,3,0])
            }
            _ => {
                let v_size = 0.5f32;
                let mut v = Vec::new();
                let mut idx = Vec::new();
                let mut add_face = |p0:[f32;3], p1:[f32;3], p2:[f32;3], p3:[f32;3], n:[f32;3]| {
                    let s = v.len() as u32;
                    v.push(VoxelVertex { position: p0, normal: n, uv: [0.0,0.0] });
                    v.push(VoxelVertex { position: p1, normal: n, uv: [1.0,0.0] });
                    v.push(VoxelVertex { position: p2, normal: n, uv: [1.0,1.0] });
                    v.push(VoxelVertex { position: p3, normal: n, uv: [0.0,1.0] });
                    idx.extend_from_slice(&[s, s+1, s+2, s, s+2, s+3]);
                };
                add_face([-v_size,-v_size,v_size], [v_size,-v_size,v_size], [v_size,v_size,v_size], [-v_size,v_size,v_size], [0.0,0.0,1.0]);
                add_face([v_size,-v_size,-v_size], [-v_size,-v_size,-v_size], [-v_size,v_size,-v_size], [v_size,v_size,-v_size], [0.0,0.0,-1.0]);
                add_face([v_size,-v_size,v_size], [v_size,-v_size,-v_size], [v_size,v_size,-v_size], [v_size,v_size,v_size], [1.0,0.0,0.0]);
                add_face([-v_size,-v_size,-v_size], [-v_size,-v_size,v_size], [-v_size,v_size,v_size], [-v_size,v_size,-v_size], [-1.0,0.0,0.0]);
                add_face([-v_size,v_size,v_size], [v_size,v_size,v_size], [v_size,v_size,-v_size], [-v_size,v_size,-v_size], [0.0,1.0,0.0]);
                add_face([-v_size,-v_size,-v_size], [v_size,-v_size,-v_size], [v_size,-v_size,v_size], [-v_size,-v_size,v_size], [0.0,-1.0,0.0]);
                (v, idx)
            }
        };

        let device = self.device.as_ref().unwrap();
        let vbo = Arc::new(device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&verts), usage: wgpu::BufferUsages::VERTEX }));
        let ibo = Arc::new(device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&indices), usage: wgpu::BufferUsages::INDEX }));
        let index_count = indices.len() as u32;

        let m = crate::executor::MeshBuffers { vbo: vbo.clone(), ibo: ibo.clone(), index_count };
        self.mesh_cache.insert(prim_name.to_string(), m);
        (vbo, ibo, index_count)
    }

    pub fn draw_mesh_immediate(&mut self, prim_name: &str) -> crate::executor::ExecResult {
        use crate::executor::ExecResult;
        self.ensure_canvas_mesh_pipeline();
        if self.canvas_mesh_pipeline.is_none() || self.device.is_none() || self.camera3d_view_proj.is_none() {
            return ExecResult::Fault("Mesh3D requires active RenderCanvas + Camera3D + WGPU device".into());
        }
        let (vbo, ibo, index_count) = self.get_or_create_mesh(prim_name);

        let mut uniforms = MeshUniforms {
            view_proj: self.camera3d_view_proj.unwrap(),
            material: [self.canvas_material[0], self.canvas_material[1], self.canvas_material[2], self.canvas_material[3]],
            pbr: [self.canvas_material[4], self.canvas_material[5], self.canvas_material[6], 0.0],
            camera_pos: [self.camera_pos[0], self.camera_pos[1], self.camera_pos[2], 1.0],
            lights: [PointLightStruct { pos: [0.0;4], color: [0.0;4] }; 4],
        };
        for (i, source) in self.point_lights.iter().take(4).enumerate() {
            uniforms.lights[i] = PointLightStruct { pos: [source.x, source.y, source.z, 1.0], color: [source.r, source.g, source.b, source.intensity] };
        }

        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        if self.mesh_ubo.is_none() {
            self.mesh_ubo = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mesh Uniform Buffer"),
                size: std::mem::size_of::<MeshUniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        let ubo = self.mesh_ubo.as_ref().unwrap();
        queue.write_buffer(ubo, 0, bytemuck::cast_slice(&[uniforms]));

        let bgl = self.canvas_mesh_pipeline.as_ref().unwrap().get_bind_group_layout(0);
        let tex_id = self.canvas_material[6] as i64;
        let (view, sampler) = if tex_id > 0 && (tex_id-1) < self.textures.len() as i64 {
            let (_, v, _, _) = &self.textures[(tex_id-1) as usize]; (v, self.default_sampler.as_ref().unwrap())
        } else { (self.default_texture_view.as_ref().unwrap(), self.default_sampler.as_ref().unwrap()) };

        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: ubo.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(self.default_texture_view.as_ref().unwrap()) },
            ],
            label: None,
        });

        if self.frame_encoder.is_none() {
            self.frame_encoder = Some(device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }));
        }
        let enc = self.frame_encoder.as_mut().unwrap();

        let fb_view = match self.current_canvas_view.as_ref() { Some(v) => v, None => return ExecResult::Fault("No active canvas view".into()) };
        {
            let mut rpass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: fb_view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: self.depth_texture_view.as_ref().map(|dv| wgpu::RenderPassDepthStencilAttachment { view: dv, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }), stencil_ops: None }),
                ..Default::default()
            });
            rpass.set_pipeline(self.canvas_mesh_pipeline.as_ref().unwrap());
            rpass.set_bind_group(0, &bg, &[]);
            rpass.set_vertex_buffer(0, vbo.slice(..));
            
            let singleton = [InstanceData { transform: [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[0.0,0.0,0.0,1.0]], color_offset: [1.0;4], material_pbr: [self.canvas_material[4], self.canvas_material[5], self.canvas_material[6], 0.0] }];
            let inst_vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&singleton), usage: wgpu::BufferUsages::VERTEX });
            rpass.set_vertex_buffer(1, inst_vbo.slice(..));
            rpass.set_index_buffer(ibo.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..index_count, 0, 0..1);
        }
        ExecResult::Value(crate::executor::RelType::Void)
    }

    pub fn present_frame(&mut self) {
        if let (Some(queue), Some(enc)) = (self.queue.as_ref(), self.frame_encoder.take()) {
            queue.submit(std::iter::once(enc.finish()));
        }
        if let Some(frame) = self.current_canvas_frame.take() {
            frame.present();
        }
        self.current_canvas_view = None;
    }
}
