use aether_compiler::ast::Node;

fn int(v: i64) -> Node {
    Node::IntLiteral(v)
}
fn str_lit(s: &str) -> Node {
    Node::StringLiteral(s.to_string())
}

fn main() {
    let mut stmts = Vec::new();

    // 1. Initialize OS Window
    stmts.push(Node::InitWindow(
        Box::new(int(800)),
        Box::new(int(600)),
        Box::new(str_lit("AetherCore Hardware-Accelerated 3D AST")),
    ));

    // 2. Initialize WGPU Context (Adapter, Device, Queue, Surface)
    stmts.push(Node::InitGraphics);

    // 3. Load WGSL Shader payload
    let wgsl = r#"
        @vertex
        fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
            let x = f32(1 - i32(in_vertex_index)) * 0.5;
            let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
            return vec4<f32>(x, y, 0.0, 1.0);
        }

        @fragment
        fn fs_main() -> @location(0) vec4<f32> {
            return vec4<f32>(1.0, 0.0, 0.5, 1.0); // Vibrant Pink Color
        }
    "#;
    stmts.push(Node::Assign(
        "shader".to_string(),
        Box::new(Node::LoadShader(Box::new(str_lit(wgsl)))),
    ));

    // 4. Begin AetherCore infinite Application Loop
    let render_mesh = Node::RenderMesh(
        Box::new(Node::Identifier("shader".to_string())),
        Box::new(Node::ArrayLiteral(vec![])), // Dummy vertex array, using SV_VertexID inside WGSL
        Box::new(Node::ArrayLiteral(vec![])), // Empty uniform payload for old demo
    );

    stmts.push(Node::PollEvents(Box::new(Node::Block(vec![render_mesh]))));

    let ast = Node::Block(stmts);

    let bytes = bincode::serialize(&ast).unwrap();
    let dest_path = "target/3d_demo.aec";
    std::fs::create_dir_all("target").unwrap();
    std::fs::write(dest_path, &bytes).unwrap();
    println!(
        "Successfully generated AetherCore GPU demo AST at {}!",
        dest_path
    );
}
