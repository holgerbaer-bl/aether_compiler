use knoten_core::ast::Node;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("Building KnotenCore Standard Library...");

    // Create stdlib directory
    let mut stdlib_dir = std::env::current_dir().unwrap();
    stdlib_dir.push("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();

    // ---------------------------------------------------------
    // 1. array_utils.aec
    // ---------------------------------------------------------
    // Provide: Array.Contains(arr, element), Array.Max(arr), Array.Reverse(arr)

    let array_utils_ast = Node::Block(vec![
        // Array.Contains(arr, element)
        Node::FnDef(
            "Array.Contains".to_string(),
            vec!["arr".to_string(), "element".to_string()],
            Box::new(Node::Block(vec![
                Node::Assign("i".to_string(), Box::new(Node::IntLiteral(0))),
                Node::Assign("found".to_string(), Box::new(Node::BoolLiteral(false))),
                Node::Assign(
                    "len".to_string(),
                    Box::new(Node::ArrayLen("arr".to_string())),
                ),
                Node::While(
                    Box::new(Node::Lt(
                        Box::new(Node::Identifier("i".to_string())),
                        Box::new(Node::Identifier("len".to_string())),
                    )),
                    Box::new(Node::Block(vec![
                        Node::If(
                            Box::new(Node::Eq(
                                Box::new(Node::ArrayGet(
                                    "arr".to_string(),
                                    Box::new(Node::Identifier("i".to_string())),
                                )),
                                Box::new(Node::Identifier("element".to_string())),
                            )),
                            Box::new(Node::Block(vec![
                                Node::Assign(
                                    "found".to_string(),
                                    Box::new(Node::BoolLiteral(true)),
                                ),
                                // Force break condition
                                Node::Assign(
                                    "i".to_string(),
                                    Box::new(Node::Identifier("len".to_string())),
                                ),
                            ])),
                            None,
                        ),
                        // Increment
                        Node::Assign(
                            "i".to_string(),
                            Box::new(Node::Add(
                                Box::new(Node::Identifier("i".to_string())),
                                Box::new(Node::IntLiteral(1)),
                            )),
                        ),
                    ])),
                ),
                Node::Return(Box::new(Node::Identifier("found".to_string()))),
            ])),
        ),
        // Array.Max(arr)
        Node::FnDef(
            "Array.Max".to_string(),
            vec!["arr".to_string()],
            Box::new(Node::Block(vec![
                Node::Assign(
                    "len".to_string(),
                    Box::new(Node::ArrayLen("arr".to_string())),
                ),
                Node::If(
                    Box::new(Node::Eq(
                        Box::new(Node::Identifier("len".to_string())),
                        Box::new(Node::IntLiteral(0)),
                    )),
                    Box::new(Node::Return(Box::new(Node::IntLiteral(0)))),
                    None,
                ),
                Node::Assign(
                    "max_val".to_string(),
                    Box::new(Node::ArrayGet(
                        "arr".to_string(),
                        Box::new(Node::IntLiteral(0)),
                    )),
                ),
                Node::Assign("i".to_string(), Box::new(Node::IntLiteral(1))),
                Node::While(
                    Box::new(Node::Lt(
                        Box::new(Node::Identifier("i".to_string())),
                        Box::new(Node::Identifier("len".to_string())),
                    )),
                    Box::new(Node::Block(vec![
                        Node::Assign(
                            "curr".to_string(),
                            Box::new(Node::ArrayGet(
                                "arr".to_string(),
                                Box::new(Node::Identifier("i".to_string())),
                            )),
                        ),
                        Node::If(
                            Box::new(Node::Lt(
                                Box::new(Node::Identifier("max_val".to_string())),
                                Box::new(Node::Identifier("curr".to_string())),
                            )),
                            Box::new(Node::Assign(
                                "max_val".to_string(),
                                Box::new(Node::Identifier("curr".to_string())),
                            )),
                            None,
                        ),
                        Node::Assign(
                            "i".to_string(),
                            Box::new(Node::Add(
                                Box::new(Node::Identifier("i".to_string())),
                                Box::new(Node::IntLiteral(1)),
                            )),
                        ),
                    ])),
                ),
                Node::Return(Box::new(Node::Identifier("max_val".to_string()))),
            ])),
        ),
        // Array.Reverse(arr)
        Node::FnDef(
            "Array.Reverse".to_string(),
            vec!["arr".to_string()],
            Box::new(Node::Block(vec![
                Node::Assign(
                    "len".to_string(),
                    Box::new(Node::ArrayLen("arr".to_string())),
                ),
                Node::Assign("reversed".to_string(), Box::new(Node::ArrayLiteral(vec![]))),
                Node::Assign("i".to_string(), Box::new(Node::IntLiteral(0))),
                Node::While(
                    Box::new(Node::Lt(
                        Box::new(Node::Identifier("i".to_string())),
                        Box::new(Node::Identifier("len".to_string())),
                    )),
                    Box::new(Node::Block(vec![
                        // reversed.push( arr[len - 1 - i] )
                        Node::Assign(
                            "idx".to_string(),
                            Box::new(Node::Sub(
                                Box::new(Node::Sub(
                                    Box::new(Node::Identifier("len".to_string())),
                                    Box::new(Node::IntLiteral(1)),
                                )),
                                Box::new(Node::Identifier("i".to_string())),
                            )),
                        ),
                        Node::ArrayPush(
                            "reversed".to_string(),
                            Box::new(Node::ArrayGet(
                                "arr".to_string(),
                                Box::new(Node::Identifier("idx".to_string())),
                            )),
                        ),
                        Node::Assign(
                            "i".to_string(),
                            Box::new(Node::Add(
                                Box::new(Node::Identifier("i".to_string())),
                                Box::new(Node::IntLiteral(1)),
                            )),
                        ),
                    ])),
                ),
                Node::Return(Box::new(Node::Identifier("reversed".to_string()))),
            ])),
        ),
    ]);

    // ---------------------------------------------------------
    // 2. math_ext.aec
    // ---------------------------------------------------------
    // Provide: Math.Clamp(val, min, max), Math.Lerp(a, b, t), Math.DegToRad(deg)

    let math_ext_ast = Node::Block(vec![
        // Math.Clamp(val, min, max)
        Node::FnDef(
            "Math.Clamp".to_string(),
            vec!["val".to_string(), "min".to_string(), "max".to_string()],
            Box::new(Node::Block(vec![
                Node::If(
                    Box::new(Node::Lt(
                        Box::new(Node::Identifier("val".to_string())),
                        Box::new(Node::Identifier("min".to_string())),
                    )),
                    Box::new(Node::Return(Box::new(Node::Identifier("min".to_string())))),
                    None,
                ),
                Node::If(
                    Box::new(Node::Lt(
                        Box::new(Node::Identifier("max".to_string())),
                        Box::new(Node::Identifier("val".to_string())),
                    )),
                    Box::new(Node::Return(Box::new(Node::Identifier("max".to_string())))),
                    None,
                ),
                Node::Return(Box::new(Node::Identifier("val".to_string()))),
            ])),
        ),
        // Math.Lerp(a, b, t)  ->  a + (b - a) * t
        Node::FnDef(
            "Math.Lerp".to_string(),
            vec!["a".to_string(), "b".to_string(), "t".to_string()],
            Box::new(Node::Return(Box::new(Node::Add(
                Box::new(Node::Identifier("a".to_string())),
                Box::new(Node::Mul(
                    Box::new(Node::Sub(
                        Box::new(Node::Identifier("b".to_string())),
                        Box::new(Node::Identifier("a".to_string())),
                    )),
                    Box::new(Node::Identifier("t".to_string())),
                )),
            )))),
        ),
        // Math.DegToRad(deg) -> deg * (PI / 180.0) -> deg * 0.01745329251
        Node::FnDef(
            "Math.DegToRad".to_string(),
            vec!["deg".to_string()],
            Box::new(Node::Return(Box::new(Node::Mul(
                Box::new(Node::Identifier("deg".to_string())),
                Box::new(Node::FloatLiteral(0.01745329251)),
            )))),
        ),
    ]);

    // ---------------------------------------------------------
    // 3. string_utils.aec
    // ---------------------------------------------------------
    // Provide: String.IsNotEmpty(str), String.FormatLog(msg)

    let string_utils_ast = Node::Block(vec![
        // String.IsNotEmpty(str) -> len > 0
        Node::FnDef(
            "String.IsNotEmpty".to_string(),
            vec!["str".to_string()],
            Box::new(Node::Block(vec![
                Node::Assign(
                    "len".to_string(),
                    Box::new(Node::ArrayLen("str".to_string())),
                ),
                Node::If(
                    Box::new(Node::Lt(
                        Box::new(Node::IntLiteral(0)),
                        Box::new(Node::Identifier("len".to_string())),
                    )),
                    Box::new(Node::Return(Box::new(Node::BoolLiteral(true)))),
                    Some(Box::new(Node::Return(Box::new(Node::BoolLiteral(false))))),
                ),
            ])),
        ),
        // String.FormatLog(msg) -> concat("[KnotenCore] ", msg)
        Node::FnDef(
            "String.FormatLog".to_string(),
            vec!["msg".to_string()],
            Box::new(Node::Return(Box::new(Node::Concat(
                Box::new(Node::StringLiteral("[KnotenCore] ".to_string())),
                Box::new(Node::Identifier("msg".to_string())),
            )))),
        ),
    ]);

    // ---------------------------------------------------------
    // 4. stdlib_demo.aec
    // ---------------------------------------------------------
    let stdlib_demo_ast = Node::Block(vec![
        Node::Import("stdlib/array_utils.nod".to_string()),
        Node::Import("stdlib/math_ext.nod".to_string()),
        Node::Import("stdlib/string_utils.nod".to_string()),
        // Print FormatLog
        Node::Print(Box::new(Node::Call(
            "String.FormatLog".to_string(),
            vec![Node::StringLiteral(
                "Testing Standard Library Modules...".to_string(),
            )],
        ))),
        // Math.Clamp test (val=10, min=0, max=5) -> 5
        Node::Assign(
            "clamp_res".to_string(),
            Box::new(Node::Call(
                "Math.Clamp".to_string(),
                vec![
                    Node::IntLiteral(10),
                    Node::IntLiteral(0),
                    Node::IntLiteral(5),
                ],
            )),
        ),
        Node::Print(Box::new(Node::Identifier("clamp_res".to_string()))),
        // Array.Max test
        Node::Assign(
            "arr".to_string(),
            Box::new(Node::ArrayLiteral(vec![
                Node::IntLiteral(10),
                Node::IntLiteral(42),
                Node::IntLiteral(5),
            ])),
        ),
        Node::Assign(
            "arr_max".to_string(),
            Box::new(Node::Call(
                "Array.Max".to_string(),
                vec![Node::Identifier("arr".to_string())],
            )),
        ),
        Node::Print(Box::new(Node::Identifier("arr_max".to_string()))),
    ]);

    // ---------------------------------------------------------
    // Save to Disk
    // ---------------------------------------------------------
    let save_file = |dir: &PathBuf, name: &str, ast: &Node| {
        let text_data = serde_json::to_string_pretty(ast).unwrap();
        let mut path = dir.clone();
        path.push(name);
        fs::write(&path, &text_data).unwrap();
        println!("Saved {:?}", path);
    };

    save_file(&stdlib_dir, "array_utils.nod", &array_utils_ast);
    save_file(&stdlib_dir, "math_ext.nod", &math_ext_ast);
    save_file(&stdlib_dir, "string_utils.nod", &string_utils_ast);

    let mut examples_dir = std::env::current_dir().unwrap();
    examples_dir.push("examples");
    examples_dir.push("core");
    save_file(&examples_dir, "stdlib_demo.nod", &stdlib_demo_ast);

    println!("ASL Generation Complete!");
}
