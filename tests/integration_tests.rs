use knoten_core::ast::Node;
use knoten_core::executor::ExecutionEngine;
use std::fs;
use std::path::PathBuf;

// Helper to determine where to output files
fn get_out_dir() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("target");
    path.push("tests_nod");
    fs::create_dir_all(&path).unwrap();
    path
}

/// The knoten_test! macro generates a standard #[test] function.
/// It serializes the provided AST node into target/tests_nod/<name>.nod,
/// records the expected return value, and then intentionally panics,
/// signaling that the compiler needs to execute it but does not exist yet.
macro_rules! knoten_test {
    ($name:ident, $node:expr, $expected_info:expr) => {
        #[test]
        fn $name() {
            let ast: Node = $node;

            // Serialize current test to disk for the Meta-Compiler to read
            let text_data = serde_json::to_string(&ast).expect("JSON Serialization failed");
            let mut path = get_out_dir();
            path.push("current_test.nod");
            fs::write(&path, &text_data).expect("Write failed");

            // Write expected output text file for test oracle validation
            let mut expected_path = get_out_dir();
            expected_path.push(format!("{}.expected", stringify!($name)));
            fs::write(&expected_path, $expected_info).unwrap();

            // SPRINT 9: Execute using direct AST evaluation instead of Meta-Compiler
            let mut engine = ExecutionEngine::new();
            let result = engine.execute(&ast);

            // Verify exactly matching the expected output string
            assert_eq!(
                result,
                $expected_info,
                "Mismatched Execution Output for '{}'",
                stringify!($name)
            );
        }
    };
}

// ------------------------------------------------------------------
// Tests 1-10: Literals and Basic Types
// ------------------------------------------------------------------
knoten_test!(
    test_01_int_literal,
    Node::IntLiteral(42),
    "Return: 42 (i64)"
);
knoten_test!(
    test_02_float_literal,
    Node::FloatLiteral(3.14),
    "Return: 3.14 (f64)"
);
knoten_test!(
    test_03_bool_literal_true,
    Node::BoolLiteral(true),
    "Return: true (bool)"
);
knoten_test!(
    test_04_bool_literal_false,
    Node::BoolLiteral(false),
    "Return: false (bool)"
);
knoten_test!(
    test_05_string_literal,
    Node::StringLiteral("Hello".to_string()),
    "Return: \"Hello\" (String)"
);
knoten_test!(test_06_int_zero, Node::IntLiteral(0), "Return: 0 (i64)");
knoten_test!(
    test_07_int_negative,
    Node::IntLiteral(-999),
    "Return: -999 (i64)"
);
knoten_test!(
    test_08_float_negative,
    Node::FloatLiteral(-1.5),
    "Return: -1.5 (f64)"
);
knoten_test!(
    test_09_float_zero,
    Node::FloatLiteral(0.0),
    "Return: 0.0 (f64)"
);
knoten_test!(
    test_10_string_empty,
    Node::StringLiteral("".to_string()),
    "Return: \"\" (String)"
);

// ------------------------------------------------------------------
// Tests 11-20: Memory Operations (Assignments)
// ------------------------------------------------------------------
knoten_test!(
    test_11_assign_int,
    Node::Assign("x".to_string(), Box::new(Node::IntLiteral(10))),
    "Return: 10 (i64), Memory: x = 10"
);
knoten_test!(
    test_12_assign_float,
    Node::Assign("pi".to_string(), Box::new(Node::FloatLiteral(3.1415))),
    "Return: 3.1415 (f64), Memory: pi = 3.1415"
);
knoten_test!(
    test_13_assign_bool,
    Node::Assign("flag".to_string(), Box::new(Node::BoolLiteral(true))),
    "Return: true (bool), Memory: flag = true"
);
knoten_test!(
    test_14_assign_string,
    Node::Assign(
        "msg".to_string(),
        Box::new(Node::StringLiteral("ok".to_string()))
    ),
    "Return: \"ok\" (String), Memory: msg = \"ok\""
);
knoten_test!(
    test_15_assign_expr,
    Node::Assign(
        "y".to_string(),
        Box::new(Node::Add(
            Box::new(Node::IntLiteral(5)),
            Box::new(Node::IntLiteral(5))
        ))
    ),
    "Return: 10 (i64), Memory: y = 10"
);
knoten_test!(
    test_16_assign_reassign,
    Node::Block(vec![
        Node::Assign("x".to_string(), Box::new(Node::IntLiteral(1))),
        Node::Assign("x".to_string(), Box::new(Node::IntLiteral(2))),
    ]),
    "Return: 2 (i64)"
);
knoten_test!(
    test_17_read_identifier,
    Node::Block(vec![
        Node::Assign("x".to_string(), Box::new(Node::IntLiteral(42))),
        Node::Identifier("x".to_string()),
    ]),
    "Return: 42 (i64)"
);
knoten_test!(
    test_18_assign_from_identifier,
    Node::Block(vec![
        Node::Assign("x".to_string(), Box::new(Node::IntLiteral(42))),
        Node::Assign("y".to_string(), Box::new(Node::Identifier("x".to_string()))),
    ]),
    "Return: 42 (i64)"
);
knoten_test!(
    test_19_read_undefined,
    Node::Identifier("undeclared".to_string()),
    "Fault: Undefined identifier: undeclared"
);
knoten_test!(
    test_20_assign_undefined,
    Node::Assign(
        "y".to_string(),
        Box::new(Node::Identifier("undeclared".to_string()))
    ),
    "Fault: Undefined identifier: undeclared"
);

