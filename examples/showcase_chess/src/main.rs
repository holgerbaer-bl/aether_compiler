use knoten_core::ast::Node;
use knoten_core::executor::ExecutionEngine;
use serde_json;

fn lit_int(v: i64) -> Node { Node::IntLiteral(v) }
fn lit_str(v: &str) -> Node { Node::StringLiteral(v.to_string()) }
fn lit_float(v: f64) -> Node { Node::FloatLiteral(v) }
fn get_id(name: &str) -> Node { Node::Identifier(name.to_string()) }
fn assign(name: &str, ex: Node) -> Node { Node::Assign(name.to_string(), Box::new(ex)) }

fn rgba(r: f64, g: f64, b: f64, a: f64) -> Node {
    Node::ArrayCreate(vec![lit_float(r), lit_float(g), lit_float(b), lit_float(a)])
}

// ── Shiny neon cyberpunk board colors ─────────────────────────────────────────
// Dark tile:  deep space indigo (#1b1f4a)
const D: (f64, f64, f64) = (0.10, 0.12, 0.30);
// Light tile: silver-blue (#d8dff2)
const L: (f64, f64, f64) = (0.85, 0.88, 0.95);
// Selected:   neon gold (#f0cc00)
const S: (f64, f64, f64) = (0.94, 0.80, 0.05);
// Hover dark: electric indigo lit up
const HD: (f64, f64, f64) = (0.22, 0.30, 0.55);
// Hover light: bright silver
const HL: (f64, f64, f64) = (0.70, 0.75, 0.92);

fn tile_style_node(idle_r: f64, idle_g: f64, idle_b: f64,
                   hover_r: f64, hover_g: f64, hover_b: f64) -> Node {
    Node::UISetStyle(
        Box::new(lit_float(0.0)),                        // 0 rounding = sharp chess squares
        Box::new(lit_float(1.0)),                        // tight spacing
        Box::new(rgba(0.0, 0.9, 0.7, 1.0)),             // unused accent
        Box::new(rgba(0.0, 0.0, 0.0, 0.0)),             // transparent fill
        Some(Box::new(rgba(idle_r, idle_g, idle_b, 1.0))),
        Some(Box::new(rgba(hover_r, hover_g, hover_b, 1.0))),
    )
}

/// Build the complete 8×8 chess board as 8 UIHorizontal rows of 8 UIButton cells.
/// Each tile has a real background color via UISetStyle — no ASCII, no text grid.
fn build_native_board() -> Vec<Node> {
    let mut rows: Vec<Node> = Vec::new();

    for row in 0..8i64 {
        let mut cells: Vec<Node> = Vec::new();

        for col in 0..8i64 {
            let i = row * 8 + col;
            let is_dark = (row + col) % 2 == 1;
            let (ir, ig, ib) = if is_dark { D } else { L };
            let (hr, hg, hb) = if is_dark { HD } else { HL };

            let get_piece = Node::ArrayGet(
                Box::new(get_id("board_state")),
                Box::new(lit_int(i)),
            );
            let is_selected = Node::Eq(
                Box::new(get_id("selected_index")),
                Box::new(lit_int(i)),
            );

            // Tile text: newlines give the button visual height so tiles look square-ish
            let piece_text = Node::Concat(
                Box::new(lit_str("\n")),
                Box::new(Node::Concat(
                    Box::new(get_piece.clone()),
                    Box::new(lit_str("\n")),
                )),
            );

            // Style: golden when selected, board color otherwise — set unconditionally  
            // (we set both variants and let the If pick, but we apply style before button draw)
            let normal_style = tile_style_node(ir, ig, ib, hr, hg, hb);
            let selected_style = tile_style_node(S.0, S.1, S.2, 1.0, 0.90, 0.20);

            // Dynamic style selection using Node::If
            let style = Node::If(
                Box::new(is_selected.clone()),
                Box::new(selected_style),
                Some(Box::new(normal_style)),
            );

            let button = Node::UIButton(Box::new(piece_text));

            // Click: select or move
            let click_logic = Node::If(
                Box::new(Node::Eq(Box::new(get_id("selected_index")), Box::new(lit_int(-1)))),
                Box::new(Node::If(
                    Box::new(Node::Eq(Box::new(get_piece.clone()), Box::new(lit_str(" ")))),
                    Box::new(Node::Block(vec![])),
                    Some(Box::new(assign("selected_index", lit_int(i)))),
                )),
                Some(Box::new(Node::Block(vec![
                    Node::ArraySet(
                        Box::new(get_id("board_state")),
                        Box::new(lit_int(i)),
                        Box::new(Node::ArrayGet(
                            Box::new(get_id("board_state")),
                            Box::new(get_id("selected_index")),
                        )),
                    ),
                    Node::ArraySet(
                        Box::new(get_id("board_state")),
                        Box::new(get_id("selected_index")),
                        Box::new(lit_str(" ")),
                    ),
                    assign("selected_index", lit_int(-1)),
                    assign("turn", Node::If(
                        Box::new(Node::Eq(Box::new(get_id("turn")), Box::new(lit_int(0)))),
                        Box::new(lit_int(1)),
                        Some(Box::new(lit_int(0))),
                    )),
                    Node::Store { key: "chess_board".to_string(), value: Box::new(get_id("board_state")) },
                    Node::Store { key: "chess_turn".to_string(), value: Box::new(get_id("turn")) },
                ]))),
            );

            // Only clickable on player's turn (turn == 0)
            let gated_click = Node::If(
                Box::new(Node::Eq(Box::new(get_id("turn")), Box::new(lit_int(0)))),
                Box::new(Node::If(Box::new(button), Box::new(click_logic), None)),
                // AI's turn: still draw the button (for display) but don't process clicks
                Some(Box::new(Node::Block(vec![
                    Node::UIButton(Box::new(Node::Concat(
                        Box::new(lit_str("\n")),
                        Box::new(Node::Concat(
                            Box::new(Node::ArrayGet(
                                Box::new(get_id("board_state")),
                                Box::new(lit_int(i)),
                            )),
                            Box::new(lit_str("\n")),
                        )),
                    ))),
                ]))),
            );

            cells.push(Node::Block(vec![style, gated_click]));
        }

        rows.push(Node::UIHorizontal(Box::new(Node::Block(cells))));
    }

    rows
}

