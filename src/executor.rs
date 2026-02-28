use crate::ast::Node;
use crate::natives::NativeModule;
use crate::natives::bridge::{BridgeModule, CoreBridge};
use cgmath::InnerSpace;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RelType {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Array(Vec<RelType>),
    Object(HashMap<String, RelType>),
    // Functions
    FnDef(String, Vec<String>, Box<Node>),
    Call(String, Vec<Node>),

    // I/OParameters, Body Block
    Void,
}

impl std::fmt::Display for RelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelType::Int(v) => write!(f, "{} (i64)", v),
            RelType::Float(v) => write!(f, "{:?} (f64)", v), // Using Debug to avoid dropping .0 on integers formatting like floats
            RelType::Bool(v) => write!(f, "{} (bool)", v),
            RelType::Str(v) => write!(f, "\"{}\" (String)", v),
            RelType::Array(v) => {
                let s: Vec<String> = v.iter().map(|i| i.to_string()).collect();
                write!(f, "[{}] (Array)", s.join(", "))
            }
            RelType::Object(map) => {
                let mut s = Vec::new();
                for (k, v) in map {
                    s.push(format!("{}: {}", k, v));
                }
                write!(f, "{{{}}} (Object)", s.join(", "))
            }
            RelType::FnDef(_, _, _) => write!(f, "<Function>"),
            RelType::Call(_, _) => write!(f, "<Function Call>"),
            RelType::Void => write!(f, "void"),
        }
    }
}

use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;
use winit::event_loop::EventLoop;
use winit::window::Window;

#[derive(Clone, Copy)]
pub struct VoiceState {
    pub active: bool,
    pub freq: f32,
    pub waveform: u8, // 0=Sine, 1=Square, 2=Saw, 3=Tri, 4=Noise
    pub phase: f32,
}

impl Default for VoiceState {
    fn default() -> Self {
        VoiceState {
            active: false,
            freq: 440.0,
            waveform: 0,
            phase: 0.0,
        }
    }
}

pub struct MeshBuffers {
    pub vbo: wgpu::Buffer,
    pub ibo: wgpu::Buffer,
    pub index_count: u32,
}

pub struct StackFrame {
    pub locals: HashMap<String, RelType>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VoxelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VoxelInstance {
    pub instance_pos_and_id: [f32; 4],
}

pub struct ExecutionEngine {
    pub memory: HashMap<String, RelType>,
    pub event_loop: Option<EventLoop<()>>,
    pub window: Option<Arc<Window>>,
    pub surface: Option<wgpu::Surface<'static>>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
    pub config: Option<wgpu::SurfaceConfiguration>,
    pub depth_texture_view: Option<wgpu::TextureView>,
    pub shaders: Vec<wgpu::ShaderModule>,
    pub render_pipelines: HashMap<usize, wgpu::RenderPipeline>,
    pub native_modules: Vec<Box<dyn NativeModule>>,
    pub bridge: Box<dyn BridgeModule>,

    // Voxel Engine (Sprint 12)
    pub camera_active: bool,
    pub camera_pos: [f32; 3],
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_fov: f32,
    pub input_w: bool,
    pub input_a: bool,
    pub input_s: bool,
    pub input_d: bool,
    pub input_space: bool,
    pub input_shift: bool,
    pub input_left_click: bool,

    // Block interactions
    pub interaction_active: bool,
    pub selected_voxel_pos: Option<[i64; 3]>,
    pub place_voxel_pos: Option<[i64; 3]>,

    pub voxel_pipeline: Option<wgpu::RenderPipeline>,
    pub voxel_vbo: Option<wgpu::Buffer>,
    pub voxel_ibo: Option<wgpu::Buffer>,
    pub voxel_instances: Vec<VoxelInstance>,
    pub voxel_bind_group: Option<wgpu::BindGroup>,
    pub voxel_atlas_bind_group: Option<wgpu::BindGroup>,
    pub voxel_ubo: Option<wgpu::Buffer>,
    pub voxel_map: HashMap<[i64; 3], u8>,
    pub voxel_map_active: bool,
    pub voxel_map_dirty: bool,
    pub interaction_enabled: bool,
    pub physics_enabled: bool,
    pub velocity_y: f32,
    pub is_grounded: bool,
    pub voxel_instance_buffer: Option<wgpu::Buffer>,

    // Asset pipeline state
    pub meshes: Vec<MeshBuffers>,
    pub textures: Vec<(
        wgpu::Texture,
        wgpu::TextureView,
        wgpu::BindGroup,
        wgpu::BindGroupLayout,
    )>,

    // UI & Text state
    pub glyph_brush: Option<wgpu_glyph::GlyphBrush<()>>,
    pub staging_belt: Option<wgpu::util::StagingBelt>,
    pub keyboard_buffer: Arc<Mutex<String>>,

    // Modern UI state (egui)
    pub egui_ctx: Option<egui::Context>,
    pub egui_state: Option<egui_winit::State>,
    pub egui_renderer: Option<egui_wgpu::Renderer>,
    pub egui_ui_ptr: Option<*mut egui::Ui>,

    // Audio backend state
    pub voices: Option<Arc<Mutex<[VoiceState; 4]>>>,
    pub stream_samples: Option<Arc<Mutex<Vec<f32>>>>,
    pub stream_pos: Option<Arc<Mutex<usize>>>,
    pub audio_stream: Option<cpal::Stream>,

    // Rodio Audio State
    pub audio_stream_handle: Option<(rodio::OutputStream, rodio::OutputStreamHandle)>,
    pub samples: HashMap<i64, std::sync::Arc<[u8]>>,

    pub call_stack: Vec<StackFrame>,
}

pub enum ExecResult {
    Value(RelType),
    ReturnBlockInfo(RelType), // Explicit return triggered
    Fault(String),
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            memory: HashMap::new(),
            event_loop: None,
            window: None,
            surface: None,
            device: None,
            queue: None,
            config: None,
            depth_texture_view: None,
            shaders: Vec::new(),
            render_pipelines: HashMap::new(),
            native_modules: Vec::new(),
            camera_active: false,
            camera_pos: [0.0, 1.0, 0.0],
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            camera_fov: 75.0,
            input_w: false,
            input_a: false,
            input_s: false,
            input_d: false,
            input_space: false,
            input_shift: false,
            input_left_click: false,
            interaction_active: false,
            selected_voxel_pos: None,
            place_voxel_pos: None,
            voxel_pipeline: None,
            voxel_vbo: None,
            voxel_ibo: None,
            voxel_instances: Vec::new(),
            voxel_bind_group: None,
            voxel_atlas_bind_group: None,
            voxel_ubo: None,
            voxel_map: HashMap::new(),
            voxel_map_active: false,
            voxel_map_dirty: true,
            interaction_enabled: false,
            physics_enabled: false,
            velocity_y: 0.0,
            is_grounded: false,
            voxel_instance_buffer: None,
            meshes: Vec::new(),
            textures: Vec::new(),
            glyph_brush: None,
            staging_belt: None,
            keyboard_buffer: Arc::new(Mutex::new(String::new())),
            egui_ctx: None,
            egui_state: None,
            egui_renderer: None,
            egui_ui_ptr: None,
            voices: None,
            stream_samples: None,
            stream_pos: None,
            audio_stream: None,
            audio_stream_handle: None,
            samples: HashMap::new(),
            call_stack: Vec::new(),
            bridge: Box::new(CoreBridge),
        };

        engine
            .native_modules
            .push(Box::new(crate::natives::math::MathModule));
        engine
            .native_modules
            .push(Box::new(crate::natives::io::IoModule));