// ------------------------------------------------------------------
// Tests 21-30: Mathematical Operations
// ------------------------------------------------------------------
knoten_test!(
    test_21_add_int,
    Node::Add(
        Box::new(Node::IntLiteral(10)),
        Box::new(Node::IntLiteral(20))
    ),
    "Return: 30 (i64)"
);
knoten_test!(
    test_22_sub_int,
    Node::Sub(
        Box::new(Node::IntLiteral(50)),
        Box::new(Node::IntLiteral(20))
    ),
    "Return: 30 (i64)"
);
knoten_test!(
    test_23_mul_int,
    Node::Mul(Box::new(Node::IntLiteral(6)), Box::new(Node::IntLiteral(7))),
    "Return: 42 (i64)"
);
knoten_test!(
    test_24_div_int,
    Node::Div(
        Box::new(Node::IntLiteral(100)),
        Box::new(Node::IntLiteral(10))
    ),
    "Return: 10 (i64)"
);
knoten_test!(
    test_25_add_float,
    Node::Add(
        Box::new(Node::FloatLiteral(1.5)),
        Box::new(Node::FloatLiteral(2.5))
    ),
    "Return: 4.0 (f64)"
);
knoten_test!(
    test_26_sub_float,
    Node::Sub(
        Box::new(Node::FloatLiteral(5.0)),
        Box::new(Node::FloatLiteral(1.1))
    ),
    "Return: 3.9 (f64)"
);
knoten_test!(
    test_27_mul_float,
    Node::Mul(
        Box::new(Node::FloatLiteral(2.0)),
        Box::new(Node::FloatLiteral(3.1))
    ),
    "Return: 6.2 (f64)"
);
knoten_test!(
    test_28_div_float,
    Node::Div(
        Box::new(Node::FloatLiteral(10.0)),
        Box::new(Node::FloatLiteral(2.5))
    ),
    "Return: 4.0 (f64)"
);
knoten_test!(
    test_29_complex_math,
    Node::Add(
        Box::new(Node::Mul(
            Box::new(Node::IntLiteral(2)),
            Box::new(Node::IntLiteral(3))
        )),
        Box::new(Node::IntLiteral(4))
    ),
    "Return: 10 (i64)"
);
knoten_test!(
    test_30_div_by_zero,
    Node::Div(
        Box::new(Node::IntLiteral(10)),
        Box::new(Node::IntLiteral(0))
    ),
    "Fault: Division by zero"
);

