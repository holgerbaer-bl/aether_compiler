// Generates c64_demo.aec
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
fn add(a: Node, b: Node) -> Node {
    Node::Add(Box::new(a), Box::new(b))
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
        Box::new(str_lit("AetherCore - C64 Demoscene")),
    ));

    stmts.push(Node::InitGraphics);
    stmts.push(Node::InitAudio);

    let wgsl = r#"
        struct Uniforms {
            mvp: mat4x4<f32>,
            time: vec4<f32>,
        }
        @group(0) @binding(0) var<uniform> uniforms: Uniforms;

        struct VertexOutput {
            @builtin(position) position: vec4<f32>,
            @location(0) color: vec3<f32>,
            @location(1) is_bg: f32,
        }

        var<private> POSITIONS: array<vec3<f32>, 36> = array<vec3<f32>, 36>(
            // 0..5: Background Quad (Fullscreen - rendering at fixed Z in clip space inside the shader)
            vec3<f32>(-1.0, -1.0, 0.999), vec3<f32>( 1.0, -1.0, 0.999), vec3<f32>(-1.0,  1.0, 0.999),
            vec3<f32>( 1.0, -1.0, 0.999), vec3<f32>( 1.0,  1.0, 0.999), vec3<f32>(-1.0,  1.0, 0.999),
            
            // 6..23: 3D Pyramid (18 vertices => 1 square base + 4 triangle sides)
            // Base (-)
            vec3<f32>(-0.5, -0.5, -0.5), vec3<f32>( 0.5, -0.5, -0.5), vec3<f32>( 0.5, -0.5,  0.5),
            vec3<f32>(-0.5, -0.5, -0.5), vec3<f32>( 0.5, -0.5,  0.5), vec3<f32>(-0.5, -0.5,  0.5),
            // Front Side
            vec3<f32>(-0.5, -0.5,  0.5), vec3<f32>( 0.5, -0.5,  0.5), vec3<f32>( 0.0,  0.5,  0.0),
            // Back Side
            vec3<f32>( 0.5, -0.5, -0.5), vec3<f32>(-0.5, -0.5, -0.5), vec3<f32>( 0.0,  0.5,  0.0),
            // Left Side
            vec3<f32>(-0.5, -0.5, -0.5), vec3<f32>(-0.5, -0.5,  0.5), vec3<f32>( 0.0,  0.5,  0.0),
            // Right Side
            vec3<f32>( 0.5, -0.5,  0.5), vec3<f32>( 0.5, -0.5, -0.5), vec3<f32>( 0.0,  0.5,  0.0),

            // 24..35: Degenerate triangles (Invisible padding to match 36 vertex draw call exactly)
            vec3<f32>(0.0,0.0,0.0), vec3<f32>(0.0,0.0,0.0), vec3<f32>(0.0,0.0,0.0),
            vec3<f32>(0.0,0.0,0.0), vec3<f32>(0.0,0.0,0.0), vec3<f32>(0.0,0.0,0.0),
            vec3<f32>(0.0,0.0,0.0), vec3<f32>(0.0,0.0,0.0), vec3<f32>(0.0,0.0,0.0),
            vec3<f32>(0.0,0.0,0.0), vec3<f32>(0.0,0.0,0.0), vec3<f32>(0.0,0.0,0.0)
        );

        var<private> PALETTE: array<vec3<f32>, 16> = array<vec3<f32>, 16>(
            vec3<f32>(0.00, 0.00, 0.00), // 0 Black
            vec3<f32>(1.00, 1.00, 1.00), // 1 White
            vec3<f32>(0.53, 0.00, 0.00), // 2 Red
            vec3<f32>(0.66, 1.00, 0.93), // 3 Cyan
            vec3<f32>(0.80, 0.26, 0.80), // 4 Purple
            vec3<f32>(0.00, 0.80, 0.33), // 5 Green
            vec3<f32>(0.00, 0.00, 0.66), // 6 Blue
            vec3<f32>(0.93, 0.93, 0.46), // 7 Yellow
            vec3<f32>(0.86, 0.53, 0.33), // 8 Orange
            vec3<f32>(0.40, 0.26, 0.00), // 9 Brown
            vec3<f32>(1.00, 0.46, 0.46), // 10 Light Red
            vec3<f32>(0.20, 0.20, 0.20), // 11 Dark Grey
            vec3<f32>(0.46, 0.46, 0.46), // 12 Grey
            vec3<f32>(0.66, 1.00, 0.40), // 13 Light Green
            vec3<f32>(0.00, 0.53, 1.00), // 14 Light Blue
            vec3<f32>(0.73, 0.73, 0.73)  // 15 Light Grey
        );

        fn get_c64_color(col: vec3<f32>) -> vec3<f32> {
            var best_dist: f32 = 100000.0;
            var best_col: vec3<f32> = PALETTE[0];
            for (var i: u32 = 0u; i < 16u; i = i + 1u) {
                let p = PALETTE[i];
                let d = distance(col, p);
                if (d < best_dist) {
                    best_dist = d;
                    best_col = p;
                }
            }
            return best_col;
        }

        @vertex
        fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
            var out: VertexOutput;
            let pos = POSITIONS[in_vertex_index];
            
            if (in_vertex_index < 6u) {
                // Background Quad bypasses MVP completely to stay fullscreen and fixed behind everything
                out.position = vec4<f32>(pos, 1.0);
                out.is_bg = 1.0;
                out.color = vec3<f32>(0.0);
            } else if (in_vertex_index < 24u) {
                // 3D Pyramid affected by MVP
                out.position = uniforms.mvp * vec4<f32>(pos, 1.0);
                out.is_bg = 0.0;
                
                // Color variation by faces
                if (in_vertex_index < 12u) {
                    out.color = PALETTE[4]; // Purple Base
                } else if (in_vertex_index < 15u) {
                    out.color = PALETTE[3]; // Cyan Front
                } else if (in_vertex_index < 18u) {
                    out.color = PALETTE[14]; // Light Blue Back
                } else if (in_vertex_index < 21u) {
                    out.color = PALETTE[7]; // Yellow Left
                } else {
                    out.color = PALETTE[10]; // Light Red Right
                }
            } else {
                // Degenerate
                out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
                out.is_bg = 0.0;
                out.color = vec3<f32>(0.0);
            }
            
            return out;
        }

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            var out_col = in.color;
            let chunky_y = floor(in.position.y / 6.0) * 6.0; // Rasterbar chunk size
            let t = uniforms.time.x;
            
            if (in.is_bg > 0.5) {
                // Raster Bars calculation
                let v = sin(chunky_y * 0.015 + t * 4.0) * cos(chunky_y * 0.01 - t * 2.0);
                // Map to palette index 0 to 15
                let idx = u32(abs(v) * 15.0) % 16u;
                out_col = PALETTE[idx];
            } else {
                // Foreground Object gets shaded slightly with height or just directly paletted
                // Basic fixed directional fake lighting using world y
                let shade = clamp(1.0 - (in.position.z * 0.001), 0.0, 1.0);
                out_col = get_c64_color(in.color * shade);
            }
            
            return vec4<f32>(out_col, 1.0);
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

    // Initial View/Translation statically evaluated later using dynamic trans offsets
    stmts.push(assign("proj_matrix", proj_node));

    // Begin render loop
    let mut loop_body = Vec::new();

    // t = Time()
    loop_body.push(assign("t", Node::Time));

    // Lissajous curve transform logic for View matrix translations!
    // trans_x = sin(t * 1.5) * 2.5
    // trans_y = cos(t * 2.1) * 1.5
    // trans_z = -4.0 + sin(t * 0.8) * 1.0
    let trans_x = mul(Node::Sin(Box::new(mul(var("t"), float(1.5)))), float(2.5));
    let trans_y = mul(Node::Cos(Box::new(mul(var("t"), float(2.1)))), float(1.5));
    let trans_z = add(
        float(-4.0),
        mul(Node::Sin(Box::new(mul(var("t"), float(0.8)))), float(1.0)),
    );

    let view_dyn = arr(vec![
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
        trans_x,
        trans_y,
        trans_z,
        float(1.0),
    ]);
    loop_body.push(assign("view_matrix", view_dyn));
    loop_body.push(assign(
        "vp_matrix",
        mat4mul(var("proj_matrix"), var("view_matrix")),
    ));

    // Matrix Rotation logic (Spinning Pyramid)
    let rot_t = mul(var("t"), float(2.0));
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

    // Create 20-element uniform payload buffer matching Uniforms definition
    // vec4 layout: mvp(16 floats), time(4 floats)
    // Wait, we need to actually array-concat the Matrix floats with the Time float.
    // Instead of expanding AetherCore AST with `ArrayConcat`, we'll just evaluate MVP as a flat array natively, but we can't easily.
    // Oh, `Mat4Mul` returns a `RelType::Array(16 elements)`. `RenderMesh` natively accepts ONE array for uniform payload.
    // Since we need to pass Time, let's update `RenderMesh` in `executor.rs` to accept an arbitrarily large array?
    // In `executor.rs`, `uniform_val` MUST map directly to the wgpu buffer cast.
    // Since `AudioTest` just used `mvp_matrix` array.
    // Let's create an AST Node `ArrayConcat(Vec<Box<Node>>)` to merge multiple computed arrays, OR we could just update the shader to extract time from one of the matrix cells if we want to cheat!
    // CHEAT CODE: We can embed `Time` into `mvp[0][3]` because `m[0][3]` is usually 0.0 in a standard MVP affine projection (Wait, for projection `[0][3]` is 0, `[1][3]` is 0, `[2][3]` is -1, `[3][3]` is 0. So `[0][3]` is safely 0.0 and we can override it).
    // Or we can just calculate color animation by time!
    // But how to pass to shader?

    // Instead of cheating, a clean way in AetherCore is to add an AST Node `ArrayPush` or `ArrayConcat` if needed, but we don't have it natively.
    // Wait, the specification and `executor` don't have Array mutation!
    // CHEAT: WGSL uniform only binds 16 floats right now automatically if we pass `mvp_matrix`.
    // Wait! `mvp_matrix` creates an Array of 16 Floats.
    // Could we just set `mvp_matrix[3]`? There is no `SetIndex` node in AST, only `Assign` var.
    // Let's use `Uniform Cheat`: We'll overwrite the standard 'unused' component of the model matrix, like `m[1][3]` which is usually `0.0` or `m[0][3]` before the final matrix multiplication?
    // No, matrix cross-multiplication will mangle the Time value. We need it untouched.
    // If we multiply (Proj * View), the w components change.

    // Let's implement `ArrayConcat(Box<Node>, Box<Node>)` quickly in `AETHER_SPEC.md`, `ast.rs`, `executor.rs`, and `bootstrap_gen.rs`.
    // However, I can also just evaluate `Time` on CPU by evaluating `Time` globally and injecting it. Wait, `PollEvents` evaluates `loop_body` endlessly via winit.

    // Actually, `RenderMesh` takes `shader`, `vertexBuffer`, `uniformBuffer`.
    // If `uniformBuffer` is evaluated to an array, and the only array we can build dynamically easily consists of evaluated variables if we make a huge 20-element Array literal?
    // NO! AetherCore evaluates arguments of `ArrayLiteral`.
    // Example: `ArrayLiteral(vec![ Node::Index(mvp, 0), Node::Index(mvp, 1) ... Node::Index(mvp, 15), var("t"), float(0), float(0), float(0) ])`
    // We HAVE an `Index` node! We can extract the 16 floats out of `mvp_matrix` and construct a 20-element array natively in AST.

    let mut flat_uniforms = Vec::new();
    for i in 0..16 {
        flat_uniforms.push(Node::Index(Box::new(var("mvp_matrix")), Box::new(int(i))));
    }
    flat_uniforms.push(var("t")); // time
    flat_uniforms.push(float(0.0));
    flat_uniforms.push(float(0.0));
    flat_uniforms.push(float(0.0));

    loop_body.push(assign(
        "mvp_matrix",
        mat4mul(var("vp_matrix"), var("model_matrix")),
    ));
    loop_body.push(assign("uniform_payload", arr(flat_uniforms)));

    let render_mesh = Node::RenderMesh(
        Box::new(var("shader")),
        Box::new(arr(vec![])), // Dummy vertex
        Box::new(var("uniform_payload")),
    );
    loop_body.push(render_mesh);

    // Audio Sequence Logic: Bassline (Sawtooth, Channel 0) & Arpeggio (Square, Channel 1)

    // Bass Freqs (C3, G2, F2, C3)
    let bass_seq = arr(vec![
        float(130.81),
        float(98.00),
        float(87.31),
        float(130.81),
    ]);
    loop_body.push(assign("bass_seq", bass_seq));

    // Arp Freqs (C4, Eb4, G4, C5, Bb4, G4)
    let arp_seq = arr(vec![
        float(261.63),
        float(311.13),
        float(392.00),
        float(523.25),
        float(466.16),
        float(392.00),
    ]);
    loop_body.push(assign("arp_seq", arp_seq));

    // Bass Index: 1 note per second
    let t_bass = sub(var("t"), float(0.5));
    let bass_idx = sub(
        t_bass.clone(),
        mul(div(t_bass.clone(), float(4.0)), float(4.0)),
    );

    // Arp Index: 16th notes (15hz = ~120bpm)
    let t_arp = sub(mul(var("t"), float(15.0)), float(0.5));
    let arp_idx = sub(
        t_arp.clone(),
        mul(div(t_arp.clone(), float(6.0)), float(6.0)),
    );

    loop_body.push(assign("bass_idx", bass_idx));
    loop_body.push(assign("arp_idx", arp_idx));

    // AetherCore Float to conditional branch indexing
    // Build Bass Tree (0 to 3)
    let b_tree = Node::If(
        Box::new(Node::Lt(Box::new(var("bass_idx")), Box::new(float(1.0)))),
        Box::new(Node::PlayNote(
            Box::new(int(0)),
            Box::new(float(130.81)),
            Box::new(int(2)),
        )), // 2 = Saw
        Some(Box::new(Node::If(
            Box::new(Node::Lt(Box::new(var("bass_idx")), Box::new(float(2.0)))),
            Box::new(Node::PlayNote(
                Box::new(int(0)),
                Box::new(float(98.00)),
                Box::new(int(2)),
            )),
            Some(Box::new(Node::If(
                Box::new(Node::Lt(Box::new(var("bass_idx")), Box::new(float(3.0)))),
                Box::new(Node::PlayNote(
                    Box::new(int(0)),
                    Box::new(float(87.31)),
                    Box::new(int(2)),
                )),
                Some(Box::new(Node::PlayNote(
                    Box::new(int(0)),
                    Box::new(float(130.81)),
                    Box::new(int(2)),
                ))),
            ))),
        ))),
    );
    loop_body.push(b_tree);

    // Build Arp Tree (0 to 5)
    let mut arp_chain = Node::PlayNote(Box::new(int(1)), Box::new(float(392.00)), Box::new(int(1))); // 1 = Square
    let arp_freqs = vec![261.63, 311.13, 392.00, 523.25, 466.16, 392.00];

    for i in (0..5).rev() {
        arp_chain = Node::If(
            Box::new(Node::Lt(
                Box::new(var("arp_idx")),
                Box::new(float((i + 1) as f64)),
            )),
            Box::new(Node::PlayNote(
                Box::new(int(1)),
                Box::new(float(arp_freqs[i])),
                Box::new(int(1)),
            )),
            Some(Box::new(arp_chain)),
        );
    }
    loop_body.push(arp_chain);

    stmts.push(Node::PollEvents(Box::new(Node::Block(loop_body))));

    let ast = Node::Block(stmts);
    let bytes = bincode::serialize(&ast).unwrap();
    let dest_path = "target/c64_demo.aec";
    std::fs::create_dir_all("target").unwrap();
    std::fs::write(dest_path, &bytes).unwrap();
    println!("Successfully generated C64 Demoscene AST at {}!", dest_path);
}
