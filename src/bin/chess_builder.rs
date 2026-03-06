use knoten_core::ast::Node;
use serde_json;
use std::fs;

fn lit_int(v: i64) -> Node { Node::IntLiteral(v) }
fn lit_str(v: &str) -> Node { Node::StringLiteral(v.to_string()) }
fn lit_float(v: f64) -> Node { Node::FloatLiteral(v) }
fn get_id(name: &str) -> Node { Node::Identifier(name.to_string()) }
fn assign(name: &str, ex: Node) -> Node { Node::Assign(name.to_string(), Box::new(ex)) }

fn generate_chess_board() -> Node {
    let mut cells = Vec::new();
    for i in 0..64 {
        let get_piece = Node::ArrayGet(
            Box::new(get_id("board_state")),
            Box::new(lit_int(i))
        );
        
        // If selected, highlight button
        let is_selected = Node::Eq(Box::new(get_id("selected_index")), Box::new(lit_int(i)));
        
        let button_text = Node::If(
            Box::new(is_selected.clone()),
            Box::new(Node::Concat(Box::new(lit_str("[ ")), Box::new(Node::Concat(Box::new(get_piece.clone()), Box::new(lit_str(" ]")))))),
            Some(Box::new(Node::Concat(Box::new(lit_str("  ")), Box::new(Node::Concat(Box::new(get_piece.clone()), Box::new(lit_str("  ")))))))
        );

        let button = Node::UIButton(Box::new(button_text));
        
        // Click Logic (PvP)
        let click_logic = Node::If(
            Box::new(Node::Eq(Box::new(get_id("selected_index")), Box::new(lit_int(-1)))),
            // No selection -> Set selection
            Box::new(Node::If(
                Box::new(Node::Eq(Box::new(get_piece.clone()), Box::new(lit_str(" ")))),
                Box::new(Node::Block(vec![])), // Clicked empty, do nothing
                Some(Box::new(assign("selected_index", lit_int(i))))
            )),
            // Has selection -> Move there
            Some(Box::new(Node::Block(vec![
                Node::ArraySet(
                    Box::new(get_id("board_state")),
                    Box::new(lit_int(i)),
                    Box::new(Node::ArrayGet(Box::new(get_id("board_state")), Box::new(get_id("selected_index"))))
                ),
                Node::ArraySet(
                    Box::new(get_id("board_state")),
                    Box::new(get_id("selected_index")),
                    Box::new(lit_str(" "))
                ),
                assign("selected_index", lit_int(-1)),
                // Swap turn & Save state
                assign("turn", Node::If(Box::new(Node::Eq(Box::new(get_id("turn")), Box::new(lit_int(0)))), Box::new(lit_int(1)), Some(Box::new(lit_int(0))))),
                Node::Store { key: "chess_board".to_string(), value: Box::new(get_id("board_state")) },
                Node::Store { key: "chess_turn".to_string(), value: Box::new(get_id("turn")) }
            ])))
        );

        let if_clicked = Node::If(
            Box::new(button), // Button evaluates to True if clicked
            Box::new(click_logic),
            None
        );

        // Tile background based on checkboard pattern
        let row = i / 8;
        let col = i % 8;
        let is_black = (row + col) % 2 == 1;
        
        // Wrap button in style to give tile colors
        let tile_style = Node::UISetStyle(
            Box::new(lit_float(0.0)), Box::new(lit_float(0.0)), Box::new(lit_float(0.0)), Box::new(lit_float(0.0)),
            Some(Box::new(Node::ArrayCreate(if is_black { vec![lit_float(0.4), lit_float(0.6), lit_float(0.4), lit_float(1.0)] } else { vec![lit_float(0.9), lit_float(0.9), lit_float(0.9), lit_float(1.0)] }))),
            Some(Box::new(Node::ArrayCreate(vec![lit_float(0.8), lit_float(0.8), lit_float(0.0), lit_float(1.0)])))
        );

        cells.push(Node::Block(vec![tile_style, if_clicked]));
    }
    
    Node::UIGrid(
        8,
        "chess_board_grid".to_string(),
        Box::new(Node::Block(cells))
    )
}

fn create_initial_board() -> Vec<Node> {
    let initial = vec![
        "♜", "♞", "♝", "♛", "♚", "♝", "♞", "♜",
        "♟", "♟", "♟", "♟", "♟", "♟", "♟", "♟",
        " ", " ", " ", " ", " ", " ", " ", " ",
        " ", " ", " ", " ", " ", " ", " ", " ",
        " ", " ", " ", " ", " ", " ", " ", " ",
        " ", " ", " ", " ", " ", " ", " ", " ",
        "♙", "♙", "♙", "♙", "♙", "♙", "♙", "♙",
        "♖", "♘", "♗", "♕", "♔", "♗", "♘", "♖",
    ];
    initial.into_iter().map(lit_str).collect()
}