        engine
    }

    pub fn ensure_voxel_pipeline(&mut self) {
        if self.voxel_pipeline.is_some() {
            return;
        }
        let (device, config) = if let (Some(d), Some(c)) = (&self.device, &self.config) {
            (d, c)
        } else {
            return;
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../assets/voxel.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("voxel_bind_group_layout"),
        });

        let atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("atlas_bind_group_layout_template"),
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Voxel Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &atlas_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Voxel Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<VoxelVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            wgpu::VertexAttribute {
                                offset: 12,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            wgpu::VertexAttribute {
                                offset: 24,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 3,
                            format: wgpu::VertexFormat::Float32x4,
                        }],
                    },
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
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

        let v: f32 = 0.5;
        let nx = [1.0, 0.0, 0.0];
        let anx = [-1.0, 0.0, 0.0];
        let ny = [0.0, 1.0, 0.0];
        let any = [0.0, -1.0, 0.0];
        let nz = [0.0, 0.0, 1.0];
        let anz = [0.0, 0.0, -1.0];

        let vertices = vec![
            VoxelVertex {
                position: [-v, v, -v],
                normal: ny,
                uv: [0.0, 0.0],
            },
            VoxelVertex {
                position: [v, v, -v],
                normal: ny,
                uv: [1.0, 0.0],
            },
            VoxelVertex {
                position: [v, v, v],
                normal: ny,
                uv: [1.0, 1.0],
            },
            VoxelVertex {
                position: [-v, v, v],
                normal: ny,
                uv: [0.0, 1.0],
            },
            VoxelVertex {
                position: [-v, -v, v],
                normal: any,
                uv: [0.0, 0.0],
            },
            VoxelVertex {
                position: [v, -v, v],
                normal: any,
                uv: [1.0, 0.0],
            },
            VoxelVertex {
                position: [v, -v, -v],
                normal: any,
                uv: [1.0, 1.0],
            },
            VoxelVertex {
                position: [-v, -v, -v],
                normal: any,
                uv: [0.0, 1.0],
            },
            VoxelVertex {
                position: [v, -v, -v],
                normal: nx,
                uv: [0.0, 0.0],
            },
            VoxelVertex {
                position: [v, v, -v],
                normal: nx,
                uv: [1.0, 0.0],
            },
            VoxelVertex {
                position: [v, v, v],
                normal: nx,
                uv: [1.0, 1.0],
            },
            VoxelVertex {
                position: [v, -v, v],
                normal: nx,
                uv: [0.0, 1.0],
            },
            VoxelVertex {
                position: [-v, -v, v],
                normal: anx,
                uv: [0.0, 0.0],
            },
            VoxelVertex {
                position: [-v, v, v],
                normal: anx,
                uv: [1.0, 0.0],
            },
            VoxelVertex {
                position: [-v, v, -v],
                normal: anx,
                uv: [1.0, 1.0],
            },
            VoxelVertex {
                position: [-v, -v, -v],
                normal: anx,
                uv: [0.0, 1.0],
            },
            VoxelVertex {
                position: [-v, -v, v],
                normal: nz,
                uv: [0.0, 0.0],
            },
            VoxelVertex {
                position: [v, -v, v],
                normal: nz,
                uv: [1.0, 0.0],
            },
            VoxelVertex {
                position: [v, v, v],
                normal: nz,
                uv: [1.0, 1.0],
            },
            VoxelVertex {
                position: [-v, v, v],
                normal: nz,
                uv: [0.0, 1.0],
            },
            VoxelVertex {
                position: [v, -v, -v],
                normal: anz,
                uv: [0.0, 0.0],
            },
            VoxelVertex {
                position: [-v, -v, -v],
                normal: anz,
                uv: [1.0, 0.0],
            },
            VoxelVertex {
                position: [-v, v, -v],
                normal: anz,
                uv: [1.0, 1.0],
            },
            VoxelVertex {
                position: [v, v, -v],
                normal: anz,
                uv: [0.0, 1.0],
            },
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16,
            17, 18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
        ];

        let vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube VBO"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let ibo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube IBO"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let c_matrix = [0.0f32; 16 + 4 + 4]; // Matrix (16) + CamPos (3+1pad) + SkyColor (3+1pad)
        let ubo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Voxel Uniform UBO"),
            contents: bytemuck::cast_slice(&c_matrix),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ubo.as_entire_binding(),
            }],
            label: Some("Voxel Bind Group"),
        });

        self.voxel_pipeline = Some(pipeline);
        self.voxel_vbo = Some(vbo);
        self.voxel_ibo = Some(ibo);
        self.voxel_bind_group = Some(bind_group);
        self.voxel_ubo = Some(ubo);
    }

    pub fn execute(&mut self, root: &Node) -> String {
        self.memory.clear();
        let res = self.evaluate(root);

        let mut out = String::new();
        match res {
            ExecResult::Value(val) | ExecResult::ReturnBlockInfo(val) => {
                out.push_str(&format!("Return: {}", val));
            }
            ExecResult::Fault(err) => {
                // Return exactly "Fault: ..." as tests expect it
                return format!("Fault: {}", err);
            }
        }

        if !self.memory.is_empty() {
            let mut keys: Vec<&String> = self.memory.keys().collect();
            // Deterministic state output order is important, albeit tests don't strictly assert the var sequence format,
            // they do exact equality of string matching on simple cases.
            // Better to sort just in case. However, some tests define order implicitly:
            // "Return: 42 (i64), Memory: x = 42, y = 42" implies sequential matching or loose containing.
            // Let's defer sorting and match the specific structure if we can.
            // We'll see how tests fail.
            out.push_str(", Memory: ");

            // To ensure 100% deterministic test behavior, sort variables.
            keys.sort();
            let mem_str: Vec<String> = keys
                .iter()
                .map(|k| {
                    let v = self.memory.get(*k).unwrap();
                    match v {
                        RelType::Str(s) => format!("{} = \"{}\"", k, s),
                        RelType::Float(f) => format!("{} = {:?}", k, f),
                        RelType::Array(_) => format!("{} = [...]", k),
                        RelType::Object(_) => format!("{} = {{...}}", k),
                        RelType::FnDef(_, _, _) => format!("{} = <fn>", k),
                        RelType::Call(_, _) => format!("{} = <fn call>", k),
                        RelType::Void => format!("{} = void", k),
                        _ => format!(
                            "{} = {}",
                            k,
                            match v {
                                RelType::Int(i) => i.to_string(),
                                RelType::Bool(b) => b.to_string(),
                                _ => format!("{}", v),
                            }
                        ),
                    }
                })
                .collect();

            out.push_str(&mem_str.join(", "));
        }

        out
    }

    pub fn get_var(&self, name: &str) -> Option<RelType> {
        // Search Call Stack first (Local Scopes)
        if let Some(frame) = self.call_stack.last()
            && let Some(val) = frame.locals.get(name)
        {
            return Some(val.clone());
        }
        // Fallback to Global Memory
        self.memory.get(name).cloned()
    }

    pub fn set_var(&mut self, name: String, val: RelType) {
        if let Some(frame) = self.call_stack.last_mut() {
            frame.locals.insert(name, val);
        } else {
            self.memory.insert(name, val);
        }
    }

    fn evaluate(&mut self, node: &Node) -> ExecResult {
        match node {
            // Literals
            Node::IntLiteral(v) => ExecResult::Value(RelType::Int(*v)),
            Node::FloatLiteral(v) => ExecResult::Value(RelType::Float(*v)),
            Node::BoolLiteral(v) => ExecResult::Value(RelType::Bool(*v)),
            Node::StringLiteral(v) => ExecResult::Value(RelType::Str(v.clone())),

            // Mem
            Node::Identifier(name) => {
                if let Some(val) = self.get_var(name) {
                    ExecResult::Value(val)
                } else {
                    ExecResult::Fault(format!("Undefined identifier: {}", name))
                }
            }
            Node::Assign(name, expr_node) => {
                let res = self.evaluate(expr_node);
                match res {
                    ExecResult::Value(val) => {
                        self.set_var(name.clone(), val.clone());
                        ExecResult::Value(val)
                    }
                    ExecResult::ReturnBlockInfo(val) => {
                        self.set_var(name.clone(), val.clone());
                        ExecResult::Value(val)
                    }
                    fault => fault,
                }
            }

            // Math
            Node::Add(l, r) => self.do_math(l, r, '+'),
            Node::Sub(l, r) => self.do_math(l, r, '-'),
            Node::Mul(l, r) => self.do_math(l, r, '*'),
            Node::Div(l, r) => self.do_math(l, r, '/'),

            // Math & Time & Matrix
            Node::Sin(n) => match self.evaluate(n) {
                ExecResult::Value(RelType::Float(f)) => ExecResult::Value(RelType::Float(f.sin())),
                ExecResult::Value(RelType::Int(i)) => {
                    ExecResult::Value(RelType::Float((i as f64).sin()))
                }
                fault => fault,
            },
            Node::Cos(n) => match self.evaluate(n) {
                ExecResult::Value(RelType::Float(f)) => ExecResult::Value(RelType::Float(f.cos())),
                ExecResult::Value(RelType::Int(i)) => {
                    ExecResult::Value(RelType::Float((i as f64).cos()))
                }
                fault => fault,
            },
            Node::Time => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let t = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                ExecResult::Value(RelType::Float(t))
            }
            Node::Mat4Mul(l, r) => {
                let lv = self.evaluate(l);
                let rv = self.evaluate(r);
                if let (
                    ExecResult::Value(RelType::Array(l_arr)),
                    ExecResult::Value(RelType::Array(r_arr)),
                ) = (lv, rv)
                {
                    if l_arr.len() == 16 && r_arr.len() == 16 {
                        let mut lm = [0.0f64; 16];
                        let mut rm = [0.0f64; 16];
                        for i in 0..16 {
                            lm[i] = match l_arr[i] {
                                RelType::Float(f) => f,
                                RelType::Int(v) => v as f64,
                                _ => 0.0,
                            };
                            rm[i] = match r_arr[i] {
                                RelType::Float(f) => f,
                                RelType::Int(v) => v as f64,
                                _ => 0.0,
                            };
                        }
                        let mut out = [0.0f64; 16];
                        // Column-major multiplication
                        // C_col_row = sum_i( A_i_row * B_col_i )
                        for col in 0..4 {
                            for row in 0..4 {
                                let mut sum = 0.0;
                                for i in 0..4 {
                                    sum += lm[i * 4 + row] * rm[col * 4 + i];
                                }
                                out[col * 4 + row] = sum;
                            }
                        }
                        let out_arr = out.iter().map(|&f| RelType::Float(f)).collect();
                        ExecResult::Value(RelType::Array(out_arr))
                    } else {
                        ExecResult::Fault("Mat4Mul requires 16-element Float Arrays".to_string())
                    }
                } else {
                    ExecResult::Fault("Mat4Mul requires two Arrays".to_string())
                }
            }

            // Logic
            Node::Eq(l, r) => {
                let lv = self.evaluate(l);
                let rv = self.evaluate(r);
                match (lv, rv) {
                    (
                        ExecResult::Value(RelType::Int(li)),
                        ExecResult::Value(RelType::Float(rf)),
                    ) => ExecResult::Value(RelType::Bool((li as f64) == rf)),
                    (
                        ExecResult::Value(RelType::Float(lf)),
                        ExecResult::Value(RelType::Int(ri)),
                    ) => ExecResult::Value(RelType::Bool(lf == (ri as f64))),
                    (ExecResult::Value(l_val), ExecResult::Value(r_val)) => {
                        ExecResult::Value(RelType::Bool(l_val == r_val))
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid Eq semantics".to_string()),
                }
            }
            // Egui UI (Sprint 10)
            Node::UIWindow(title, body) => {
                let title_val = self.evaluate(title);
                let title_str = match title_val {
                    ExecResult::Value(RelType::Str(s)) => s,
                    _ => "Knoten Window".to_string(),
                };
                if let Some(ctx) = self.egui_ctx.clone() {
                    egui::Window::new(title_str).show(&ctx, |ui| {
                        self.egui_ui_ptr = Some(ui as *mut egui::Ui);
                        self.evaluate(body);
                        self.egui_ui_ptr = None;
                    });
                }
                ExecResult::Value(RelType::Void)
            }
            Node::UILabel(text) => {
                let text_val = self.evaluate(text);
                let text_str = match text_val {
                    ExecResult::Value(RelType::Str(s)) => s,
                    _ => "".to_string(),
                };
                if let Some(ui_ptr) = self.egui_ui_ptr {
                    unsafe {
                        (*ui_ptr).label(text_str);
                    }
                }
                ExecResult::Value(RelType::Void)
            }
            Node::UIButton(text) => {
                let text_val = self.evaluate(text);
                let text_str = match text_val {
                    ExecResult::Value(RelType::Str(s)) => s,
                    _ => "".to_string(),
                };
                let mut clicked = 0;
                if let Some(ui_ptr) = self.egui_ui_ptr {
                    unsafe {
                        if (*ui_ptr).button(text_str).clicked() {
                            clicked = 1;
                        }
                    }
                }
                ExecResult::Value(RelType::Int(clicked))
            }
            Node::UITextInput(var_name_node) => {
                let var_val = self.evaluate(var_name_node);
                if let ExecResult::Value(RelType::Str(var_name)) = var_val {
                    let mut current_text = match self.memory.get(&var_name) {
                        Some(RelType::Str(s)) => s.clone(),
                        _ => String::new(),
                    };
                    if let Some(ui_ptr) = self.egui_ui_ptr {
                        unsafe {
                            if (*ui_ptr).text_edit_singleline(&mut current_text).changed() {
                                self.memory
                                    .insert(var_name.clone(), RelType::Str(current_text));
                            }
                        }
                    }
                }
                ExecResult::Value(RelType::Void)
            }
            Node::Lt(l, r) => {
                let lv = self.evaluate(l);
                let rv = self.evaluate(r);
                match (lv, rv) {
                    (ExecResult::Value(RelType::Int(li)), ExecResult::Value(RelType::Int(ri))) => {
                        ExecResult::Value(RelType::Bool(li < ri))
                    }
                    (
                        ExecResult::Value(RelType::Float(lf)),
                        ExecResult::Value(RelType::Float(rf)),
                    ) => ExecResult::Value(RelType::Bool(lf < rf)),
                    (
                        ExecResult::Value(RelType::Int(li)),
                        ExecResult::Value(RelType::Float(rf)),
                    ) => ExecResult::Value(RelType::Bool((li as f64) < rf)),
                    (
                        ExecResult::Value(RelType::Float(lf)),
                        ExecResult::Value(RelType::Int(ri)),
                    ) => ExecResult::Value(RelType::Bool(lf < (ri as f64))),
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid Lt semantics".to_string()),
                }
            }

            // Objects
            Node::ObjectLiteral(map) => {
                let mut obj = HashMap::new();
                for (k, v) in map {
                    match self.evaluate(v) {
                        ExecResult::Value(val) => {
                            obj.insert(k.clone(), val);
                        }
                        fault => return fault,
                    }
                }
                ExecResult::Value(RelType::Object(obj))
            }
            Node::PropertyGet(obj_node, prop_name) => match self.evaluate(obj_node) {
                ExecResult::Value(RelType::Object(obj)) => {
                    if let Some(val) = obj.get(prop_name) {
                        ExecResult::Value(val.clone())
                    } else {
                        ExecResult::Fault(format!("Property '{}' not found on Object", prop_name))
                    }
                }
                ExecResult::Fault(err) => ExecResult::Fault(err),
                _ => ExecResult::Fault("PropertyGet on non-object".to_string()),
            },
            Node::PropertySet(obj_node, prop_name, val_node) => {
                let val = match self.evaluate(val_node) {
                    ExecResult::Value(v) => v,
                    fault => return fault,
                };

                // Because Knoten variables are resolved by returning their values, mutating an object requires we look up its name.
                // For PropertySet to actually mutate memory, it must know WHERE the object is.
                // To keep this pure MVP sprint simple, if `obj_node` is an Identifier, we mutate memory directly.
                match &**obj_node {
                    Node::Identifier(var_name) => {
                        let mut current_obj = match self.get_var(var_name) {
                            Some(RelType::Object(o)) => o,
                            _ => {
                                return ExecResult::Fault(format!(
                                    "Identifier '{}' is not an object",
                                    var_name
                                ));
                            }
                        };
                        current_obj.insert(prop_name.clone(), val.clone());
                        self.set_var(var_name.clone(), RelType::Object(current_obj));
                        ExecResult::Value(val)
                    }
                    _ => ExecResult::Fault(
                        "PropertySet must target an Identifier for now".to_string(),
                    ),
                }
            }

            // Arrays & Strings
            Node::ArrayLiteral(nodes) => {
                let mut vals = Vec::new();
                for item in nodes {
                    match self.evaluate(item) {
                        ExecResult::Value(v) => vals.push(v),
                        fault => return fault,
                    }
                }
                ExecResult::Value(RelType::Array(vals))
            }
            Node::ArrayGet(var_name, index_node) => {
                let val = match self.get_var(var_name) {
                    Some(v) => v,
                    None => {
                        return ExecResult::Fault(format!(
                            "Undefined array variable '{}'",
                            var_name
                        ));
                    }
                };
                if let RelType::Array(arr) = val {
                    match self.evaluate(index_node) {
                        ExecResult::Value(RelType::Int(idx)) => {
                            if idx >= 0 && (idx as usize) < arr.len() {
                                ExecResult::Value(arr[idx as usize].clone())
                            } else {
                                ExecResult::Fault(format!(
                                    "Array index {} out of bounds for '{}'",
                                    idx, var_name
                                ))
                            }
                        }
                        ExecResult::Value(_) => {
                            ExecResult::Fault("Array index must be an Integer".to_string())
                        }
                        fault => fault,
                    }
                } else {
                    ExecResult::Fault(format!("Variable '{}' is not an array", var_name))
                }
            }
            Node::ArraySet(var_name, index_node, val_node) => {
                let val = match self.get_var(var_name) {
                    Some(v) => v,
                    None => {
                        return ExecResult::Fault(format!(
                            "Undefined array variable '{}'",
                            var_name
                        ));
                    }
                };
                if let RelType::Array(mut arr) = val {
                    let idx_res = self.evaluate(index_node);
                    let val_res = self.evaluate(val_node);
                    match (idx_res, val_res) {
                        (ExecResult::Value(RelType::Int(idx)), ExecResult::Value(new_val)) => {
                            if idx >= 0 && (idx as usize) < arr.len() {
                                arr[idx as usize] = new_val;
                                self.set_var(var_name.clone(), RelType::Array(arr));
                                ExecResult::Value(RelType::Void)
                            } else {
                                ExecResult::Fault(format!(
                                    "Array index {} out of bounds for '{}'",
                                    idx, var_name
                                ))
                            }
                        }
                        (ExecResult::Value(_), ExecResult::Value(_)) => {
                            ExecResult::Fault("Array index must be an Integer".to_string())
                        }
                        (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                            ExecResult::Fault(err)
                        }
                        (ExecResult::ReturnBlockInfo(v), _)
                        | (_, ExecResult::ReturnBlockInfo(v)) => ExecResult::ReturnBlockInfo(v),
                    }
                } else {
                    ExecResult::Fault(format!("Variable '{}' is not an array", var_name))
                }
            }
            Node::ArrayPush(var_name, val_node) => {
                let val = match self.get_var(var_name) {
                    Some(v) => v,
                    None => {
                        return ExecResult::Fault(format!(
                            "Undefined array variable '{}'",
                            var_name
                        ));
                    }
                };
                if let RelType::Array(mut arr) = val {
                    match self.evaluate(val_node) {
                        ExecResult::Value(new_val) => {
                            arr.push(new_val);
                            self.set_var(var_name.clone(), RelType::Array(arr));
                            ExecResult::Value(RelType::Void)
                        }
                        fault => fault,
                    }
                } else {
                    ExecResult::Fault(format!("Variable '{}' is not an array", var_name))
                }
            }
            Node::ArrayLen(var_name) => {
                let val = match self.get_var(var_name) {
                    Some(v) => v,
                    None => {
                        return ExecResult::Fault(format!(
                            "Undefined array variable '{}'",
                            var_name
                        ));
                    }
                };
                match val {
                    RelType::Array(arr) => ExecResult::Value(RelType::Int(arr.len() as i64)),
                    RelType::Str(s) => ExecResult::Value(RelType::Int(s.len() as i64)),
                    _ => ExecResult::Fault(format!("Variable '{}' has no length", var_name)),
                }
            }
            Node::Index(container, index) => {
                let cv = self.evaluate(container);
                let iv = self.evaluate(index);
                match (cv, iv) {
                    (
                        ExecResult::Value(RelType::Array(arr)),
                        ExecResult::Value(RelType::Int(idx)),
                    ) => {
                        if idx >= 0 && (idx as usize) < arr.len() {
                            ExecResult::Value(arr[idx as usize].clone())
                        } else {
                            ExecResult::Fault("Index out of bounds".to_string())
                        }
                    }
                    (ExecResult::Value(RelType::Str(s)), ExecResult::Value(RelType::Int(idx))) => {
                        if idx >= 0 && (idx as usize) < s.len() {
                            let ch = s.chars().nth(idx as usize).unwrap();
                            ExecResult::Value(RelType::Str(ch.to_string()))
                        } else {
                            ExecResult::Fault("Index out of bounds".to_string())
                        }
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid Index semantics".to_string()),
                }
            }
            Node::Concat(l, r) => {
                let lv = self.evaluate(l);
                let rv = self.evaluate(r);
                match (lv, rv) {
                    (ExecResult::Value(RelType::Str(ls)), ExecResult::Value(RelType::Str(rs))) => {
                        ExecResult::Value(RelType::Str(ls + &rs))
                    }
                    (
                        ExecResult::Value(RelType::Array(mut la)),
                        ExecResult::Value(RelType::Array(ra)),
                    ) => {
                        la.extend(ra);
                        ExecResult::Value(RelType::Array(la))
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid Concat semantics".to_string()),
                }
            }

            // Bitwise
            Node::BitAnd(l, r) => {
                let lv = self.evaluate(l);
                let rv = self.evaluate(r);
                match (lv, rv) {
                    (ExecResult::Value(RelType::Int(li)), ExecResult::Value(RelType::Int(ri))) => {
                        ExecResult::Value(RelType::Int(li & ri))
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid BitAnd semantics".to_string()),
                }
            }
            Node::BitShiftLeft(l, r) => {
                let lv = self.evaluate(l);
                let rv = self.evaluate(r);
                match (lv, rv) {
                    (ExecResult::Value(RelType::Int(li)), ExecResult::Value(RelType::Int(ri))) => {
                        ExecResult::Value(RelType::Int(li << ri))
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid BitShiftLeft semantics".to_string()),
                }
            }
            Node::BitShiftRight(l, r) => {
                let lv = self.evaluate(l);
                let rv = self.evaluate(r);
                match (lv, rv) {
                    (ExecResult::Value(RelType::Int(li)), ExecResult::Value(RelType::Int(ri))) => {
                        ExecResult::Value(RelType::Int(li >> ri))
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid BitShiftRight semantics".to_string()),
                }
            }

            // Functions
            Node::FnDef(name, params, body) => {
                let func = RelType::FnDef(name.clone(), params.clone(), body.clone());
                self.memory.insert(name.clone(), func.clone());
                ExecResult::Value(func)
            }
            Node::Call(name, args) => {
                let func_val = match self.memory.get(name) {
                    Some(val) => val.clone(),
                    None => return ExecResult::Fault(format!("Undefined function '{}'", name)),
                };

                match func_val {
                    RelType::FnDef(_, params, body) => {
                        if args.len() != params.len() {
                            return ExecResult::Fault(format!(
                                "Argument count mismatch for function '{}': expected {}, got {}",
                                name,
                                params.len(),
                                args.len()
                            ));
                        }

                        let mut evaluated_args = Vec::new();
                        for arg in args {
                            match self.evaluate(arg) {
                                ExecResult::Value(v) => evaluated_args.push(v),
                                ExecResult::ReturnBlockInfo(v) => evaluated_args.push(v),
                                fault => return fault,
                            }
                        }

                        // Create new Stack Frame
                        let mut frame = StackFrame {
                            locals: HashMap::new(),
                        };
                        for (i, p) in params.iter().enumerate() {
                            frame.locals.insert(p.clone(), evaluated_args[i].clone());
                        }

                        // Push and Execute
                        self.call_stack.push(frame);
                        let mut call_res = self.evaluate(&body);
                        self.call_stack.pop(); // Pop scope

                        // Unwrap Return value if applicable
                        if let ExecResult::ReturnBlockInfo(v) = call_res {
                            call_res = ExecResult::Value(v);
                        }

                        call_res
                    }
                    _ => ExecResult::Fault(format!("Identifier '{}' is not a function", name)),
                }
            }
            Node::NativeCall(func_name, args) => {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    match self.evaluate(arg) {
                        ExecResult::Value(v) => evaluated_args.push(v),
                        fault => return fault,
                    }
                }

                for module in &self.native_modules {
                    if let Some(res) = module.handle(func_name, &evaluated_args) {
                        return res;
                    }
                }
                ExecResult::Fault(format!("Unknown native function '{}'", func_name))
            }
            Node::ExternCall {
                module,
                function,
                args,
            } => {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    match self.evaluate(arg) {
                        ExecResult::Value(v) => evaluated_args.push(v),
                        fault => return fault,
                    }
                }

                if let Some(res) = self.bridge.handle(module, function, &evaluated_args) {
                    return res;
                }

                // Future FFI gateway fallback
                ExecResult::Fault(format!(
                    "ExternCall mapped to foreign {}::{} - FFI Binding Not Found",
                    module, function
                ))
            }
            // I/O
            Node::FileRead(path_node) => match self.evaluate(path_node) {
                ExecResult::Value(RelType::Str(path)) => match std::fs::read(&path) {
                    Ok(bytes) => {
                        let arr = bytes.into_iter().map(|b| RelType::Int(b as i64)).collect();
                        ExecResult::Value(RelType::Array(arr))
                    }
                    Err(e) => ExecResult::Fault(format!("FileRead error: {}", e)),
                },
                ExecResult::Fault(err) => ExecResult::Fault(err),
                _ => ExecResult::Fault("FileRead semantic error: path not a string".to_string()),
            },
            Node::FileWrite(path_node, data_node) => {
                let p_val = self.evaluate(path_node);
                let d_val = self.evaluate(data_node);
                match (p_val, d_val) {
                    (
                        ExecResult::Value(RelType::Str(path)),
                        ExecResult::Value(RelType::Array(arr)),
                    ) => {
                        let mut bytes = Vec::new();
                        for item in arr {
                            if let RelType::Int(b) = item {
                                bytes.push(b as u8);
                            } else {
                                return ExecResult::Fault(
                                    "FileWrite error: data array contains non-integer".to_string(),
                                );
                            }
                        }
                        if let Err(e) = std::fs::write(&path, bytes) {
                            return ExecResult::Fault(format!("FileWrite error: {}", e));
                        }
                        ExecResult::Value(RelType::Void)
                    }
                    (ExecResult::Value(RelType::Str(path)), ExecResult::Value(RelType::Str(s))) => {
                        if let Err(e) = std::fs::write(&path, s.as_bytes()) {
                            return ExecResult::Fault(format!("FileWrite error: {}", e));
                        }
                        ExecResult::Value(RelType::Void)
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("FileWrite semantic error".to_string()),
                }
            }
            Node::Print(n) => {
                let val = self.evaluate(n);
                match val {
                    ExecResult::Value(v) => {
                        println!("{}", v);
                        ExecResult::Value(v)
                    }
                    fault => fault,
                }
            }

            // FFI / Reflection
            Node::EvalJSONNative(json_node) => match self.evaluate(json_node) {
                ExecResult::Value(RelType::Str(json)) => {
                    match serde_json::from_str::<Node>(&json) {
                        Ok(parsed) => {
                            let mut sub_engine = ExecutionEngine::new();
                            let output = sub_engine.execute(&parsed);
                            ExecResult::Value(RelType::Str(output))
                        }
                        Err(e) => ExecResult::Fault(format!("JSON Native Eval Fault: {}", e)),
                    }
                }
                fault => fault,
            },
            Node::ToString(n) => {
                match self.evaluate(n) {
                    ExecResult::Value(v) => {
                        let s = format!("{}", v);
                        // Clean up type signatures "42 (i64)" -> "42" so it can be combined easily
                        // Wait, no. We just use standard format. If it matches test output needs, it shouldn't have signatures.
                        // Actually, our RelType::Display has signatures. The evaluator output string matches Display.
                        // For building arbitrary strings to file we might need raw conversions, but we just want Display format for tests.
                        ExecResult::Value(RelType::Str(s))
                    }
                    fault => fault,
                }
            }
            Node::Import(path) => match std::fs::read_to_string(path) {
                Ok(json) => match serde_json::from_str::<Node>(&json) {
                    Ok(parsed) => self.evaluate(&parsed),
                    Err(e) => {
                        ExecResult::Fault(format!("Import JSON Parse Fault ({}): {}", path, e))
                    }
                },
                Err(e) => ExecResult::Fault(format!("Import File Read Fault ({}): {}", path, e)),
            },

            // 3D Graphics (WGPU FFI)
            Node::InitWindow(w_node, h_node, t_node) => {
                let w_val = self.evaluate(w_node);
                let h_val = self.evaluate(h_node);
                let t_val = self.evaluate(t_node);
                if let (
                    ExecResult::Value(RelType::Int(w)),
                    ExecResult::Value(RelType::Int(h)),
                    ExecResult::Value(RelType::Str(t)),
                ) = (w_val, h_val, t_val)
                {
                    use winit::application::ApplicationHandler;
                    use winit::platform::pump_events::EventLoopExtPumpEvents;

                    struct WindowPump {
                        window: Option<Arc<Window>>,
                        width: i32,
                        height: i32,
                        title: String,
                    }

                    impl ApplicationHandler for WindowPump {
                        fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
                            if self.window.is_none() {
                                let attrs = winit::window::Window::default_attributes()
                                    .with_inner_size(winit::dpi::LogicalSize::new(
                                        self.width as f64,
                                        self.height as f64,
                                    ))
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

                    let mut event_loop = EventLoop::new().unwrap();
                    let mut pump = WindowPump {
                        window: None,
                        width: w as i32,
                        height: h as i32,
                        title: t,
                    };
                    // pump events once to trigger resumed and create the window
                    let _ = event_loop
                        .pump_app_events(Some(std::time::Duration::from_millis(50)), &mut pump);

                    if pump.window.is_none() {
                        return ExecResult::Fault(
                            "InitWindow failed to create window via resumed()".to_string(),
                        );
                    }
                    self.window = pump.window;
                    self.event_loop = Some(event_loop);
                    ExecResult::Value(RelType::Void)
                } else {
                    ExecResult::Fault("InitWindow expects (Int, Int, String)".to_string())
                }
            }
            Node::InitGraphics => {
                if let Some(window) = &self.window {
                    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
                    let w_ptr = Arc::clone(window);

                    let surface = instance.create_surface(w_ptr.clone()).unwrap();
                    let adapter = pollster::block_on(instance.request_adapter(
                        &wgpu::RequestAdapterOptions {
                            power_preference: wgpu::PowerPreference::default(),
                            compatible_surface: Some(&surface),
                            force_fallback_adapter: false,
                        },
                    ))
                    .unwrap();

                    let (device, queue) = pollster::block_on(
                        adapter.request_device(
                            &wgpu::DeviceDescriptor {
                                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                                    .using_resolution(adapter.limits()),
                                ..Default::default()
                            },
                            None,
                        ),
                    )
                    .unwrap();
                    let size = window.inner_size();
                    let caps = surface.get_capabilities(&adapter);
                    let config = wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: caps.formats[0],
                        width: size.width.max(1),
                        height: size.height.max(1),
                        present_mode: wgpu::PresentMode::Fifo,
                        alpha_mode: caps.alpha_modes[0],
                        view_formats: vec![],
                        desired_maximum_frame_latency: 2,
                    };
                    surface.configure(&device, &config);

                    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Depth Texture"),
                        size: wgpu::Extent3d {
                            width: config.width,
                            height: config.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Depth32Float,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    });
                    self.depth_texture_view =
                        Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));

                    let static_surface = unsafe {
                        std::mem::transmute::<wgpu::Surface<'_>, wgpu::Surface<'static>>(surface)
                    };
                    self.surface = Some(static_surface);

                    let ctx = egui::Context::default();
                    let viewport_id = egui::ViewportId::ROOT;
                    let egui_state = egui_winit::State::new(
                        ctx.clone(),
                        viewport_id,
                        window.as_ref(),
                        Some(window.scale_factor() as f32),
                        None,
                        Some(2 * 1024 * 1024),
                    );
                    let egui_renderer =
                        egui_wgpu::Renderer::new(&device, config.format, None, 1, false);
                    self.egui_ctx = Some(ctx);
                    self.egui_state = Some(egui_state);
                    self.egui_renderer = Some(egui_renderer);

                    self.device = Some(device);
                    self.queue = Some(queue);
                    self.config = Some(config);
                    ExecResult::Value(RelType::Void)
                } else {
                    ExecResult::Fault("InitGraphics requires InitWindow first".to_string())
                }
            }
            Node::LoadShader(code_node) => {
                if let ExecResult::Value(RelType::Str(code)) = self.evaluate(code_node) {
                    if let Some(device) = &self.device {
                        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("KnotenShader"),
                            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(code)),
                        });
                        let id = self.shaders.len();
                        self.shaders.push(shader);
                        ExecResult::Value(RelType::Int(id as i64))
                    } else {
                        ExecResult::Fault("LoadShader requires InitGraphics".to_string())
                    }
                } else {
                    ExecResult::Fault("LoadShader expects String".to_string())
                }
            }
            Node::RenderMesh(shader_id_node, verts_node, uniform_node) => {
                let shader_val = self.evaluate(shader_id_node);
                let _verts_val = self.evaluate(verts_node);
                let uniform_val = self.evaluate(uniform_node);

                if let ExecResult::Value(RelType::Int(s_id)) = shader_val {
                    if let (Some(device), Some(queue), Some(surface), Some(config)) =
                        (&self.device, &self.queue, &self.surface, &self.config)
                    {
                        let shader = &self.shaders[s_id as usize];

                        let bind_group_layout =
                            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                                entries: &[wgpu::BindGroupLayoutEntry {
                                    binding: 0,
                                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                    ty: wgpu::BindingType::Buffer {
                                        ty: wgpu::BufferBindingType::Uniform,
                                        has_dynamic_offset: false,
                                        min_binding_size: None,
                                    },
                                    count: None,
                                }],
                                label: Some("uniform_bind_group_layout"),
                            });

                        let pipeline_layout =
                            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                                label: None,
                                bind_group_layouts: &[&bind_group_layout],
                                push_constant_ranges: &[],
                            });

                        let pipeline =
                            self.render_pipelines
                                .entry(s_id as usize)
                                .or_insert_with(|| {
                                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                                        label: Some("Demo Pipeline"),
                                        layout: Some(&pipeline_layout),
                                        vertex: wgpu::VertexState {
                                            module: shader,
                                            entry_point: Some("vs_main"),
                                            buffers: &[],
                                            compilation_options:
                                                wgpu::PipelineCompilationOptions::default(),
                                        },
                                        fragment: Some(wgpu::FragmentState {
                                            module: shader,
                                            entry_point: Some("fs_main"),
                                            targets: &[Some(wgpu::ColorTargetState {
                                                format: config.format,
                                                blend: Some(wgpu::BlendState::REPLACE),
                                                write_mask: wgpu::ColorWrites::ALL,
                                            })],
                                            compilation_options:
                                                wgpu::PipelineCompilationOptions::default(),
                                        }),
                                        primitive: wgpu::PrimitiveState::default(),
                                        depth_stencil: None,
                                        multisample: wgpu::MultisampleState::default(),
                                        multiview: None,
                                        cache: None,
                                    })
                                });

                        let mut active_bind_group = None;

                        // Parse uniforms
                        if let ExecResult::Value(RelType::Array(arr)) = uniform_val {
                            let floats: Vec<f32> = arr
                                .into_iter()
                                .map(|v| match v {
                                    RelType::Float(f) => f as f32,
                                    RelType::Int(i) => i as f32,
                                    _ => 0.0,
                                })
                                .collect();

                            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                                label: Some("Uniform Buffer"),
                                size: (floats.len() * 4).max(64) as u64,
                                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                                mapped_at_creation: false,
                            });
                            queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&floats));

                            active_bind_group =
                                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                                    layout: &bind_group_layout,
                                    entries: &[wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: buffer.as_entire_binding(),
                                    }],
                                    label: Some("uniform_bind_group"),
                                }));
                        }

                        match surface.get_current_texture() {
                            Ok(frame) => {
                                let view = frame
                                    .texture
                                    .create_view(&wgpu::TextureViewDescriptor::default());
                                let mut encoder = device.create_command_encoder(
                                    &wgpu::CommandEncoderDescriptor::default(),
                                );
                                {
                                    let mut rpass =
                                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                            label: Some("Render Pass"),
                                            color_attachments: &[Some(
                                                wgpu::RenderPassColorAttachment {
                                                    view: &view,
                                                    resolve_target: None,
                                                    ops: wgpu::Operations {
                                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                                            r: 0.1,
                                                            g: 0.2,
                                                            b: 0.3,
                                                            a: 1.0,
                                                        }),
                                                        store: wgpu::StoreOp::Store,
                                                    },
                                                },
                                            )],
                                            depth_stencil_attachment: None,
                                            timestamp_writes: None,
                                            occlusion_query_set: None,
                                        });
                                    rpass.set_pipeline(pipeline);
                                    if let Some(bg) = &active_bind_group {
                                        rpass.set_bind_group(0, bg, &[]);
                                    }
                                    rpass.draw(0..36, 0..1); // 36 vertices handles cubes natively!
                                }
                                queue.submit(Some(encoder.finish()));
                                frame.present();
                                ExecResult::Value(RelType::Void)
                            }
                            Err(e) => ExecResult::Fault(format!(
                                "RenderMesh failed to acquire frame: {:?}",
                                e
                            )),
                        }
                    } else {
                        ExecResult::Fault("Graphics context not initialized".to_string())
                    }
                } else {
                    ExecResult::Fault("RenderMesh expects (Int, Array, Array)".to_string())
                }
            }
            Node::LoadMesh(path_node) => {
                if let ExecResult::Value(RelType::Str(path)) = self.evaluate(path_node) {
                    if let Some(device) = &self.device {
                        let obj = tobj::load_obj(
                            &path,
                            &tobj::LoadOptions {
                                triangulate: true,
                                single_index: true,
                                ..Default::default()
                            },
                        );
                        match obj {
                            Ok((models, _)) => {
                                let mesh = &models[0].mesh;

                                #[repr(C)]
                                #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
                                struct Vertex {
                                    position: [f32; 3],
                                    tex_coords: [f32; 2],
                                    normal: [f32; 3],
                                }

                                let num_vertices = mesh.positions.len() / 3;
                                let mut vertices = Vec::new();
                                for i in 0..num_vertices {
                                    let px = mesh.positions[i * 3];
                                    let py = mesh.positions[i * 3 + 1];
                                    let pz = mesh.positions[i * 3 + 2];
                                    let u = if mesh.texcoords.len() > i * 2 {
                                        mesh.texcoords[i * 2]
                                    } else {
                                        0.0
                                    };
                                    let v = if mesh.texcoords.len() > i * 2 + 1 {
                                        mesh.texcoords[i * 2 + 1]
                                    } else {
                                        0.0
                                    };
                                    let nx = if mesh.normals.len() > i * 3 {
                                        mesh.normals[i * 3]
                                    } else {
                                        0.0
                                    };
                                    let ny = if mesh.normals.len() > i * 3 + 1 {
                                        mesh.normals[i * 3 + 1]
                                    } else {
                                        0.0
                                    };
                                    let nz = if mesh.normals.len() > i * 3 + 2 {
                                        mesh.normals[i * 3 + 2]
                                    } else {
                                        0.0
                                    };
                                    vertices.push(Vertex {
                                        position: [px, py, pz],
                                        tex_coords: [u, v],
                                        normal: [nx, ny, nz],
                                    });
                                }

                                let vbo =
                                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                        label: Some("Mesh VBO"),
                                        contents: bytemuck::cast_slice(&vertices),
                                        usage: wgpu::BufferUsages::VERTEX,
                                    });
                                let ibo =
                                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                        label: Some("Mesh IBO"),
                                        contents: bytemuck::cast_slice(&mesh.indices),
                                        usage: wgpu::BufferUsages::INDEX,
                                    });
                                let id = self.meshes.len();
                                self.meshes.push(MeshBuffers {
                                    vbo,
                                    ibo,
                                    index_count: mesh.indices.len() as u32,
                                });
                                ExecResult::Value(RelType::Int(id as i64))
                            }
                            Err(e) => ExecResult::Fault(format!("LoadMesh failed: {}", e)),
                        }
                    } else {
                        ExecResult::Fault("LoadMesh requires InitGraphics".to_string())
                    }
                } else {
                    ExecResult::Fault("LoadMesh expects String path".to_string())
                }
            }
            Node::LoadTexture(path_node) => {
                if let ExecResult::Value(RelType::Str(path)) = self.evaluate(path_node) {
                    if let (Some(device), Some(queue)) = (&self.device, &self.queue) {
                        match image::open(&path) {
                            Ok(img_dyn) => {
                                let img = img_dyn.into_rgba8();
                                let dimensions = img.dimensions();
                                let texture_size = wgpu::Extent3d {
                                    width: dimensions.0,
                                    height: dimensions.1,
                                    depth_or_array_layers: 1,
                                };
                                let texture = device.create_texture(&wgpu::TextureDescriptor {
                                    label: Some("Texture"),
                                    size: texture_size,
                                    mip_level_count: 1,
                                    sample_count: 1,
                                    dimension: wgpu::TextureDimension::D2,
                                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                                        | wgpu::TextureUsages::COPY_DST,
                                    view_formats: &[],
                                });
                                queue.write_texture(
                                    wgpu::ImageCopyTexture {
                                        texture: &texture,
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
                                let view =
                                    texture.create_view(&wgpu::TextureViewDescriptor::default());
                                let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                                    mag_filter: wgpu::FilterMode::Linear,
                                    min_filter: wgpu::FilterMode::Linear,
                                    mipmap_filter: wgpu::FilterMode::Linear,
                                    ..Default::default()
                                });

                                let bind_group_layout = device.create_bind_group_layout(
                                    &wgpu::BindGroupLayoutDescriptor {
                                        entries: &[
                                            wgpu::BindGroupLayoutEntry {
                                                binding: 0,
                                                visibility: wgpu::ShaderStages::FRAGMENT,
                                                ty: wgpu::BindingType::Texture {
                                                    multisampled: false,
                                                    view_dimension: wgpu::TextureViewDimension::D2,
                                                    sample_type: wgpu::TextureSampleType::Float {
                                                        filterable: true,
                                                    },
                                                },
                                                count: None,
                                            },
                                            wgpu::BindGroupLayoutEntry {
                                                binding: 1,
                                                visibility: wgpu::ShaderStages::FRAGMENT,
                                                ty: wgpu::BindingType::Sampler(
                                                    wgpu::SamplerBindingType::Filtering,
                                                ),
                                                count: None,
                                            },
                                        ],
                                        label: Some("texture_bind_group_layout"),
                                    },
                                );

                                let bind_group =
                                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                                        layout: &bind_group_layout,
                                        entries: &[
                                            wgpu::BindGroupEntry {
                                                binding: 0,
                                                resource: wgpu::BindingResource::TextureView(&view),
                                            },
                                            wgpu::BindGroupEntry {
                                                binding: 1,
                                                resource: wgpu::BindingResource::Sampler(&sampler),
                                            },
                                        ],
                                        label: Some("diffuse_bind_group"),
                                    });

                                let id = self.textures.len();
                                self.textures
                                    .push((texture, view, bind_group, bind_group_layout));
                                ExecResult::Value(RelType::Int(id as i64))
                            }
                            Err(e) => ExecResult::Fault(format!("LoadTexture failed: {}", e)),
                        }
                    } else {
                        ExecResult::Fault("LoadTexture requires InitGraphics".to_string())
                    }
                } else {
                    ExecResult::Fault("LoadTexture expects String path".to_string())
                }
            }
            Node::LoadFont(path_node) => {
                if let ExecResult::Value(RelType::Str(path)) = self.evaluate(path_node) {
                    if let (Some(device), Some(config)) = (&self.device, &self.config) {
                        match std::fs::read(&path) {
                            Ok(bytes) => {
                                let font =
                                    wgpu_glyph::ab_glyph::FontArc::try_from_vec(bytes).unwrap();
                                let brush = wgpu_glyph::GlyphBrushBuilder::using_font(font)
                                    .build(device, config.format);
                                self.glyph_brush = Some(brush);
                                self.staging_belt = Some(wgpu::util::StagingBelt::new(1024));
                                ExecResult::Value(RelType::Void)
                            }
                            Err(e) => ExecResult::Fault(format!("LoadFont Failed: {}", e)),
                        }
                    } else {
                        ExecResult::Fault("LoadFont requires InitGraphics".to_string())
                    }
                } else {
                    ExecResult::Fault("LoadFont expects String path".to_string())
                }
            }
            Node::DrawText(text_n, x_n, y_n, size_n, color_n) => {
                let text_val = self.evaluate(text_n);
                let x_val = self.evaluate(x_n);
                let y_val = self.evaluate(y_n);
                let size_val = self.evaluate(size_n);
                let color_val = self.evaluate(color_n);

                if let (
                    ExecResult::Value(RelType::Str(text)),
                    ExecResult::Value(RelType::Float(x)),
                    ExecResult::Value(RelType::Float(y)),
                    ExecResult::Value(RelType::Float(size)),
                    ExecResult::Value(RelType::Array(color_arr)),
                ) = (text_val, x_val, y_val, size_val, color_val)
                {
                    if let (
                        Some(device),
                        Some(queue),
                        Some(surface),
                        Some(config),
                        Some(glyph_brush),
                        Some(staging_belt),
                    ) = (
                        &self.device,
                        &self.queue,
                        &self.surface,
                        &self.config,
                        &mut self.glyph_brush,
                        &mut self.staging_belt,
                    ) {
                        let c = [
                            match &color_arr.first() {
                                Some(RelType::Float(f)) => *f as f32,
                                Some(RelType::Int(i)) => *i as f32,
                                _ => 0.0,
                            },
                            match &color_arr.get(1) {
                                Some(RelType::Float(f)) => *f as f32,
                                Some(RelType::Int(i)) => *i as f32,
                                _ => 0.0,
                            },
                            match &color_arr.get(2) {
                                Some(RelType::Float(f)) => *f as f32,
                                Some(RelType::Int(i)) => *i as f32,
                                _ => 0.0,
                            },
                            match &color_arr.get(3) {
                                Some(RelType::Float(f)) => *f as f32,
                                Some(RelType::Int(i)) => *i as f32,
                                _ => 1.0,
                            },
                        ];
                        glyph_brush.queue(wgpu_glyph::Section {
                            screen_position: (x as f32, y as f32),
                            bounds: (config.width as f32, config.height as f32),
                            text: vec![
                                wgpu_glyph::Text::new(&text)
                                    .with_color(c)
                                    .with_scale(size as f32),
                            ],
                            ..wgpu_glyph::Section::default()
                        });

                        match surface.get_current_texture() {
                            Ok(frame) => {
                                let view = frame
                                    .texture
                                    .create_view(&wgpu::TextureViewDescriptor::default());
                                let mut encoder = device.create_command_encoder(
                                    &wgpu::CommandEncoderDescriptor::default(),
                                );
                                {
                                    let _rpass =
                                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                            label: Some("DrawText Pass"),
                                            color_attachments: &[Some(
                                                wgpu::RenderPassColorAttachment {
                                                    view: &view,
                                                    resolve_target: None,
                                                    ops: wgpu::Operations {
                                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                                            r: 0.1,
                                                            g: 0.1,
                                                            b: 0.1,
                                                            a: 1.0,
                                                        }),
                                                        store: wgpu::StoreOp::Store,
                                                    },
                                                },
                                            )],
                                            depth_stencil_attachment: None,
                                            timestamp_writes: None,
                                            occlusion_query_set: None,
                                        });
                                }
                                glyph_brush
                                    .draw_queued(
                                        device,
                                        staging_belt,
                                        &mut encoder,
                                        &view,
                                        config.width,
                                        config.height,
                                    )
                                    .unwrap();
                                staging_belt.finish();
                                queue.submit(Some(encoder.finish()));
                                frame.present();
                                staging_belt.recall();
                                ExecResult::Value(RelType::Void)
                            }
                            Err(e) => ExecResult::Fault(format!("DrawText surface error: {:?}", e)),
                        }
                    } else {
                        ExecResult::Fault("DrawText requires LoadFont and graphics".to_string())
                    }
                } else {
                    ExecResult::Fault(
                        "DrawText expects (Str, Float, Float, Float, Array)".to_string(),
                    )
                }
            }
            Node::GetLastKeypress => {
                let mut kb = self.keyboard_buffer.lock().unwrap();
                let txt = kb.clone();
                kb.clear();
                ExecResult::Value(RelType::Str(txt))
            }
            Node::PlayAudioFile(path_node) => {
                if let ExecResult::Value(RelType::Str(path)) = self.evaluate(path_node) {
                    if let Ok(mut reader) = hound::WavReader::open(path) {
                        let spec = reader.spec();
                        let samples: Vec<f32> = match spec.sample_format {
                            hound::SampleFormat::Float => {
                                reader.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect()
                            }
                            hound::SampleFormat::Int => reader
                                .samples::<i16>()
                                .map(|s| (s.unwrap_or(0) as f32) / 32768.0)
                                .collect(),
                        };

                        if let Some(stream_samples) = &self.stream_samples {
                            let mut lock = stream_samples.lock().unwrap();
                            *lock = samples;
                            if let Some(pos) = &self.stream_pos {
                                *pos.lock().unwrap() = 0;
                            }
                        }
                        ExecResult::Value(RelType::Void)
                    } else {
                        ExecResult::Fault("PlayAudioFile failed to open wav".to_string())
                    }
                } else {
                    ExecResult::Fault("PlayAudioFile expects String".to_string())
                }
            }
            Node::RenderAsset(shader_node, mesh_node, tex_node, uniform_node) => {
                let shader_val = self.evaluate(shader_node);
                let mesh_val = self.evaluate(mesh_node);
                let tex_val = self.evaluate(tex_node);
                let uniform_val = self.evaluate(uniform_node);

                if let (
                    ExecResult::Value(RelType::Int(s_id)),
                    ExecResult::Value(RelType::Int(m_id)),
                    ExecResult::Value(RelType::Int(t_id)),
                ) = (shader_val, mesh_val, tex_val)
                {
                    if let (Some(device), Some(queue), Some(surface), Some(config)) =
                        (&self.device, &self.queue, &self.surface, &self.config)
                    {
                        if s_id < 0 || s_id as usize >= self.shaders.len() {
                            return ExecResult::Fault("Invalid Shader ID".to_string());
                        }
                        if m_id < 0 || m_id as usize >= self.meshes.len() {
                            return ExecResult::Fault("Invalid Mesh ID".to_string());
                        }
                        if t_id < 0 || t_id as usize >= self.textures.len() {
                            return ExecResult::Fault("Invalid Texture ID".to_string());
                        }

                        let shader = &self.shaders[s_id as usize];
                        let mesh = &self.meshes[m_id as usize];
                        let texture_bind = &self.textures[t_id as usize];

                        let uniform_bind_group_layout =
                            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                                entries: &[wgpu::BindGroupLayoutEntry {
                                    binding: 0,
                                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                    ty: wgpu::BindingType::Buffer {
                                        ty: wgpu::BufferBindingType::Uniform,
                                        has_dynamic_offset: false,
                                        min_binding_size: None,
                                    },
                                    count: None,
                                }],
                                label: Some("uniform_bind_group_layout"),
                            });

                        let pipeline_layout =
                            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                                label: None,
                                bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind.3],
                                push_constant_ranges: &[],
                            });

                        let pipeline =
                            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                                label: Some("Asset Pipeline"),
                                layout: Some(&pipeline_layout),
                                vertex: wgpu::VertexState {
                                    module: shader,
                                    entry_point: Some("vs_main"),
                                    buffers: &[wgpu::VertexBufferLayout {
                                        array_stride: 32 as wgpu::BufferAddress,
                                        step_mode: wgpu::VertexStepMode::Vertex,
                                        attributes: &[
                                            wgpu::VertexAttribute {
                                                offset: 0,
                                                shader_location: 0,
                                                format: wgpu::VertexFormat::Float32x3,
                                            },
                                            wgpu::VertexAttribute {
                                                offset: 12,
                                                shader_location: 1,
                                                format: wgpu::VertexFormat::Float32x2,
                                            },
                                            wgpu::VertexAttribute {
                                                offset: 20,
                                                shader_location: 2,
                                                format: wgpu::VertexFormat::Float32x3,
                                            },
                                        ],
                                    }],
                                    compilation_options: wgpu::PipelineCompilationOptions::default(
                                    ),
                                },
                                fragment: Some(wgpu::FragmentState {
                                    module: shader,
                                    entry_point: Some("fs_main"),
                                    targets: &[Some(wgpu::ColorTargetState {
                                        format: config.format,
                                        blend: Some(wgpu::BlendState::REPLACE),
                                        write_mask: wgpu::ColorWrites::ALL,
                                    })],
                                    compilation_options: wgpu::PipelineCompilationOptions::default(
                                    ),
                                }),
                                primitive: wgpu::PrimitiveState {
                                    topology: wgpu::PrimitiveTopology::TriangleList,
                                    strip_index_format: None,
                                    front_face: wgpu::FrontFace::Ccw,
                                    cull_mode: Some(wgpu::Face::Back),
                                    unclipped_depth: false,
                                    polygon_mode: wgpu::PolygonMode::Fill,
                                    conservative: false,
                                },
                                depth_stencil: None, // Simplified for now, relies on ordering or simple scenes
                                multisample: wgpu::MultisampleState::default(),
                                multiview: None,
                                cache: None,
                            });

                        let mut active_bind_group = None;
                        if let ExecResult::Value(RelType::Array(arr)) = uniform_val {
                            let floats: Vec<f32> = arr
                                .into_iter()
                                .map(|v| match v {
                                    RelType::Float(f) => f as f32,
                                    RelType::Int(i) => i as f32,
                                    _ => 0.0,
                                })
                                .collect();

                            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                                label: Some("Uniform Buffer"),
                                size: (floats.len() * 4).max(64) as u64,
                                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                                mapped_at_creation: false,
                            });
                            queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&floats));

                            active_bind_group =
                                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                                    layout: &uniform_bind_group_layout,
                                    entries: &[wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: buffer.as_entire_binding(),
                                    }],
                                    label: Some("uniform_bind_group"),
                                }));
                        }

                        match surface.get_current_texture() {
                            Ok(frame) => {
                                let view = frame
                                    .texture
                                    .create_view(&wgpu::TextureViewDescriptor::default());
                                let mut encoder = device.create_command_encoder(
                                    &wgpu::CommandEncoderDescriptor::default(),
                                );
                                {
                                    let mut rpass =
                                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                            label: Some("Render Pass"),
                                            color_attachments: &[Some(
                                                wgpu::RenderPassColorAttachment {
                                                    view: &view,
                                                    resolve_target: None,
                                                    ops: wgpu::Operations {
                                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                                            r: 0.1,
                                                            g: 0.2,
                                                            b: 0.3,
                                                            a: 1.0,
                                                        }),
                                                        store: wgpu::StoreOp::Store,
                                                    },
                                                },
                                            )],
                                            depth_stencil_attachment: None,
                                            timestamp_writes: None,
                                            occlusion_query_set: None,
                                        });
                                    rpass.set_pipeline(&pipeline);

                                    // Bind VBO & IBO
                                    rpass.set_vertex_buffer(0, mesh.vbo.slice(..));
                                    rpass.set_index_buffer(
                                        mesh.ibo.slice(..),
                                        wgpu::IndexFormat::Uint32,
                                    );

                                    // Bind Uniforms
                                    if let Some(bg) = &active_bind_group {
                                        rpass.set_bind_group(0, bg, &[]);
                                    }

                                    // Bind Texture (Group 1)
                                    rpass.set_bind_group(1, &texture_bind.2, &[]);

                                    rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
                                }
                                queue.submit(Some(encoder.finish()));
                                frame.present();
                                ExecResult::Value(RelType::Void)
                            }
                            Err(e) => ExecResult::Fault(format!("RenderAsset failed: {:?}", e)),
                        }
                    } else {
                        ExecResult::Fault("Graphics context not initialized".to_string())
                    }
                } else {
                    ExecResult::Fault("RenderAsset expects (Int, Int, Int, Array)".to_string())
                }
            }
            Node::PollEvents(body) => {
                if let Some(mut event_loop) = self.event_loop.take() {
                    use winit::application::ApplicationHandler;
                    use winit::event::WindowEvent;
                    use winit::event_loop::ActiveEventLoop;
                    use winit::platform::run_on_demand::EventLoopExtRunOnDemand;
                    use winit::window::WindowId;

                    struct KnotenApp<'a> {
                        engine: &'a mut ExecutionEngine,
                        body: &'a Node,
                        exit: bool,
                    }

                    impl<'a> ApplicationHandler for KnotenApp<'a> {
                        fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

                        fn window_event(
                            &mut self,
                            event_loop: &ActiveEventLoop,
                            _id: WindowId,
                            event: WindowEvent,
                        ) {
                            if let (Some(state), Some(window)) =
                                (&mut self.engine.egui_state, &self.engine.window)
                            {
                                let _ = state.on_window_event(window.as_ref(), &event);
                            }
                            match event {
                                WindowEvent::CloseRequested => {
                                    event_loop.exit();
                                    self.exit = true;
                                }
                                WindowEvent::KeyboardInput { event: key_ev, .. } => {
                                    let is_pressed =
                                        key_ev.state == winit::event::ElementState::Pressed;
                                    if let winit::keyboard::Key::Named(k) = &key_ev.logical_key {
                                        if is_pressed
                                            && let winit::keyboard::NamedKey::Backspace = k
                                        {
                                            let mut kb =
                                                self.engine.keyboard_buffer.lock().unwrap();
                                            kb.pop();
                                        }
                                    } else if let winit::keyboard::Key::Character(c) =
                                        &key_ev.logical_key
                                    {
                                        if is_pressed {
                                            let mut kb =
                                                self.engine.keyboard_buffer.lock().unwrap();
                                            kb.push_str(c);
                                        }
                                        match c.as_str() {
                                            "w" | "W" => self.engine.input_w = is_pressed,
                                            "a" | "A" => self.engine.input_a = is_pressed,
                                            "s" | "S" => self.engine.input_s = is_pressed,
                                            "d" | "D" => self.engine.input_d = is_pressed,
                                            " " => self.engine.input_space = is_pressed,
                                            _ => {}
                                        }
                                    }
                                }
                                WindowEvent::Resized(physical_size) => {
                                    if let (Some(surface), Some(device), Some(config)) = (
                                        &self.engine.surface,
                                        &self.engine.device,
                                        &mut self.engine.config,
                                    ) {
                                        config.width = physical_size.width.max(1);
                                        config.height = physical_size.height.max(1);
                                        surface.configure(device, config);

                                        let depth_texture =
                                            device.create_texture(&wgpu::TextureDescriptor {
                                                label: Some("Depth Texture"),
                                                size: wgpu::Extent3d {
                                                    width: config.width,
                                                    height: config.height,
                                                    depth_or_array_layers: 1,
                                                },
                                                mip_level_count: 1,
                                                sample_count: 1,
                                                dimension: wgpu::TextureDimension::D2,
                                                format: wgpu::TextureFormat::Depth32Float,
                                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                                                    | wgpu::TextureUsages::TEXTURE_BINDING,
                                                view_formats: &[],
                                            });
                                        self.engine.depth_texture_view =
                                            Some(depth_texture.create_view(
                                                &wgpu::TextureViewDescriptor::default(),
                                            ));
                                    }
                                }
                                WindowEvent::MouseInput { state, button, .. } => {
                                    if self.engine.interaction_enabled {
                                        let is_pressed =
                                            state == winit::event::ElementState::Pressed;
                                        if is_pressed {
                                            let yaw = self.engine.camera_yaw;
                                            let pitch = self.engine.camera_pitch;
                                            let (sy, cy) = yaw.sin_cos();
                                            let (sp, cp) = pitch.sin_cos();
                                            let forward =
                                                cgmath::Vector3::new(sy * cp, sp, cy * cp)
                                                    .normalize();
                                            let origin = cgmath::Point3::new(
                                                self.engine.camera_pos[0],
                                                self.engine.camera_pos[1],
                                                self.engine.camera_pos[2],
                                            );

                                            if let Some((hit_pos, normal)) =
                                                self.engine.raycast_voxels(origin, forward, 5.0)
                                            {
                                                if button == winit::event::MouseButton::Left {
                                                    // Break
                                                    if self
                                                        .engine
                                                        .voxel_map
                                                        .remove(&hit_pos)
                                                        .is_some()
                                                    {
                                                        self.engine.voxel_map_dirty = true;
                                                    }
                                                } else if button == winit::event::MouseButton::Right
                                                {
                                                    // Place
                                                    let place_pos = [
                                                        hit_pos[0] + normal[0],
                                                        hit_pos[1] + normal[1],
                                                        hit_pos[2] + normal[2],
                                                    ];
                                                    self.engine.voxel_map.insert(place_pos, 2); // Stone
                                                    self.engine.voxel_map_dirty = true;
                                                }

                                                // Amiga Sound Feedback with Random Pitch
                                                if let Some((_stream, handle)) =
                                                    &self.engine.audio_stream_handle
                                                    && let Some(sample_bytes) =
                                                        self.engine.samples.get(&1)
                                                {
                                                    // Assume 1 is jump/break
                                                    let cursor =
                                                        std::io::Cursor::new(sample_bytes.clone());
                                                    if let Ok(source) = rodio::Decoder::new(cursor)
                                                    {
                                                        use rodio::Source;
                                                        let random_pitch =
                                                            0.9 + (rand::random::<f32>() * 0.2);
                                                        let source =
                                                            source.amplify(1.0).speed(random_pitch);
                                                        let _ = handle
                                                            .play_raw(source.convert_samples());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }

                        fn device_event(
                            &mut self,
                            _event_loop: &ActiveEventLoop,
                            _device_id: winit::event::DeviceId,
                            event: winit::event::DeviceEvent,
                        ) {
                            if self.engine.camera_active
                                && let winit::event::DeviceEvent::MouseMotion { delta } = event
                            {
                                self.engine.camera_yaw += delta.0 as f32 * 0.002;
                                self.engine.camera_pitch -= delta.1 as f32 * 0.002;

                                let limit = std::f32::consts::FRAC_PI_2 - 0.01;
                                if self.engine.camera_pitch > limit {
                                    self.engine.camera_pitch = limit;
                                } else if self.engine.camera_pitch < -limit {
                                    self.engine.camera_pitch = -limit;
                                }
                            }
                        }

                        fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
                            if self.engine.camera_active {
                                let speed = 0.05;
                                let yaw = self.engine.camera_yaw;
                                let (sy, cy) = yaw.sin_cos();
                                let mut dx = 0.0;
                                let mut dz = 0.0;

                                if self.engine.input_w {
                                    dx -= sy * speed;
                                    dz -= cy * speed;
                                }
                                if self.engine.input_s {
                                    dx += sy * speed;
                                    dz += cy * speed;
                                }
                                if self.engine.input_a {
                                    dx -= cy * speed;
                                    dz += sy * speed;
                                }
                                if self.engine.input_d {
                                    dx += cy * speed;
                                    dz -= sy * speed;
                                }

                                if self.engine.physics_enabled {
                                    // Apply Gravity
                                    self.engine.velocity_y -= 0.008;

                                    // Handle Jump (Spacebar)
                                    if self.engine.input_space && self.engine.is_grounded {
                                        self.engine.velocity_y = 0.15;
                                        self.engine.is_grounded = false;

                                        // Jump Sound Feedback (Sample ID 1)
                                        if let Some((_, handle)) = &self.engine.audio_stream_handle
                                            && let Some(sample_bytes) = self.engine.samples.get(&1)
                                        {
                                            let cursor = std::io::Cursor::new(sample_bytes.clone());
                                            if let Ok(source) = rodio::Decoder::new(cursor) {
                                                use rodio::Source;
                                                let source = source.amplify(0.5).speed(1.2);
                                                let _ = handle.play_raw(source.convert_samples());
                                            }
                                        }
                                    }
                                }

                                // Apply Physics-Based Movement with AABB Collision
                                if self.engine.physics_enabled {
                                    let mut new_pos = self.engine.camera_pos;

                                    // 1. Move Y (Gravity/Jump)
                                    new_pos[1] += self.engine.velocity_y;

                                    // Collision Y
                                    let player_height = 1.6;
                                    let _player_radius = 0.3;
                                    let mut collided_y = false;

                                    // Check feet area for Y collision
                                    let foot_y = (new_pos[1] - player_height).floor() as i64;
                                    let head_y = new_pos[1].floor() as i64;

                                    // Simple Ground Check against Voxel Map
                                    let check_x = new_pos[0].floor() as i64;
                                    let check_z = new_pos[2].floor() as i64;

                                    if self
                                        .engine
                                        .voxel_map
                                        .contains_key(&[check_x, foot_y, check_z])
                                    {
                                        if self.engine.velocity_y < 0.0 {
                                            new_pos[1] = (foot_y + 1) as f32 + player_height;
                                            self.engine.velocity_y = 0.0;
                                            self.engine.is_grounded = true;
                                            collided_y = true;
                                        }
                                    } else {
                                        self.engine.is_grounded = false;
                                    }

                                    // Ceiling check
                                    if !collided_y
                                        && self
                                            .engine
                                            .voxel_map
                                            .contains_key(&[check_x, head_y, check_z])
                                        && self.engine.velocity_y > 0.0
                                    {
                                        new_pos[1] = head_y as f32 - 0.1;
                                        self.engine.velocity_y = 0.0;
                                    }

                                    // 2. Move X & Z (WASD) - Only if not colliding
                                    let try_x = new_pos[0] + dx;
                                    let try_z = new_pos[2] + dz;

                                    let tx = try_x.floor() as i64;
                                    let tz = try_z.floor() as i64;
                                    let ty = (new_pos[1] - 0.5).floor() as i64; // Check body level

                                    if !self.engine.voxel_map.contains_key(&[tx, ty, check_z]) {
                                        new_pos[0] = try_x;
                                    }
                                    if !self.engine.voxel_map.contains_key(&[check_x, ty, tz]) {
                                        new_pos[2] = try_z;
                                    }

                                    self.engine.camera_pos = new_pos;
                                } else {
                                    // Noclip Movement (Sprint 17 style)
                                    self.engine.camera_pos[0] += dx;
                                    self.engine.camera_pos[2] += dz;
                                }
                            }

                            let egui_ctx = self.engine.egui_ctx.clone();
                            if let (Some(ctx), Some(state), Some(window)) =
                                (&egui_ctx, &mut self.engine.egui_state, &self.engine.window)
                            {
                                let raw_input = state.take_egui_input(window.as_ref());
                                ctx.begin_pass(raw_input);
                            }

                            let res = self.engine.evaluate(self.body);

                            let has_voxels = self.engine.camera_active
                                && (!self.engine.voxel_instances.is_empty()
                                    || self.engine.voxel_map_active);
                            if has_voxels {
                                self.engine.ensure_voxel_pipeline();
                            }

                            if let (
                                Some(ctx),
                                Some(state),
                                Some(renderer),
                                Some(device),
                                Some(queue),
                                Some(surface),
                                Some(window),
                                Some(config),
                                depth_view_opt,
                            ) = (
                                &self.engine.egui_ctx,
                                &mut self.engine.egui_state,
                                &mut self.engine.egui_renderer,
                                &self.engine.device,
                                &self.engine.queue,
                                &self.engine.surface,
                                &self.engine.window,
                                &self.engine.config,
                                &self.engine.depth_texture_view,
                            ) {
                                let full_output = ctx.end_pass();
                                state.handle_platform_output(
                                    window.as_ref(),
                                    full_output.platform_output,
                                );
                                let paint_jobs = ctx
                                    .tessellate(full_output.shapes, full_output.pixels_per_point);

                                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                                    size_in_pixels: [config.width, config.height],
                                    pixels_per_point: window.scale_factor() as f32,
                                };

                                for (id, image_delta) in &full_output.textures_delta.set {
                                    renderer.update_texture(device, queue, *id, image_delta);
                                }

                                if let Ok(frame) = surface.get_current_texture() {
                                    let view = frame
                                        .texture
                                        .create_view(&wgpu::TextureViewDescriptor::default());
                                    let mut encoder = device.create_command_encoder(
                                        &wgpu::CommandEncoderDescriptor::default(),
                                    );

                                    let has_voxels = self.engine.camera_active
                                        && (!self.engine.voxel_instances.is_empty()
                                            || self.engine.voxel_map_active);

                                    if has_voxels {
                                        let aspect = config.width as f32 / config.height as f32;
                                        let proj = cgmath::perspective(
                                            cgmath::Deg(self.engine.camera_fov),
                                            aspect,
                                            0.1,
                                            1000.0,
                                        );
                                        let pos = cgmath::Point3::new(
                                            self.engine.camera_pos[0],
                                            self.engine.camera_pos[1],
                                            self.engine.camera_pos[2],
                                        );

                                        let yaw = self.engine.camera_yaw;
                                        let pitch = self.engine.camera_pitch;
                                        let (sy, cy) = yaw.sin_cos();
                                        let (sp, cp) = pitch.sin_cos();

                                        use cgmath::InnerSpace;
                                        let forward =
                                            cgmath::Vector3::new(sy * cp, sp, cy * cp).normalize();
                                        let view_mat = cgmath::Matrix4::look_to_rh(
                                            pos,
                                            forward,
                                            cgmath::Vector3::unit_y(),
                                        );
                                        let view_proj = proj * view_mat;

                                        let matrix_ref: &[f32; 16] = view_proj.as_ref();

                                        if let Some(ubo) = &self.engine.voxel_ubo {
                                            queue.write_buffer(
                                                ubo,
                                                0,
                                                bytemuck::cast_slice(matrix_ref),
                                            );
                                            let cp = [
                                                self.engine.camera_pos[0],
                                                self.engine.camera_pos[1],
                                                self.engine.camera_pos[2],
                                                1.0f32,
                                            ];
                                            queue.write_buffer(ubo, 64, bytemuck::cast_slice(&cp));
                                            let sc = [0.5f32, 0.8f32, 1.0f32, 1.0f32];
                                            queue.write_buffer(ubo, 80, bytemuck::cast_slice(&sc));
                                        }

                                        // Update voxel instances from map if active and dirty
                                        if self.engine.voxel_map_active
                                            && self.engine.voxel_map_dirty
                                        {
                                            self.engine.voxel_instances.clear();
                                            for (&[x, y, z], &id) in self.engine.voxel_map.iter() {
                                                self.engine.voxel_instances.push(VoxelInstance {
                                                    instance_pos_and_id: [
                                                        x as f32, y as f32, z as f32, id as f32,
                                                    ],
                                                });
                                            }

                                            // Rebuild the buffer
                                            if !self.engine.voxel_instances.is_empty() {
                                                self.engine.voxel_instance_buffer =
                                                    Some(device.create_buffer_init(
                                                        &wgpu::util::BufferInitDescriptor {
                                                            label: Some("Instance Buffer"),
                                                            contents: bytemuck::cast_slice(
                                                                &self.engine.voxel_instances,
                                                            ),
                                                            usage: wgpu::BufferUsages::VERTEX,
                                                        },
                                                    ));
                                            } else {
                                                self.engine.voxel_instance_buffer = None;
                                            }

                                            self.engine.voxel_map_dirty = false;
                                        }

                                        if let (
                                            Some(pipeline),
                                            Some(vbo),
                                            Some(ibo),
                                            Some(bind_group),
                                            Some(atlas_bind_group),
                                            Some(depth_view),
                                            Some(instance_buf),
                                        ) = (
                                            &self.engine.voxel_pipeline,
                                            &self.engine.voxel_vbo,
                                            &self.engine.voxel_ibo,
                                            &self.engine.voxel_bind_group,
                                            &self.engine.voxel_atlas_bind_group,
                                            depth_view_opt.as_ref(),
                                            self.engine.voxel_instance_buffer.as_ref(),
                                        ) {
                                            let mut rpass =
                                                encoder
                                                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                                                    label: Some("Voxel Pass"),
                                                    color_attachments: &[Some(
                                                        wgpu::RenderPassColorAttachment {
                                                            view: &view,
                                                            resolve_target: None,
                                                            ops: wgpu::Operations {
                                                                load: wgpu::LoadOp::Clear(
                                                                    wgpu::Color {
                                                                        r: 0.5,
                                                                        g: 0.8,
                                                                        b: 1.0,
                                                                        a: 1.0,
                                                                    },
                                                                ),
                                                                store: wgpu::StoreOp::Store,
                                                            },
                                                        },
                                                    )],
                                                    depth_stencil_attachment: Some(
                                                        wgpu::RenderPassDepthStencilAttachment {
                                                            view: depth_view,
                                                            depth_ops: Some(wgpu::Operations {
                                                                load: wgpu::LoadOp::Clear(1.0),
                                                                store: wgpu::StoreOp::Store,
                                                            }),
                                                            stencil_ops: None,
                                                        },
                                                    ),
                                                    timestamp_writes: None,
                                                    occlusion_query_set: None,
                                                });

                                            rpass.set_pipeline(pipeline);
                                            rpass.set_bind_group(0, bind_group, &[]);
                                            rpass.set_bind_group(1, atlas_bind_group, &[]);
                                            rpass.set_vertex_buffer(0, vbo.slice(..));
                                            rpass.set_vertex_buffer(1, instance_buf.slice(..));
                                            rpass.set_index_buffer(
                                                ibo.slice(..),
                                                wgpu::IndexFormat::Uint32,
                                            );
                                            rpass.draw_indexed(
                                                0..36,
                                                0,
                                                0..self.engine.voxel_instances.len() as u32,
                                            );
                                        }
                                    }

                                    renderer.update_buffers(
                                        device,
                                        queue,
                                        &mut encoder,
                                        &paint_jobs,
                                        &screen_descriptor,
                                    );

                                    {
                                        {
                                            let mut rpass = encoder
                                                .begin_render_pass(&wgpu::RenderPassDescriptor {
                                                    color_attachments: &[Some(
                                                        wgpu::RenderPassColorAttachment {
                                                            view: &view,
                                                            resolve_target: None,
                                                            ops: wgpu::Operations {
                                                                load: if has_voxels {
                                                                    wgpu::LoadOp::Load
                                                                } else {
                                                                    wgpu::LoadOp::Clear(
                                                                        wgpu::Color {
                                                                            r: 0.05,
                                                                            g: 0.05,
                                                                            b: 0.05,
                                                                            a: 1.0,
                                                                        },
                                                                    )
                                                                },
                                                                store: wgpu::StoreOp::Store,
                                                            },
                                                        },
                                                    )],
                                                    depth_stencil_attachment: depth_view_opt
                                                        .as_ref()
                                                        .map(|dv| {
                                                            wgpu::RenderPassDepthStencilAttachment {
                                                                view: dv,
                                                                depth_ops: Some(wgpu::Operations {
                                                                    load: if has_voxels {
                                                                        wgpu::LoadOp::Load
                                                                    } else {
                                                                        wgpu::LoadOp::Clear(1.0)
                                                                    },
                                                                    store: wgpu::StoreOp::Store,
                                                                }),
                                                                stencil_ops: None,
                                                            }
                                                        }),
                                                    timestamp_writes: None,
                                                    occlusion_query_set: None,
                                                    label: Some("egui render pass"),
                                                })
                                                .forget_lifetime();

                                            renderer.render(
                                                &mut rpass,
                                                &paint_jobs,
                                                &screen_descriptor,
                                            );
                                        }
                                    }

                                    queue.submit(Some(encoder.finish()));
                                    frame.present();
                                }

                                for id in &full_output.textures_delta.free {
                                    renderer.free_texture(id);
                                }
                            }

                            if let ExecResult::ReturnBlockInfo(_) | ExecResult::Fault(_) = res {
                                event_loop.exit();
                            }
                        }
                    }

                    let mut app = KnotenApp {
                        engine: self,
                        body,
                        exit: false,
                    };
                    let _ = event_loop.run_app_on_demand(&mut app);

                    let exit = app.exit;

                    self.event_loop = Some(event_loop);

                    if exit {
                        ExecResult::ReturnBlockInfo(RelType::Void)
                    } else {
                        ExecResult::Value(RelType::Void)
                    }
                } else {
                    ExecResult::Fault("PollEvents requires an active Window".to_string())
                }
            }

            // Audio Engine (CPAL FFI)
            Node::InitAudio => {
                let host = cpal::default_host();
                if let Some(device) = host.default_output_device() {
                    let mut supported_configs_range = device.supported_output_configs().unwrap();
                    let supported_config = supported_configs_range
                        .next()
                        .unwrap()
                        .with_max_sample_rate();
                    let sample_rate = supported_config.sample_rate().0 as f32; // wait, if cpal changed this to u32, this will work.
                    let config = supported_config.config();
                    let channels = config.channels as usize;

                    let voices = Arc::new(Mutex::new([VoiceState::default(); 4]));
                    self.voices = Some(voices.clone());

                    let stream_samples = Arc::new(Mutex::new(Vec::<f32>::new()));
                    let stream_pos = Arc::new(Mutex::new(0usize));
                    self.stream_samples = Some(stream_samples.clone());
                    self.stream_pos = Some(stream_pos.clone());

                    let err_fn =
                        |err| eprintln!("An error occurred on the output audio stream: {}", err);

                    let stream = match supported_config.sample_format() {
                        cpal::SampleFormat::F32 => {
                            let stream_samples_clone = stream_samples.clone();
                            let stream_pos_clone = stream_pos.clone();
                            device
                                .build_output_stream(
                                    &config,
                                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                        let mut voices_lock = voices.lock().unwrap();
                                        let mut sample_idx = stream_pos_clone.lock().unwrap();
                                        let samples_lock = stream_samples_clone.lock().unwrap();

                                        for frame in data.chunks_mut(channels) {
                                            let mut sample: f32 = 0.0;

                                            if *sample_idx < samples_lock.len() {
                                                sample += samples_lock[*sample_idx];
                                                *sample_idx += 1;
                                            }

                                            for voice in voices_lock.iter_mut() {
                                                if voice.active {
                                                    voice.phase = (voice.phase
                                                        + voice.freq / sample_rate)
                                                        % 1.0;
                                                    let p = voice.phase;

                                                    let v_sample = match voice.waveform {
                                                        0 => (p * 2.0 * std::f32::consts::PI).sin(), // Sine
                                                        1 => {
                                                            if p < 0.5 {
                                                                1.0
                                                            } else {
                                                                -1.0
                                                            }
                                                        } // Square
                                                        2 => (p * 2.0) - 1.0, // Saw
                                                        3 => {
                                                            if p < 0.5 {
                                                                p * 4.0 - 1.0
                                                            } else {
                                                                3.0 - p * 4.0
                                                            }
                                                        } // Tri
                                                        4 => rand::random::<f32>() * 2.0 - 1.0, // Noise
                                                        _ => 0.0,
                                                    };
                                                    sample += v_sample * 0.15; // Volume scaling
                                                }
                                            }
                                            for channel in frame.iter_mut() {
                                                *channel = sample;
                                            }
                                        }
                                    },
                                    err_fn,
                                    None,
                                )
                                .unwrap()
                        }
                        cpal::SampleFormat::I16 => {
                            let stream_samples_clone = stream_samples.clone();
                            let stream_pos_clone = stream_pos.clone();
                            device
                                .build_output_stream(
                                    &config,
                                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                                        let mut voices_lock = voices.lock().unwrap();
                                        let mut sample_idx = stream_pos_clone.lock().unwrap();
                                        let samples_lock = stream_samples_clone.lock().unwrap();

                                        for frame in data.chunks_mut(channels) {
                                            let mut sample: f32 = 0.0;

                                            if *sample_idx < samples_lock.len() {
                                                sample += samples_lock[*sample_idx];
                                                *sample_idx += 1;
                                            }

                                            for voice in voices_lock.iter_mut() {
                                                if voice.active {
                                                    voice.phase = (voice.phase
                                                        + voice.freq / sample_rate)
                                                        % 1.0;
                                                    let p = voice.phase;

                                                    let v_sample = match voice.waveform {
                                                        0 => (p * 2.0 * std::f32::consts::PI).sin(),
                                                        1 => {
                                                            if p < 0.5 {
                                                                1.0
                                                            } else {
                                                                -1.0
                                                            }
                                                        }
                                                        2 => (p * 2.0) - 1.0,
                                                        3 => {
                                                            if p < 0.5 {
                                                                p * 4.0 - 1.0
                                                            } else {
                                                                3.0 - p * 4.0
                                                            }
                                                        }
                                                        4 => rand::random::<f32>() * 2.0 - 1.0,
                                                        _ => 0.0,
                                                    };
                                                    sample += v_sample * 0.15;
                                                }
                                            }
                                            let int_sample = (sample.clamp(-1.0, 1.0)
                                                * f32::from(i16::MAX))
                                                as i16;
                                            for channel in frame.iter_mut() {
                                                *channel = int_sample;
                                            }
                                        }
                                    },
                                    err_fn,
                                    None,
                                )
                                .unwrap()
                        }
                        cpal::SampleFormat::U16 => {
                            let stream_samples_clone = stream_samples.clone();
                            let stream_pos_clone = stream_pos.clone();
                            device
                                .build_output_stream(
                                    &config,
                                    move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                                        let mut voices_lock = voices.lock().unwrap();
                                        let mut sample_idx = stream_pos_clone.lock().unwrap();
                                        let samples_lock = stream_samples_clone.lock().unwrap();

                                        for frame in data.chunks_mut(channels) {
                                            let mut sample: f32 = 0.0;

                                            if *sample_idx < samples_lock.len() {
                                                sample += samples_lock[*sample_idx];
                                                *sample_idx += 1;
                                            }

                                            for voice in voices_lock.iter_mut() {
                                                if voice.active {
                                                    voice.phase = (voice.phase
                                                        + voice.freq / sample_rate)
                                                        % 1.0;
                                                    let p = voice.phase;

                                                    let v_sample = match voice.waveform {
                                                        0 => (p * 2.0 * std::f32::consts::PI).sin(),
                                                        1 => {
                                                            if p < 0.5 {
                                                                1.0
                                                            } else {
                                                                -1.0
                                                            }
                                                        }
                                                        2 => (p * 2.0) - 1.0,
                                                        3 => {
                                                            if p < 0.5 {
                                                                p * 4.0 - 1.0
                                                            } else {
                                                                3.0 - p * 4.0
                                                            }
                                                        }
                                                        4 => rand::random::<f32>() * 2.0 - 1.0,
                                                        _ => 0.0,
                                                    };
                                                    sample += v_sample * 0.15;
                                                }
                                            }
                                            let int_sample = ((sample.clamp(-1.0, 1.0) * 0.5 + 0.5)
                                                * f32::from(u16::MAX))
                                                as u16;
                                            for channel in frame.iter_mut() {
                                                *channel = int_sample;
                                            }
                                        }
                                    },
                                    err_fn,
                                    None,
                                )
                                .unwrap()
                        }
                        cpal::SampleFormat::U8 => {
                            let stream_samples_clone = stream_samples.clone();
                            let stream_pos_clone = stream_pos.clone();
                            device
                                .build_output_stream(
                                    &config,
                                    move |data: &mut [u8], _: &cpal::OutputCallbackInfo| {
                                        let mut voices_lock = voices.lock().unwrap();
                                        let mut sample_idx = stream_pos_clone.lock().unwrap();
                                        let samples_lock = stream_samples_clone.lock().unwrap();

                                        for frame in data.chunks_mut(channels) {
                                            let mut sample: f32 = 0.0;

                                            if *sample_idx < samples_lock.len() {
                                                sample += samples_lock[*sample_idx];
                                                *sample_idx += 1;
                                            }

                                            for voice in voices_lock.iter_mut() {
                                                if voice.active {
                                                    voice.phase = (voice.phase
                                                        + voice.freq / sample_rate)
                                                        % 1.0;
                                                    let p = voice.phase;

                                                    let v_sample = match voice.waveform {
                                                        0 => (p * 2.0 * std::f32::consts::PI).sin(),
                                                        1 => {
                                                            if p < 0.5 {
                                                                1.0
                                                            } else {
                                                                -1.0
                                                            }
                                                        }
                                                        2 => (p * 2.0) - 1.0,
                                                        3 => {
                                                            if p < 0.5 {
                                                                p * 4.0 - 1.0
                                                            } else {
                                                                3.0 - p * 4.0
                                                            }
                                                        }
                                                        4 => rand::random::<f32>() * 2.0 - 1.0,
                                                        _ => 0.0,
                                                    };
                                                    sample += v_sample * 0.15;
                                                }
                                            }
                                            let int_sample = ((sample.clamp(-1.0, 1.0) * 0.5 + 0.5)
                                                * f32::from(u8::MAX))
                                                as u8;
                                            for channel in frame.iter_mut() {
                                                *channel = int_sample;
                                            }
                                        }
                                    },
                                    err_fn,
                                    None,
                                )
                                .unwrap()
                        }
                        f => panic!("Unsupported Audio Format: {:?}", f),
                    };

                    stream.play().unwrap();
                    self.audio_stream = Some(stream);
                    ExecResult::Value(RelType::Void)
                } else {
                    ExecResult::Fault("No Audio Output Device Available".to_string())
                }
            }
            Node::PlayNote(channel_node, freq_node, wave_node) => {
                let cv = self.evaluate(channel_node);
                let fv = self.evaluate(freq_node);
                let wv = self.evaluate(wave_node);

                if let (
                    Some(voices),
                    ExecResult::Value(RelType::Int(c)),
                    ExecResult::Value(RelType::Float(f)),
                    ExecResult::Value(RelType::Int(w)),
                ) = (&self.voices, cv, fv, wv)
                {
                    if (0..4).contains(&c) {
                        let mut v_lock = voices.lock().unwrap();
                        v_lock[c as usize].active = true;
                        v_lock[c as usize].freq = f as f32;
                        v_lock[c as usize].waveform = w as u8;
                        ExecResult::Value(RelType::Void)
                    } else {
                        ExecResult::Fault("Invalid Audio Channel ID".to_string())
                    }
                } else {
                    ExecResult::Fault(
                        "PlayNote expects (Int, Float, Int) and an InitAudio call".to_string(),
                    )
                }
            }
            Node::StopNote(channel_node) => {
                let cv = self.evaluate(channel_node);
                if let (Some(voices), ExecResult::Value(RelType::Int(c))) = (&self.voices, cv) {
                    if (0..4).contains(&c) {
                        let mut v_lock = voices.lock().unwrap();
                        v_lock[c as usize].active = false;
                        ExecResult::Value(RelType::Void)
                    } else {
                        ExecResult::Fault("Invalid Audio Channel ID".to_string())
                    }
                } else {
                    ExecResult::Fault("StopNote expects (Int) and an InitAudio call".to_string())
                }
            }

            // Flow
            Node::If(cond, then_br, else_br) => {
                let cv = self.evaluate(cond);
                match cv {
                    ExecResult::Value(RelType::Bool(true)) => self.evaluate(then_br),
                    ExecResult::Value(RelType::Bool(false)) => {
                        if let Some(eb) = else_br {
                            self.evaluate(eb)
                        } else {
                            ExecResult::Value(RelType::Void)
                        }
                    }
                    ExecResult::Fault(err) => ExecResult::Fault(err),
                    _ => ExecResult::Fault("If condition not a boolean".to_string()),
                }
            }
            Node::While(cond, body) => {
                loop {
                    match self.evaluate(cond) {
                        ExecResult::Value(RelType::Bool(true)) => match self.evaluate(body) {
                            ExecResult::ReturnBlockInfo(r) => {
                                return ExecResult::ReturnBlockInfo(r);
                            }
                            ExecResult::Fault(err) => return ExecResult::Fault(err),
                            _ => {}
                        },
                        ExecResult::Value(RelType::Bool(false)) => break,
                        ExecResult::Fault(err) => return ExecResult::Fault(err),
                        _ => return ExecResult::Fault("While condition not a boolean".to_string()),
                    }
                }
                ExecResult::Value(RelType::Void) // while evaluate returns void naturally unless return hits
            }
            Node::InitCamera(fov_node) => {
                let fov_res = self.evaluate(fov_node);
                if let ExecResult::Value(RelType::Float(f)) = fov_res {
                    self.camera_fov = f as f32;
                    self.camera_active = true;
                    if let Some(window) = &self.window {
                        let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined);
                        window.set_cursor_visible(false);
                    }
                    ExecResult::Value(RelType::Void)
                } else {
                    ExecResult::Fault("InitCamera expects (Float FOV)".to_string())
                }
            }
            Node::DrawVoxelGrid(positions_node) => {
                let pos_res = self.evaluate(positions_node);
                if let ExecResult::Value(RelType::Array(positions)) = pos_res {
                    if !self.voxel_map_active {
                        self.voxel_instances.clear();
                        for chunk in positions.chunks_exact(4) {
                            if let (
                                RelType::Float(x),
                                RelType::Float(y),
                                RelType::Float(z),
                                RelType::Int(id),
                            ) = (&chunk[0], &chunk[1], &chunk[2], &chunk[3])
                            {
                                self.voxel_instances.push(VoxelInstance {
                                    instance_pos_and_id: [
                                        *x as f32, *y as f32, *z as f32, *id as f32,
                                    ],
                                });
                            }
                        }
                        self.voxel_map_dirty = true;
                    } else {
                        // If map active, we ignore the static array after initial load if needed,
                        // or we can sync it once. Let's sync it once if map is empty.
                        if self.voxel_map.is_empty() {
                            for chunk in positions.chunks_exact(4) {
                                if let (
                                    RelType::Float(x),
                                    RelType::Float(y),
                                    RelType::Float(z),
                                    RelType::Int(id),
                                ) = (&chunk[0], &chunk[1], &chunk[2], &chunk[3])
                                {
                                    self.voxel_map
                                        .insert([*x as i64, *y as i64, *z as i64], *id as u8);
                                }
                            }
                            self.voxel_map_dirty = true;
                        }
                    }
                    ExecResult::Value(RelType::Void)
                } else {
                    ExecResult::Fault(
                        "DrawVoxelGrid requires an array of floats (X,Y,Z,ID)".to_string(),
                    )
                }
            }
            Node::LoadTextureAtlas(path_n, tile_size_n) => {
                let path_res = self.evaluate(path_n);
                let tile_size_res = self.evaluate(tile_size_n);

                if let (
                    ExecResult::Value(RelType::Str(path)),
                    ExecResult::Value(RelType::Float(_tile_size)), // Passing to shader logic eventually if dynamic
                ) = (path_res, tile_size_res)
                {
                    if let (Some(device), Some(queue)) = (&self.device, &self.queue) {
                        match image::open(&path) {
                            Ok(img) => {
                                let rgba = img.to_rgba8();
                                let dimensions = rgba.dimensions();

                                let texture_size = wgpu::Extent3d {
                                    width: dimensions.0,
                                    height: dimensions.1,
                                    depth_or_array_layers: 1,
                                };

                                let texture = device.create_texture(&wgpu::TextureDescriptor {
                                    size: texture_size,
                                    mip_level_count: 1,
                                    sample_count: 1,
                                    dimension: wgpu::TextureDimension::D2,
                                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                                        | wgpu::TextureUsages::COPY_DST,
                                    label: Some("Atlas Texture"),
                                    view_formats: &[],
                                });

                                queue.write_texture(
                                    wgpu::ImageCopyTexture {
                                        texture: &texture,
                                        mip_level: 0,
                                        origin: wgpu::Origin3d::ZERO,
                                        aspect: wgpu::TextureAspect::All,
                                    },
                                    &rgba,
                                    wgpu::ImageDataLayout {
                                        offset: 0,
                                        bytes_per_row: Some(4 * dimensions.0),
                                        rows_per_image: Some(dimensions.1),
                                    },
                                    texture_size,
                                );

                                let view =
                                    texture.create_view(&wgpu::TextureViewDescriptor::default());
                                let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                                    mag_filter: wgpu::FilterMode::Nearest, // CRISP PIXELS!
                                    min_filter: wgpu::FilterMode::Nearest,
                                    mipmap_filter: wgpu::FilterMode::Nearest,
                                    ..Default::default()
                                });

                                let layout = device.create_bind_group_layout(
                                    &wgpu::BindGroupLayoutDescriptor {
                                        entries: &[
                                            wgpu::BindGroupLayoutEntry {
                                                binding: 0,
                                                visibility: wgpu::ShaderStages::FRAGMENT,
                                                ty: wgpu::BindingType::Texture {
                                                    multisampled: false,
                                                    view_dimension: wgpu::TextureViewDimension::D2,
                                                    sample_type: wgpu::TextureSampleType::Float {
                                                        filterable: true,
                                                    },
                                                },
                                                count: None,
                                            },
                                            wgpu::BindGroupLayoutEntry {
                                                binding: 1,
                                                visibility: wgpu::ShaderStages::FRAGMENT,
                                                ty: wgpu::BindingType::Sampler(
                                                    wgpu::SamplerBindingType::Filtering,
                                                ),
                                                count: None,
                                            },
                                        ],
                                        label: Some("atlas_bind_group_layout"),
                                    },
                                );

                                let bind_group =
                                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                                        layout: &layout,
                                        entries: &[
                                            wgpu::BindGroupEntry {
                                                binding: 0,
                                                resource: wgpu::BindingResource::TextureView(&view),
                                            },
                                            wgpu::BindGroupEntry {
                                                binding: 1,
                                                resource: wgpu::BindingResource::Sampler(&sampler),
                                            },
                                        ],
                                        label: Some("atlas_bind_group"),
                                    });

                                self.voxel_atlas_bind_group = Some(bind_group);
                                ExecResult::Value(RelType::Void)
                            }
                            Err(e) => {
                                ExecResult::Fault(format!("Failed to open atlas {}: {}", path, e))
                            }
                        }
                    } else {
                        ExecResult::Fault("WGPU Device missing for Atlas Loading".to_string())
                    }
                } else {
                    ExecResult::Fault("LoadTextureAtlas expects (String, Float)".to_string())
                }
            }
            Node::LoadSample(id_n, path_n) => {
                let id_res = self.evaluate(id_n);
                let path_res = self.evaluate(path_n);

                if let (
                    ExecResult::Value(RelType::Int(id)),
                    ExecResult::Value(RelType::Str(path)),
                ) = (id_res, path_res)
                {
                    if let Ok(bytes) = std::fs::read(&path) {
                        self.samples.insert(id, bytes.into());
                        ExecResult::Value(RelType::Void)
                    } else {
                        ExecResult::Fault(format!("Failed to read sample {:?}", path))
                    }
                } else {
                    ExecResult::Fault("LoadSample expects (Int, String)".to_string())
                }
            }
            Node::PlaySample(id_n, vol_n, pitch_n) => {
                let id_res = self.evaluate(id_n);
                let vol_res = self.evaluate(vol_n);
                let pitch_res = self.evaluate(pitch_n);

                if let (
                    ExecResult::Value(RelType::Int(id)),
                    ExecResult::Value(RelType::Float(vol)),
                    ExecResult::Value(RelType::Float(pitch)),
                ) = (id_res, vol_res, pitch_res)
                {
                    if let Some((_, handle)) = &self.audio_stream_handle {
                        if let Some(sample_bytes) = self.samples.get(&id) {
                            let cursor = std::io::Cursor::new(sample_bytes.clone());
                            if let Ok(source) = rodio::Decoder::new(cursor) {
                                use rodio::Source;
                                let source = source.amplify(vol as f32).speed(pitch as f32);
                                let _ = handle.play_raw(source.convert_samples());
                                ExecResult::Value(RelType::Void)
                            } else {
                                ExecResult::Fault("Failed to decode sample".to_string())
                            }
                        } else {
                            ExecResult::Fault(format!("Sample ID {} not found", id))
                        }
                    } else {
                        ExecResult::Fault("Audio stream not initialized".to_string())
                    }
                } else {
                    ExecResult::Fault("PlaySample expects (Int, Float, Float)".to_string())
                }
            }
            Node::InitVoxelMap => {
                self.voxel_map_active = true;
                self.voxel_map_dirty = true;
                // Seed some initial floor if empty
                if self.voxel_map.is_empty() {
                    for x in -10..10 {
                        for z in -10..10 {
                            self.voxel_map.insert([x, -1, z], 1);
                        }
                    }
                }
                ExecResult::Value(RelType::Void)
            }
            Node::SetVoxel(x_n, y_n, z_n, id_n) => {
                let xr = self.evaluate(x_n);
                let yr = self.evaluate(y_n);
                let zr = self.evaluate(z_n);
                let idr = self.evaluate(id_n);

                if let (
                    ExecResult::Value(xv),
                    ExecResult::Value(yv),
                    ExecResult::Value(zv),
                    ExecResult::Value(idv),
                ) = (xr, yr, zr, idr)
                {
                    let x = match xv {
                        RelType::Int(i) => i,
                        RelType::Float(f) => f.floor() as i64,
                        _ => return ExecResult::Fault("SetVoxel X must be a Number".to_string()),
                    };
                    let y = match yv {
                        RelType::Int(i) => i,
                        RelType::Float(f) => f.floor() as i64,
                        _ => return ExecResult::Fault("SetVoxel Y must be a Number".to_string()),
                    };
                    let z = match zv {
                        RelType::Int(i) => i,
                        RelType::Float(f) => f.floor() as i64,
                        _ => return ExecResult::Fault("SetVoxel Z must be a Number".to_string()),
                    };
                    let id = match idv {
                        RelType::Int(i) => i as u8,
                        RelType::Float(f) => f.floor() as u8,
                        _ => return ExecResult::Fault("SetVoxel ID must be a Number".to_string()),
                    };

                    self.voxel_map.insert([x, y, z], id);
                    self.voxel_map_dirty = true;
                    ExecResult::Value(RelType::Void)
                } else {
                    ExecResult::Fault("SetVoxel arguments must evaluate to Values".to_string())
                }
            }
            Node::EnableInteraction(enabled_n) => {
                let res = self.evaluate(enabled_n);
                if let ExecResult::Value(RelType::Bool(b)) = res {
                    self.interaction_enabled = b;
                    ExecResult::Value(RelType::Void)
                } else {
                    ExecResult::Fault("EnableInteraction expects Boolean".to_string())
                }
            }
            Node::Block(nodes) => {
                let mut last_val = RelType::Void;
                for n in nodes {
                    match self.evaluate(n) {
                        ExecResult::ReturnBlockInfo(val) => {
                            return ExecResult::ReturnBlockInfo(val);
                        }
                        ExecResult::Fault(err) => return ExecResult::Fault(err),
                        ExecResult::Value(val) => {
                            last_val = val;
                        }
                    }
                }
                ExecResult::Value(last_val)
            }
            Node::EnablePhysics(enable_n) => {
                let res = self.evaluate(enable_n);
                if let ExecResult::Value(RelType::Bool(b)) = res {
                    self.physics_enabled = b;
                    ExecResult::Value(RelType::Void)
                } else {
                    ExecResult::Fault("EnablePhysics expects Boolean".to_string())
                }
            }
            Node::Return(val_node) => match self.evaluate(val_node) {
                ExecResult::Value(v) => ExecResult::ReturnBlockInfo(v),
                fault => fault,
            },
        }
    }

    pub fn raycast_voxels(
        &self,
        origin: cgmath::Point3<f32>,
        direction: cgmath::Vector3<f32>,
        max_dist: f32,
    ) -> Option<([i64; 3], [i64; 3])> {
        let mut x = origin.x.floor() as i64;
        let mut y = origin.y.floor() as i64;
        let mut z = origin.z.floor() as i64;

        let step_x = if direction.x > 0.0 { 1 } else { -1 };
        let step_y = if direction.y > 0.0 { 1 } else { -1 };
        let step_z = if direction.z > 0.0 { 1 } else { -1 };

        let t_delta_x = (1.0 / direction.x).abs();
        let t_delta_y = (1.0 / direction.y).abs();
        let t_delta_z = (1.0 / direction.z).abs();

        let mut t_max_x = if direction.x > 0.0 {
            (x as f32 + 1.0 - origin.x) * t_delta_x
        } else {
            (origin.x - x as f32) * t_delta_x
        };
        let mut t_max_y = if direction.y > 0.0 {
            (y as f32 + 1.0 - origin.y) * t_delta_y
        } else {
            (origin.y - y as f32) * t_delta_y
        };
        let mut t_max_z = if direction.z > 0.0 {
            (z as f32 + 1.0 - origin.z) * t_delta_z
        } else {
            (origin.z - z as f32) * t_delta_z
        };

        let mut face_normal = [0, 0, 0];
        let mut dist = 0.0;

        while dist < max_dist {
            if let Some(&id) = self.voxel_map.get(&[x, y, z])
                && id > 0
            {
                return Some(([x, y, z], face_normal));
            }

            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    dist = t_max_x;
                    t_max_x += t_delta_x;
                    x += step_x;
                    face_normal = [-step_x, 0, 0];
                } else {
                    dist = t_max_z;
                    t_max_z += t_delta_z;
                    z += step_z;
                    face_normal = [0, 0, -step_z];
                }
            } else if t_max_y < t_max_z {
                dist = t_max_y;
                t_max_y += t_delta_y;
                y += step_y;
                face_normal = [0, -step_y, 0];
            } else {
                dist = t_max_z;
                t_max_z += t_delta_z;
                z += step_z;
                face_normal = [0, 0, -step_z];
            }
        }
        None
    }

    fn do_math(&mut self, l: &Node, r: &Node, op: char) -> ExecResult {
        let lv = self.evaluate(l);
        let rv = self.evaluate(r);

        match (lv, rv) {
            (ExecResult::Value(RelType::Int(li)), ExecResult::Value(RelType::Int(ri))) => {
                match op {
                    '+' => ExecResult::Value(RelType::Int(li + ri)),
                    '-' => ExecResult::Value(RelType::Int(li - ri)),
                    '*' => ExecResult::Value(RelType::Int(li * ri)),
                    '/' => {
                        if ri == 0 {
                            ExecResult::Fault("Division by zero".to_string())
                        } else {
                            ExecResult::Value(RelType::Int(li / ri))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (ExecResult::Value(RelType::Float(lf)), ExecResult::Value(RelType::Float(rf))) => {
                match op {
                    '+' => ExecResult::Value(RelType::Float(lf + rf)),
                    '-' => ExecResult::Value(RelType::Float(lf - rf)),
                    '*' => ExecResult::Value(RelType::Float(lf * rf)),
                    '/' => {
                        if rf == 0.0 {
                            ExecResult::Fault("Division by zero".to_string())
                        } else {
                            ExecResult::Value(RelType::Float(lf / rf))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (ExecResult::Value(RelType::Int(li)), ExecResult::Value(RelType::Float(rf))) => {
                let lf = li as f64;
                match op {
                    '+' => ExecResult::Value(RelType::Float(lf + rf)),
                    '-' => ExecResult::Value(RelType::Float(lf - rf)),
                    '*' => ExecResult::Value(RelType::Float(lf * rf)),
                    '/' => {
                        if rf == 0.0 {
                            ExecResult::Fault("Division by zero".to_string())
                        } else {
                            ExecResult::Value(RelType::Float(lf / rf))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (ExecResult::Value(RelType::Float(lf)), ExecResult::Value(RelType::Int(ri))) => {
                let rf = ri as f64;
                match op {
                    '+' => ExecResult::Value(RelType::Float(lf + rf)),
                    '-' => ExecResult::Value(RelType::Float(lf - rf)),
                    '*' => ExecResult::Value(RelType::Float(lf * rf)),
                    '/' => {
                        if rf == 0.0 {
                            ExecResult::Fault("Division by zero".to_string())
                        } else {
                            ExecResult::Value(RelType::Float(lf / rf))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (ExecResult::Value(RelType::Str(ls)), ExecResult::Value(RelType::Str(rs))) => {
                if op == '+' {
                    ExecResult::Value(RelType::Str(format!("{}{}", ls, rs)))
                } else {
                    ExecResult::Fault("Invalid string operation".to_string())
                }
            }
            (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => ExecResult::Fault(err),
            (ExecResult::ReturnBlockInfo(v), _) | (_, ExecResult::ReturnBlockInfo(v)) => {
                ExecResult::ReturnBlockInfo(v)
            }
            _ => ExecResult::Fault("Mathematical type mismatch".to_string()),
        }
    }
}