// ------------------------------------------------------------------
// Tests 31-40: Logical Operations and Control Flow (If)
// ------------------------------------------------------------------
knoten_test!(
    test_31_eq_true,
    Node::Eq(Box::new(Node::IntLiteral(5)), Box::new(Node::IntLiteral(5))),
    "Return: true (bool)"
);
knoten_test!(
    test_32_eq_false,
    Node::Eq(Box::new(Node::IntLiteral(5)), Box::new(Node::IntLiteral(6))),
    "Return: false (bool)"
);
knoten_test!(
    test_33_lt_true,
    Node::Lt(
        Box::new(Node::IntLiteral(5)),
        Box::new(Node::IntLiteral(10))
    ),
    "Return: true (bool)"
);
knoten_test!(
    test_34_lt_false,
    Node::Lt(
        Box::new(Node::IntLiteral(10)),
        Box::new(Node::IntLiteral(5))
    ),
    "Return: false (bool)"
);
knoten_test!(
    test_35_if_true,
    Node::If(
        Box::new(Node::BoolLiteral(true)),
        Box::new(Node::IntLiteral(1)),
        None
    ),
    "Return: 1 (i64)"
);
knoten_test!(
    test_36_if_false,
    Node::If(
        Box::new(Node::BoolLiteral(false)),
        Box::new(Node::IntLiteral(1)),
        None
    ),
    "Return: void"
);
knoten_test!(
    test_37_if_else_true,
    Node::If(
        Box::new(Node::BoolLiteral(true)),
        Box::new(Node::IntLiteral(1)),
        Some(Box::new(Node::IntLiteral(2)))
    ),
    "Return: 1 (i64)"
);
knoten_test!(
    test_38_if_else_false,
    Node::If(
        Box::new(Node::BoolLiteral(false)),
        Box::new(Node::IntLiteral(1)),
        Some(Box::new(Node::IntLiteral(2)))
    ),
    "Return: 2 (i64)"
);
knoten_test!(
    test_39_if_lt,
    Node::If(
        Box::new(Node::Lt(
            Box::new(Node::IntLiteral(5)),
            Box::new(Node::IntLiteral(10))
        )),
        Box::new(Node::StringLiteral("Less".to_string())),
        Some(Box::new(Node::StringLiteral("Greater".to_string())))
    ),
    "Return: \"Less\" (String)"
);
knoten_test!(
    test_40_if_eq_assign,
    Node::Block(vec![
        Node::Assign("x".to_string(), Box::new(Node::IntLiteral(42))),
        Node::If(
            Box::new(Node::Eq(
                Box::new(Node::Identifier("x".to_string())),
                Box::new(Node::IntLiteral(42))
            )),
            Box::new(Node::Assign("y".to_string(), Box::new(Node::IntLiteral(1)))),
            Some(Box::new(Node::Assign(
                "y".to_string(),
                Box::new(Node::IntLiteral(0))
            )))
        )
    ]),
    "Return: 1 (i64)"
);

