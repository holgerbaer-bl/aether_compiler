// Generates audio_test.aec
// Features: 3D Cube Rotation via Uniform matrices, and an 8-Bit C64 Arpeggio scale via CPAL Audio Engine.
use aether_compiler::ast::Node;

fn int(v: i64) -> Node {
    Node::IntLiteral(v)
}
fn float(v: f64) -> Node {
    Node::FloatLiteral(v)
}
fn var(n: &str) -> Node {
    Node::Identifier(n.to_string())
}
fn assign(n: &str, v: Node) -> Node {
    Node::Assign(n.to_string(), Box::new(v))
}
fn str_lit(s: &str) -> Node {
    Node::StringLiteral(s.to_string())
}
fn mul(a: Node, b: Node) -> Node {
    Node::Mul(Box::new(a), Box::new(b))
}
fn sub(a: Node, b: Node) -> Node {
    Node::Sub(Box::new(a), Box::new(b))
}
fn div(a: Node, b: Node) -> Node {
    Node::Div(Box::new(a), Box::new(b))
}
fn arr(v: Vec<Node>) -> Node {
    Node::ArrayLiteral(v)
}
fn mat4mul(a: Node, b: Node) -> Node {
    Node::Mat4Mul(Box::new(a), Box::new(b))
}

fn main() {
    let mut stmts = Vec::new();

    stmts.push(Node::InitWindow(
        Box::new(int(800)),
        Box::new(int(600)),
        Box::new(str_lit("AetherCore 3D & Audio (CPAL + WGPU)")),
    ));

    stmts.push(Node::InitGraphics);
    stmts.push(Node::InitAudio);

    let wgsl = r#"
        struct Uniforms {
            mvp: mat4x4<f32>,
        }
        @group(0) @binding(0) var<uniform> uniforms: Uniforms;

        struct VertexOutput {
            @builtin(position) position: vec4<f32>,
            @location(0) color: vec3<f32>,
        }

        var<private> POSITIONS: array<vec3<f32>, 36> = array<vec3<f32>, 36>(
            // Front
            vec3<f32>(-0.5, -0.5,  0.5), vec3<f32>( 0.5, -0.5,  0.5), vec3<f32>( 0.5,  0.5,  0.5),
            vec3<f32>(-0.5, -0.5,  0.5), vec3<f32>( 0.5,  0.5,  0.5), vec3<f32>(-0.5,  0.5,  0.5),
            // Back
            vec3<f32>(-0.5,  0.5, -0.5), vec3<f32>( 0.5,  0.5, -0.5), vec3<f32>( 0.5, -0.5, -0.5),
            vec3<f32>(-0.5,  0.5, -0.5), vec3<f32>( 0.5, -0.5, -0.5), vec3<f32>(-0.5, -0.5, -0.5),
            // Top
            vec3<f32>(-0.5,  0.5,  0.5), vec3<f32>( 0.5,  0.5,  0.5), vec3<f32>( 0.5,  0.5, -0.5),
            vec3<f32>(-0.5,  0.5,  0.5), vec3<f32>( 0.5,  0.5, -0.5), vec3<f32>(-0.5,  0.5, -0.5),
            // Bottom
            vec3<f32>(-0.5, -0.5, -0.5), vec3<f32>( 0.5, -0.5, -0.5), vec3<f32>( 0.5, -0.5,  0.5),
            vec3<f32>(-0.5, -0.5, -0.5), vec3<f32>( 0.5, -0.5,  0.5), vec3<f32>(-0.5, -0.5,  0.5),
            // Right
            vec3<f32>( 0.5, -0.5,  0.5), vec3<f32>( 0.5, -0.5, -0.5), vec3<f32>( 0.5,  0.5, -0.5),
            vec3<f32>( 0.5, -0.5,  0.5), vec3<f32>( 0.5,  0.5, -0.5), vec3<f32>( 0.5,  0.5,  0.5),
            // Left
            vec3<f32>(-0.5, -0.5, -0.5), vec3<f32>(-0.5, -0.5,  0.5), vec3<f32>(-0.5,  0.5,  0.5),
            vec3<f32>(-0.5, -0.5, -0.5), vec3<f32>(-0.5,  0.5,  0.5), vec3<f32>(-0.5,  0.5, -0.5)
        );

        var<private> COLORS: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
            vec3<f32>(1.0, 0.0, 0.0), // Front Red
            vec3<f32>(0.0, 1.0, 0.0), // Back Green
            vec3<f32>(0.0, 0.0, 1.0), // Top Blue
            vec3<f32>(1.0, 1.0, 0.0), // Bottom Yellow
            vec3<f32>(1.0, 0.0, 1.0), // Right Magenta
            vec3<f32>(0.0, 1.0, 1.0)  // Left Cyan
        );

        @vertex
        fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
            var out: VertexOutput;
            let pos = POSITIONS[in_vertex_index];
            out.position = uniforms.mvp * vec4<f32>(pos, 1.0);
            
            let color_idx = in_vertex_index / 6u;
            out.color = COLORS[color_idx];
            return out;
        }

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return vec4<f32>(in.color, 1.0);
        }
    "#;
    stmts.push(assign("shader", Node::LoadShader(Box::new(str_lit(wgsl)))));

    // Generate basic Projection Matrix Array [16] statically
    let fov_rad = std::f32::consts::FRAC_PI_4;
    let aspect = 800.0 / 600.0;
    let near = 0.1;
    let far = 100.0;

    let f = 1.0 / (fov_rad / 2.0).tan();
    let y_scale = f;
    let x_scale = f / aspect;
    let z_scale = far / (near - far);
    let z_trans = near * far / (near - far);

    let proj: Vec<Node> = vec![
        float(x_scale as f64),
        float(0.0),
        float(0.0),
        float(0.0),
        float(0.0),
        float(y_scale as f64),
        float(0.0),
        float(0.0),
        float(0.0),
        float(0.0),
        float(z_scale as f64),
        float(-1.0),
        float(0.0),
        float(0.0),
        float(z_trans as f64),
        float(0.0),
    ];
    let proj_node = arr(proj);

    // Translation matrix Z = -3.0
    let trans_z: Vec<Node> = vec![
        float(1.0),
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
        float(0.0),
        float(0.0),
        float(0.0),
        float(-3.0),
        float(1.0),
    ];
    let view_node = arr(trans_z);

    // Calculate (Proj * View) once before the loop
    stmts.push(assign("vp_matrix", mat4mul(proj_node, view_node)));

    // Begin render loop
    let mut loop_body = Vec::new();

    // t = Time()
    loop_body.push(assign("t", Node::Time));

    // Matrix Rotation logic
    let rot_t = mul(var("t"), float(1.5));
    let s = Node::Sin(Box::new(rot_t.clone()));
    let c = Node::Cos(Box::new(rot_t.clone()));

    let rot_y = arr(vec![
        c.clone(),
        float(0.0),
        mul(s.clone(), float(-1.0)),
        float(0.0),
        float(0.0),
        float(1.0),
        float(0.0),
        float(0.0),
        s.clone(),
        float(0.0),
        c.clone(),
        float(0.0),
        float(0.0),
        float(0.0),
        float(0.0),
        float(1.0),
    ]);

    let rot_x = arr(vec![
        float(1.0),
        float(0.0),
        float(0.0),
        float(0.0),
        float(0.0),
        c.clone(),
        s.clone(),
        float(0.0),
        float(0.0),
        mul(float(-1.0), s.clone()),
        c.clone(),
        float(0.0),
        float(0.0),
        float(0.0),
        float(0.0),
        float(1.0),
    ]);

    loop_body.push(assign("model_matrix", mat4mul(rot_y, rot_x)));

    loop_body.push(assign(
        "mvp_matrix",
        mat4mul(var("vp_matrix"), var("model_matrix")),
    ));

    let render_mesh = Node::RenderMesh(
        Box::new(var("shader")),
        Box::new(arr(vec![])), // Dummy vertex
        Box::new(var("mvp_matrix")),
    );
    loop_body.push(render_mesh);

    // Audio Arpeggio Logic: Plays a C Minor Arpeggio
    // Array of notes: C4 (261.63), Eb4 (311.13), G4 (392.00), C5 (523.25)
    let arp = arr(vec![
        float(261.63),
        float(311.13),
        float(392.00),
        float(523.25),
    ]);
    loop_body.push(assign("arp", arp));

    // Note speed 15hz (approx 16th notes at 120bpm)
    // index = int(t * 15.0) % 4
    let t_scale = sub(mul(var("t"), float(15.0)), float(0.5)); // Poor man's floor
    let arp_index = sub(
        t_scale.clone(),
        mul(div(t_scale.clone(), float(4.0)), float(4.0)),
    ); // Basic float modulo

    // Play Note on Channel 0, Square Wave (1)
    let freq = Node::Index(Box::new(var("arp")), Box::new(int(0))); // Fallback for pure AST modulo: we just pick directly via bitmagic, or simple array.
    // wait, our aether array doesn't support Float indexes directly for lookup...
    // We should compute arp_index as an Int for reliability.
    // Just using Int cast would require native cast support, we can just do discrete conditionals for now.

    let cycle = sub(
        sub(mul(var("t"), float(15.0)), float(0.5)),
        mul(div(mul(var("t"), float(15.0)), float(4.0)), float(4.0)),
    );

    loop_body.push(assign("cycle", cycle));

    // AetherCore Float array indexing is not strictly castable to Int yet implicitly! Let's just use IF block tree.
    let arp_tree = Node::If(
        Box::new(Node::Lt(Box::new(var("cycle")), Box::new(float(1.0)))),
        Box::new(Node::PlayNote(
            Box::new(int(0)),
            Box::new(float(261.63)),
            Box::new(int(1)),
        )),
        Some(Box::new(Node::If(
            Box::new(Node::Lt(Box::new(var("cycle")), Box::new(float(2.0)))),
            Box::new(Node::PlayNote(
                Box::new(int(0)),
                Box::new(float(311.13)),
                Box::new(int(1)),
            )),
            Some(Box::new(Node::If(
                Box::new(Node::Lt(Box::new(var("cycle")), Box::new(float(3.0)))),
                Box::new(Node::PlayNote(
                    Box::new(int(0)),
                    Box::new(float(392.00)),
                    Box::new(int(1)),
                )),
                Some(Box::new(Node::PlayNote(
                    Box::new(int(0)),
                    Box::new(float(523.25)),
                    Box::new(int(1)),
                ))),
            ))),
        ))),
    );

    loop_body.push(arp_tree);

    stmts.push(Node::PollEvents(Box::new(Node::Block(loop_body))));

    let ast = Node::Block(stmts);
    let bytes = bincode::serialize(&ast).unwrap();
    let dest_path = "target/audio_test.aec";
    std::fs::create_dir_all("target").unwrap();
    std::fs::write(dest_path, &bytes).unwrap();
    println!(
        "Successfully generated AetherCore GPU & Audio demo AST at {}!",
        dest_path
    );
}
