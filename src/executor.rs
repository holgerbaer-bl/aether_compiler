use crate::ast::Node;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RelType {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Array(Vec<RelType>),
    Function(Vec<String>, Box<Node>), // Parameters, Body Block
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
            RelType::Function(_, _) => write!(f, "<Function>"),
            RelType::Void => write!(f, "void"),
        }
    }
}

use std::sync::{Arc, Mutex};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

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

pub struct ExecutionEngine {
    pub memory: HashMap<String, RelType>,
    pub event_loop: Option<EventLoop<()>>,
    pub window: Option<Arc<Window>>,
    pub surface: Option<wgpu::Surface<'static>>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
    pub config: Option<wgpu::SurfaceConfiguration>,
    pub shaders: Vec<wgpu::ShaderModule>,
    pub render_pipelines: HashMap<usize, wgpu::RenderPipeline>,

    // Audio backend state
    pub voices: Option<Arc<Mutex<[VoiceState; 4]>>>,
    pub audio_stream: Option<cpal::Stream>,
}

pub enum ExecResult {
    Value(RelType),
    ReturnBlockInfo(RelType), // Explicit return triggered
    Fault(String),
}

impl ExecutionEngine {
    pub fn new() -> Self {
        Self {
            memory: HashMap::new(),
            event_loop: None,
            window: None,
            surface: None,
            device: None,
            queue: None,
            config: None,
            shaders: Vec::new(),
            render_pipelines: HashMap::new(),
            voices: None,
            audio_stream: None,
        }
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
                        RelType::Function(_, _) => format!("{} = <fn>", k),
                        _ => format!(
                            "{} = {}",
                            k,
                            match v {
                                RelType::Int(i) => i.to_string(),
                                RelType::Bool(b) => b.to_string(),
                                _ => unreachable!(),
                            }
                        ),
                    }
                })
                .collect();

            out.push_str(&mem_str.join(", "));
        }

        out
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
                if let Some(val) = self.memory.get(name) {
                    ExecResult::Value(val.clone())
                } else {
                    ExecResult::Fault("Undefined identifier".to_string())
                }
            }
            Node::Assign(name, expr_node) => match self.evaluate(expr_node) {
                ExecResult::Value(val) => {
                    self.memory.insert(name.clone(), val.clone());
                    ExecResult::Value(val)
                }
                ExecResult::ReturnBlockInfo(val) => {
                    self.memory.insert(name.clone(), val.clone());
                    ExecResult::Value(val)
                }
                fault => fault,
            },

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
                    (ExecResult::Value(l_val), ExecResult::Value(r_val)) => {
                        ExecResult::Value(RelType::Bool(l_val == r_val))
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid Eq semantics".to_string()),
                }
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
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid Lt semantics".to_string()),
                }
            }

            // Arrays & Strings
            Node::ArrayLiteral(items) => {
                let mut vals = Vec::new();
                for item in items {
                    match self.evaluate(item) {
                        ExecResult::Value(v) => vals.push(v),
                        fault => return fault,
                    }
                }
                ExecResult::Value(RelType::Array(vals))
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
                let func = RelType::Function(params.clone(), body.clone());
                self.memory.insert(name.clone(), func.clone());
                ExecResult::Value(func)
            }
            Node::Call(name, args) => {
                let func_val = match self.memory.get(name) {
                    Some(val) => val.clone(),
                    None => return ExecResult::Fault(format!("Undefined function '{}'", name)),
                };

                match func_val {
                    RelType::Function(params, body) => {
                        if args.len() != params.len() {
                            return ExecResult::Fault("Argument count mismatch".to_string());
                        }

                        let mut evaluated_args = Vec::new();
                        for arg in args {
                            match self.evaluate(arg) {
                                ExecResult::Value(v) => evaluated_args.push(v),
                                ExecResult::ReturnBlockInfo(v) => evaluated_args.push(v),
                                fault => return fault,
                            }
                        }

                        let old_memory = self.memory.clone();
                        for (i, p) in params.iter().enumerate() {
                            self.memory.insert(p.clone(), evaluated_args[i].clone());
                        }

                        let mut call_res = self.evaluate(&body);
                        if let ExecResult::ReturnBlockInfo(v) = call_res {
                            call_res = ExecResult::Value(v);
                        }

                        self.memory = old_memory; // Pop scope
                        call_res
                    }
                    _ => ExecResult::Fault(format!("Identifier '{}' is not a function", name)),
                }
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

            // FFI / Reflection
            Node::EvalBincodeNative(bytes_node) => match self.evaluate(bytes_node) {
                ExecResult::Value(RelType::Array(arr)) => {
                    let bytes: Vec<u8> = arr
                        .into_iter()
                        .map(|v| match v {
                            RelType::Int(i) => i as u8,
                            _ => 0,
                        })
                        .collect();

                    match bincode::deserialize::<Node>(&bytes) {
                        Ok(parsed) => {
                            let mut sub_engine = ExecutionEngine::new();
                            let output = sub_engine.execute(&parsed);
                            ExecResult::Value(RelType::Str(output))
                        }
                        Err(e) => ExecResult::Fault(format!("Bincode Native Eval Fault: {}", e)),
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
                    let event_loop = EventLoop::new().unwrap();
                    let window = WindowBuilder::new()
                        .with_inner_size(winit::dpi::LogicalSize::new(w as f64, h as f64))
                        .with_title(t)
                        .build(&event_loop)
                        .unwrap();
                    self.window = Some(Arc::new(window));
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
                        adapter.request_device(&wgpu::DeviceDescriptor::default(), None),
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

                    let static_surface = unsafe {
                        std::mem::transmute::<wgpu::Surface<'_>, wgpu::Surface<'static>>(surface)
                    };
                    self.surface = Some(static_surface);
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
                            label: Some("AetherShader"),
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
                                            entry_point: "vs_main",
                                            buffers: &[],
                                            compilation_options:
                                                wgpu::PipelineCompilationOptions::default(),
                                        },
                                        fragment: Some(wgpu::FragmentState {
                                            module: shader,
                                            entry_point: "fs_main",
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
            Node::PollEvents(body) => {
                if let Some(mut event_loop) = self.event_loop.take() {
                    use winit::platform::run_on_demand::EventLoopExtRunOnDemand;
                    let mut exit = false;
                    let _ = event_loop.run_on_demand(|event, elwt| {
                        elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);
                        match event {
                            winit::event::Event::WindowEvent {
                                event: winit::event::WindowEvent::CloseRequested,
                                ..
                            } => {
                                elwt.exit();
                                exit = true;
                            }
                            winit::event::Event::AboutToWait => {
                                let res = self.evaluate(body);
                                if let ExecResult::ReturnBlockInfo(_) | ExecResult::Fault(_) = res {
                                    elwt.exit();
                                }
                            }
                            _ => {}
                        }
                    });

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
                    let sample_rate = supported_config.sample_rate() as f32; // wait, if cpal changed this to u32, this will work.
                    let config = supported_config.config();
                    let channels = config.channels as usize;

                    let voices = Arc::new(Mutex::new([VoiceState::default(); 4]));
                    self.voices = Some(voices.clone());

                    let err_fn =
                        |err| eprintln!("An error occurred on the output audio stream: {}", err);

                    let stream = match supported_config.sample_format() {
                        cpal::SampleFormat::F32 => {
                            device
                                .build_output_stream(
                                    &config,
                                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                        let mut voices_lock = voices.lock().unwrap();
                                        for frame in data.chunks_mut(channels) {
                                            let mut sample: f32 = 0.0;
                                            for voice in voices_lock.iter_mut() {
                                                if voice.active {
                                                    // Advance phase
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
                                                    sample += v_sample * 0.15; // Volume scaling per voice
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
                            device
                                .build_output_stream(
                                    &config,
                                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                                        let mut voices_lock = voices.lock().unwrap();
                                        for frame in data.chunks_mut(channels) {
                                            let mut sample: f32 = 0.0;
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
                                                    sample += v_sample * 0.15; // Volume scaling per voice
                                                }
                                            }
                                            let int_sample = (sample.max(-1.0).min(1.0)
                                                * f32::from(std::i16::MAX))
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
                            device
                                .build_output_stream(
                                    &config,
                                    move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                                        let mut voices_lock = voices.lock().unwrap();
                                        for frame in data.chunks_mut(channels) {
                                            let mut sample: f32 = 0.0;
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
                                                    sample += v_sample * 0.15; // Volume scaling per voice
                                                }
                                            }
                                            let int_sample = ((sample.max(-1.0).min(1.0) * 0.5
                                                + 0.5)
                                                * f32::from(std::u16::MAX))
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
                            device
                                .build_output_stream(
                                    &config,
                                    move |data: &mut [u8], _: &cpal::OutputCallbackInfo| {
                                        let mut voices_lock = voices.lock().unwrap();
                                        for frame in data.chunks_mut(channels) {
                                            let mut sample: f32 = 0.0;
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
                                                    sample += v_sample * 0.15; // Volume scaling per voice
                                                }
                                            }
                                            let int_sample = ((sample.max(-1.0).min(1.0) * 0.5
                                                + 0.5)
                                                * f32::from(std::u8::MAX))
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
                    if c >= 0 && c < 4 {
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
                    if c >= 0 && c < 4 {
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
            Node::Return(val_node) => match self.evaluate(val_node) {
                ExecResult::Value(v) => ExecResult::ReturnBlockInfo(v),
                fault => fault,
            },
        }
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
            (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => ExecResult::Fault(err),
            _ => ExecResult::Fault("Mathematical type mismatch".to_string()),
        }
    }
}
