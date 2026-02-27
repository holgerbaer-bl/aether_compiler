use aether_compiler::ast::Node;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn int(i: i64) -> Node {
    Node::IntLiteral(i)
}

fn float(f: f64) -> Node {
    Node::FloatLiteral(f)
}

fn assign(name: &str, val: Node) -> Node {
    Node::Assign(name.to_string(), Box::new(val))
}

fn var(name: &str) -> Node {
    Node::Identifier(name.to_string())
}

fn call(name: &str, args: Vec<Node>) -> Node {
    Node::Call(name.to_string(), args)
}

fn array(items: Vec<Node>) -> Node {
    Node::ArrayLiteral(items)
}

fn block(nodes: Vec<Node>) -> Node {
    Node::Block(nodes)
}

fn main() {
    // 1. Generate Dummy Assets
    std::fs::create_dir_all("assets").unwrap();

    // Simple Quad OBJ
    let obj_data = "v -0.5 -0.5 0.0\nv 0.5 -0.5 0.0\nv 0.5 0.5 0.0\nv -0.5 0.5 0.0\n\
                    vt 0.0 1.0\nvt 1.0 1.0\nvt 1.0 0.0\nvt 0.0 0.0\n\
                    f 1/1 2/2 3/3\nf 1/1 3/3 4/4\n";
    std::fs::write("assets/quad.obj", obj_data).unwrap();

    // Checked Texture PNG
    let mut img = image::ImageBuffer::new(256, 256);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let is_white = ((x / 32) % 2) == ((y / 32) % 2);
        let color: u8 = if is_white { 255 } else { 50 };
        *pixel = image::Rgba([color, color, color, 255u8]);
    }
    img.save("assets/texture.png").unwrap();

    // 440Hz Sine Wav
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("assets/sound.wav", spec).unwrap();
    for t in 0..44100 {
        // 1 second
        let sample = (t as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin();
        let amplitude = std::i16::MAX as f32 * 0.5;
        writer.write_sample((sample * amplitude) as i16).unwrap();
    }
    writer.finalize().unwrap();

    // 2. Generate AetherCore AST Program
    let wgsl_shader = "
struct Uniforms {
    mvp: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = uniforms.mvp * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
";

    let program = block(vec![
        // Window Setup
        Node::InitWindow(
            Box::new(int(800)),
            Box::new(int(600)),
            Box::new(Node::StringLiteral(
                "AetherCore Asset Pipeline Demo".to_string(),
            )),
        ),
        Node::InitGraphics,
        Node::InitAudio,
        // Load Assets
        assign(
            "shader",
            Node::LoadShader(Box::new(Node::StringLiteral(wgsl_shader.to_string()))),
        ),
        assign(
            "mesh",
            Node::LoadMesh(Box::new(Node::StringLiteral("assets/quad.obj".to_string()))),
        ),
        assign(
            "tex",
            Node::LoadTexture(Box::new(Node::StringLiteral(
                "assets/texture.png".to_string(),
            ))),
        ),
        // Start Audio looping in background
        Node::PlayAudioFile(Box::new(Node::StringLiteral(
            "assets/sound.wav".to_string(),
        ))),
        // Execution Loop
        assign("time", float(0.0)),
        Node::PollEvents(Box::new(block(vec![
            assign(
                "time",
                Node::Add(Box::new(var("time")), Box::new(float(0.016))),
            ),
            // Generate MVP (simple rotation matrix)
            assign("c", Node::Cos(Box::new(var("time")))),
            assign("s", Node::Sin(Box::new(var("time")))),
            assign(
                "mvp",
                array(vec![
                    var("c"),
                    var("s"),
                    float(0.0),
                    float(0.0),
                    Node::Sub(Box::new(float(0.0)), Box::new(var("s"))),
                    var("c"),
                    float(0.0),
                    float(0.0),
                    float(0.0),
                    float(0.0),
                    float(1.0),
                    float(0.0),
                    float(0.0),
                    float(0.0),
                    float(0.0),
                    float(1.0),
                ]),
            ),
            // Render the asset
            Node::RenderAsset(
                Box::new(var("shader")),
                Box::new(var("mesh")),
                Box::new(var("tex")),
                Box::new(var("mvp")),
            ),
        ]))),
    ]);

    // 3. Compile Program
    let bin = bincode::serialize(&program).unwrap();
    let mut file = File::create("asset_demo.aec").unwrap();
    file.write_all(&bin).unwrap();

    println!(
        "Demo generator success: asset_demo.aec created. Run with: cargo run --bin run_aec asset_demo.aec"
    );
}