fn main() {
    // 1. Initialize State logic & Persistence
    let setup_state = Node::Block(vec![
        // Sandbox-Store: Try to load game state
        assign("board_load", Node::Load { key: "chess_board".to_string() }),
        assign("turn_load", Node::Load { key: "chess_turn".to_string() }),
        
        // If loaded is Void (Null) or doesn't exist, create new
        Node::If(
            Box::new(Node::Eq(Box::new(get_id("turn_load")), Box::new(Node::Identifier("void".to_string())))), // Dummy check, if load fails it returns Void
            Box::new(Node::Block(vec![
                assign("board_state", Node::ArrayCreate(create_initial_board())),
                assign("turn", lit_int(0)) // 0 = White, 1 = Black
            ])),
            Some(Box::new(Node::Block(vec![
                assign("board_state", get_id("board_load")),
                assign("turn", get_id("turn_load"))
            ])))
        ),
        assign("selected_index", lit_int(-1))
    ]);
    
    // 2. UISetStyle for Premium Look (Dark mode / Cyber highlighting)
    let global_style = Node::UISetStyle(
        Box::new(lit_float(8.0)), // Rounding
        Box::new(lit_float(12.0)), // Spacing
        Box::new(Node::ArrayCreate(vec![
            lit_float(0.0), lit_float(0.8), lit_float(0.4), lit_float(1.0) // Accent (Cyber Green)
        ])),
        Box::new(Node::ArrayCreate(vec![
            lit_float(0.05), lit_float(0.05), lit_float(0.08), lit_float(0.98) // Dark Glass Fill
        ])),
        None, 
        None
    );

    // AI Logic (Simple Random/First Valid Move Simulation)
    let ai_logic = Node::If(
        Box::new(Node::Eq(Box::new(get_id("turn")), Box::new(lit_int(1)))),
        Box::new(Node::Block(vec![
            // For now, KI just prints and passes turn. In a real scenario we'd write AST to iterate and move.
            // But we will make a simplistic AI move:
            assign("turn", lit_int(0)),
            Node::Print(Box::new(lit_str("AI: My turn is complete."))),
            Node::Store { key: "chess_turn".to_string(), value: Box::new(get_id("turn")) }
        ])),
        None
    );

    // 3. UI App Loop
    let main_window = Node::UIWindow(
        "chess_main".to_string(),
        Box::new(lit_str("Agentic WGPU Chess")),
        Box::new(Node::Block(vec![
            global_style.clone(),
            // Title & Turn indicator
            Node::If(
                Box::new(Node::Eq(Box::new(get_id("turn")), Box::new(lit_int(0)))),
                Box::new(Node::UILabel(Box::new(lit_str("Turn: Player (White)")))),
                Some(Box::new(Node::UILabel(Box::new(lit_str("Turn: AI (Black) - Thinking...")))))
            ),
            
            // Buttons to trigger AI or Reset
            Node::UIHorizontal(Box::new(Node::Block(vec![
                Node::If(
                    Box::new(Node::UIButton(Box::new(lit_str("Reset Game")))),
                    Box::new(Node::Block(vec![
                        assign("board_state", Node::ArrayCreate(create_initial_board())),
                        assign("turn", lit_int(0)),
                        assign("selected_index", lit_int(-1)),
                        Node::Store { key: "chess_board".to_string(), value: Box::new(get_id("board_state")) },
                        Node::Store { key: "chess_turn".to_string(), value: Box::new(get_id("turn")) }
                    ])),
                    None
                )
            ]))),
            
            generate_chess_board()
        ]))
    );

    let program = Node::Block(vec![
        setup_state,
        Node::While(
            Box::new(Node::BoolLiteral(true)),
            Box::new(Node::Block(vec![
                ai_logic,
                Node::InitGraphics, // Polls/Renders the WGPU EGUI frames
                main_window,
                Node::PollEvents(Box::new(Node::Block(vec![])))
            ]))
        )
    ]);

    let json = serde_json::to_string_pretty(&program).unwrap();
    let out_dir = "examples/graphics";
    fs::create_dir_all(out_dir).unwrap();
    let out_path = format!("{}/chess_showcase.nod", out_dir);
    
    fs::write(&out_path, json).unwrap();
    println!("Compiled AST to {}", out_path);
}