/// Replace UIGrid("chess_board_grid") with 8 UIHorizontal rows of native colored tiles.
fn inject_native_board(ast: &mut Node) {
    match ast {
        Node::UIGrid(_, id, _) if id == "chess_board_grid" => {
            println!(">> Replacing UIGrid '{}' → native 8×8 UIHorizontal rows", id);
            let board_rows = build_native_board();
            *ast = Node::Block(board_rows);
        }
        Node::Block(nodes) => { for n in nodes.iter_mut() { inject_native_board(n); } }
        Node::PollEvents(body) => inject_native_board(body),
        Node::While(_, body) => inject_native_board(body),
        Node::If(_, then_b, else_b) => {
            inject_native_board(then_b);
            if let Some(e) = else_b { inject_native_board(e); }
        }
        Node::UIWindow(_, _, body) => inject_native_board(body),
        Node::UIHorizontal(body) => inject_native_board(body),
        Node::UIFullscreen(body) => inject_native_board(body),
        Node::UIScrollArea(_, body) => inject_native_board(body),
        _ => {}
    }
}

fn run() {
    println!("Loading chess_showcase.nod (embedded at compile time)...");
    // AST is baked into the binary — no external file needed at runtime.
    let json_string = include_str!("../../../examples/graphics/chess_showcase.nod");
    let mut ast: Node = serde_json::from_str(json_string)
        .expect("Failed to parse embedded chess_showcase.nod");

    println!("Injecting native graphical board (8×8 UIHorizontal rows)...");
    inject_native_board(&mut ast);

    println!("Starting KnotenCore Engine [Human vs. Computer mode]...");
    let mut engine = ExecutionEngine::new();
    engine.permissions.allow_fs_read = true;
    engine.permissions.allow_fs_write = true;

    let result = engine.execute(&ast);
    if result.starts_with("Fault:") {
        eprintln!("❌ FAULT: {}", result);
        std::process::exit(1);
    } else {
        println!("✅ Exited: {}", result);
    }
}

fn main() {
    // 32 MB stack for deep recursive AST.
    // KnotenCore uses winit `any_thread` (Sprint 62) so EventLoop works from this thread.
    let builder = std::thread::Builder::new()
        .name("knoten-runtime".to_string())
        .stack_size(32 * 1024 * 1024);
    let handler = builder.spawn(run).expect("Failed to spawn runtime");
    handler.join().unwrap();
}