// ------------------------------------------------------------------
// Tests 41-50: Control Flow (While, Block, Return)
// ------------------------------------------------------------------
knoten_test!(test_41_empty_block, Node::Block(vec![]), "Return: void");
knoten_test!(
    test_42_single_expr_block,
    Node::Block(vec![Node::IntLiteral(99)]),
    "Return: 99 (i64)"
);
knoten_test!(
    test_43_multi_expr_block,
    Node::Block(vec![
        Node::IntLiteral(1),
        Node::IntLiteral(2),
        Node::IntLiteral(3),
    ]),
    "Return: 3 (i64)"
);
knoten_test!(
    test_44_return_early,
    Node::Block(vec![
        Node::Return(Box::new(Node::IntLiteral(42))),
        Node::IntLiteral(99), // Should not be reached
    ]),
    "Return: 42 (i64)"
);
knoten_test!(
    test_45_while_loop,
    Node::Block(vec![
        Node::Assign("i".to_string(), Box::new(Node::IntLiteral(0))),
        Node::While(
            Box::new(Node::Lt(
                Box::new(Node::Identifier("i".to_string())),
                Box::new(Node::IntLiteral(3))
            )),
            Box::new(Node::Assign(
                "i".to_string(),
                Box::new(Node::Add(
                    Box::new(Node::Identifier("i".to_string())),
                    Box::new(Node::IntLiteral(1))
                ))
            ))
        ),
        Node::Identifier("i".to_string())
    ]),
    "Return: 3 (i64)"
);
knoten_test!(
    test_46_while_never_executes,
    Node::Block(vec![
        Node::Assign("i".to_string(), Box::new(Node::IntLiteral(10))),
        Node::While(
            Box::new(Node::Lt(
                Box::new(Node::Identifier("i".to_string())),
                Box::new(Node::IntLiteral(5))
            )),
            Box::new(Node::Assign("i".to_string(), Box::new(Node::IntLiteral(0))))
        ),
        Node::Identifier("i".to_string())
    ]),
    "Return: 10 (i64)"
);
knoten_test!(
    test_47_nested_blocks,
    Node::Block(vec![
        Node::Assign("outer".to_string(), Box::new(Node::IntLiteral(1))),
        Node::Block(vec![Node::Assign(
            "inner".to_string(),
            Box::new(Node::IntLiteral(2))
        ),]),
        Node::Add(
            Box::new(Node::Identifier("outer".to_string())),
            Box::new(Node::Identifier("inner".to_string()))
        )
    ]),
    "Fault: Undefined identifier: inner"
);
knoten_test!(
    test_48_nested_if,
    Node::If(
        Box::new(Node::BoolLiteral(true)),
        Box::new(Node::If(
            Box::new(Node::BoolLiteral(false)),
            Box::new(Node::IntLiteral(1)),
            Some(Box::new(Node::IntLiteral(2)))
        )),
        None
    ),
    "Return: 2 (i64)"
);
knoten_test!(
    test_49_nested_while,
    Node::Block(vec![
        Node::Assign("i".to_string(), Box::new(Node::IntLiteral(0))),
        Node::Assign("sum".to_string(), Box::new(Node::IntLiteral(0))),
        Node::While(
            Box::new(Node::Lt(
                Box::new(Node::Identifier("i".to_string())),
                Box::new(Node::IntLiteral(2))
            )),
            Box::new(Node::Block(vec![
                Node::Assign("j".to_string(), Box::new(Node::IntLiteral(0))),
                Node::While(
                    Box::new(Node::Lt(
                        Box::new(Node::Identifier("j".to_string())),
                        Box::new(Node::IntLiteral(2))
                    )),
                    Box::new(Node::Block(vec![
                        Node::Assign(
                            "sum".to_string(),
                            Box::new(Node::Add(
                                Box::new(Node::Identifier("sum".to_string())),
                                Box::new(Node::IntLiteral(1))
                            ))
                        ),
                        Node::Assign(
                            "j".to_string(),
                            Box::new(Node::Add(
                                Box::new(Node::Identifier("j".to_string())),
                                Box::new(Node::IntLiteral(1))
                            ))
                        ),
                    ]))
                ),
                Node::Assign(
                    "i".to_string(),
                    Box::new(Node::Add(
                        Box::new(Node::Identifier("i".to_string())),
                        Box::new(Node::IntLiteral(1))
                    ))
                ),
            ]))
        ),
        Node::Identifier("sum".to_string())
    ]),
    "Return: 4 (i64)"
);
knoten_test!(
    test_50_complex_combination,
    Node::Block(vec![
        Node::Assign("n".to_string(), Box::new(Node::IntLiteral(5))),
        Node::Assign("fact".to_string(), Box::new(Node::IntLiteral(1))),
        Node::While(
            Box::new(Node::Lt(
                Box::new(Node::IntLiteral(0)),
                Box::new(Node::Identifier("n".to_string()))
            )),
            Box::new(Node::Block(vec![
                Node::Assign(
                    "fact".to_string(),
                    Box::new(Node::Mul(
                        Box::new(Node::Identifier("fact".to_string())),
                        Box::new(Node::Identifier("n".to_string()))
                    ))
                ),
                Node::Assign(
                    "n".to_string(),
                    Box::new(Node::Sub(
                        Box::new(Node::Identifier("n".to_string())),
                        Box::new(Node::IntLiteral(1))
                    ))
                ),
            ]))
        ),
        Node::Identifier("fact".to_string())
    ]),
    "Return: 120 (i64)"
);

// ------------------------------------------------------------------
// Tests 51-54: V2 Sprint Bootstrapping Features (Arrays, Bitwise, Funcs)
// ------------------------------------------------------------------
knoten_test!(
    test_51_array_literal,
    Node::ArrayCreate(vec![Node::IntLiteral(1), Node::IntLiteral(2)]),
    "Return: [1 (i64), 2 (i64)] (Array)"
);

knoten_test!(
    test_52_bitwise_ops,
    Node::BitShiftLeft(Box::new(Node::IntLiteral(2)), Box::new(Node::IntLiteral(3))),
    "Return: 16 (i64)"
);

knoten_test!(
    test_53_function_call,
    Node::Block(vec![
        Node::FnDef(
            "double".to_string(),
            vec!["x".to_string()],
            Box::new(Node::Return(Box::new(Node::Add(
                Box::new(Node::Identifier("x".to_string())),
                Box::new(Node::Identifier("x".to_string()))
            ))))
        ),
        Node::Call("double".to_string(), vec![Node::IntLiteral(21)])
    ]),
    "Return: 42 (i64), Memory: double = <fn>"
);

knoten_test!(
    test_54_string_concat,
    Node::Concat(
        Box::new(Node::StringLiteral("hello ".to_string())),
        Box::new(Node::StringLiteral("world".to_string()))
    ),
    "Return: \"hello world\" (String)"
);
