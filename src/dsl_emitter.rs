use crate::ast::Node;

pub fn emit_dsl(node: &Node, indent: usize) -> String {
    let pad = " ".repeat(indent * 4);
    match node {
        // Literals
        Node::IntLiteral(v) => v.to_string(),
        Node::FloatLiteral(v) => {
            let s = v.to_string();
            if !s.contains('.') {
                format!("{}.0", s)
            } else {
                s
            }
        }
        Node::BoolLiteral(v) => v.to_string(),
        Node::StringLiteral(v) => format!("\"{}\"", v),

        // Memory
        Node::Identifier(name) => name.clone(),
        Node::Assign(name, val) => format!("{} = {}", name, emit_dsl(val, indent)),

        // Math & Logic
        Node::Add(l, r) => format!("({} + {})", emit_dsl(l, indent), emit_dsl(r, indent)),
        Node::Sub(l, r) => format!("({} - {})", emit_dsl(l, indent), emit_dsl(r, indent)),
        Node::Mul(l, r) => format!("({} * {})", emit_dsl(l, indent), emit_dsl(r, indent)),
        Node::Div(l, r) => format!("({} / {})", emit_dsl(l, indent), emit_dsl(r, indent)),
        Node::Eq(l, r) => format!("({} == {})", emit_dsl(l, indent), emit_dsl(r, indent)),
        Node::Lt(l, r) => format!("({} < {})", emit_dsl(l, indent), emit_dsl(r, indent)),
        Node::Gt(l, r) => format!("({} > {})", emit_dsl(l, indent), emit_dsl(r, indent)),
        Node::BitAnd(l, r) => format!("({} & {})", emit_dsl(l, indent), emit_dsl(r, indent)),
        Node::BitShiftLeft(l, r) => format!("({} << {})", emit_dsl(l, indent), emit_dsl(r, indent)),
        Node::BitShiftRight(l, r) => {
            format!("({} >> {})", emit_dsl(l, indent), emit_dsl(r, indent))
        }

        // Structural
        Node::Block(stmts) => {
            let mut s = String::new();
            s.push_str("{\n");
            for stmt in stmts {
                let stmt_str = emit_dsl(stmt, indent + 1);
                s.push_str(&format!("    {}{};\n", pad, stmt_str));
            }
            s.push_str(&format!("{}}}", pad));
            s
        }
        Node::If(cond, then_b, else_b) => {
            let mut is_event = false;
            if let Node::UIButton(_) = **cond {
                is_event = true;
            }

            if is_event && else_b.is_none() {
                format!("{} -> {}", emit_dsl(cond, indent), emit_dsl(then_b, indent))
            } else {
                let mut s = format!(
                    "if ({}) {}",
                    emit_dsl(cond, indent),
                    emit_dsl(then_b, indent)
                );
                if let Some(eb) = else_b {
                    s.push_str(&format!(" else {}", emit_dsl(eb, indent)));
                }
                s
            }
        }
        Node::While(cond, body) => format!(
            "while ({}) {}",
            emit_dsl(cond, indent),
            emit_dsl(body, indent)
        ),

        Node::FnDef(name, args, body) => {
            format!(
                "fn {}({}) {}",
                name,
                args.join(", "),
                emit_dsl(body, indent)
            )
        }
        Node::Return(val) => format!("return {}", emit_dsl(val, indent)),

        // Arrays & Objects
        Node::ArrayCreate(args) => {
            let a: Vec<String> = args.iter().map(|n| emit_dsl(n, indent)).collect();
            format!("[{}]", a.join(", "))
        }
        Node::Index(container, idx) => {
            format!("{}[{}]", emit_dsl(container, indent), emit_dsl(idx, indent))
        }
        Node::ArrayGet(container, idx) => {
            format!("{}[{}]", emit_dsl(container, indent), emit_dsl(idx, indent))
        }
        Node::MapGet(container, idx) => {
            format!("{}[{}]", emit_dsl(container, indent), emit_dsl(idx, indent))
        }
        Node::PropertyGet(obj, prop) => format!("{}.{}", emit_dsl(obj, indent), prop),
        Node::ArraySet(container, idx, val) => format!(
            "{}[{}] = {}",
            emit_dsl(container, indent),
            emit_dsl(idx, indent),
            emit_dsl(val, indent)
        ),
        Node::MapSet(container, idx, val) => format!(
            "{}[{}] = {}",
            emit_dsl(container, indent),
            emit_dsl(idx, indent),
            emit_dsl(val, indent)
        ),
        Node::PropertySet(obj, prop, val) => format!(
            "{}.{} = {}",
            emit_dsl(obj, indent),
            prop,
            emit_dsl(val, indent)
        ),

        // GUI and Others -> Call syntax `Name(args) { block }`
        _ => {
            // Very generic fallback for all functions
            let variant_name = emit_node_name(node);
            let args = extract_args(node);

            // Check if the last arg is a block (for trailing closure)
            let mut arg_strs = Vec::new();
            let mut trailing_block = None;

            for (i, arg) in args.iter().enumerate() {
                if i == args.len() - 1 {
                    if let Node::Block(_) = arg {
                        trailing_block = Some(emit_dsl(arg, indent));
                        continue;
                    }
                }
                arg_strs.push(emit_dsl(arg, indent));
            }

            let mut s = format!("{}({})", variant_name, arg_strs.join(", "));
            if let Some(b) = trailing_block {
                s.push_str(&format!(" {}", b));
            }
            s
        }
    }
}

// Helper to extract args as refs from any Node enum variant dynamically via matching
fn extract_args(node: &Node) -> Vec<&Node> {
    match node {
        Node::Print(a) => vec![&**a],
        Node::Time => vec![],
        Node::InitGraphics => vec![],
        Node::InitAudio => vec![],
        Node::GetLastKeypress => vec![],
        Node::UIWindow(_, a, b) => vec![&**a, &**b], // Omitting ID from DSL for simplicity or pass it as string? DSL has UIWindow("id", "title") { }. So ID is arg 1.
        Node::UILabel(a) => vec![&**a],
        Node::UIButton(a) => vec![&**a],
        Node::UITextInput(a) => vec![&**a],
        Node::UIScrollArea(_, b) => vec![&**b], // Name is a string, not a Node, so skipping from function args for simpler DSL representation, wait!
        Node::UIHorizontal(a) => vec![&**a],
        Node::UIFullscreen(a) => vec![&**a],
        Node::UIGrid(_, _, c) => vec![&**c],
        Node::UISetStyle(a, b, c, d, e, f) => {
            let mut v = vec![&**a, &**b, &**c, &**d];
            if let Some(ex) = e {
                v.push(&**ex);
            }
            if let Some(fx) = f {
                v.push(&**fx);
            }
            v
        }
        Node::Concat(a, b) => vec![&**a, &**b],
        Node::FileRead(a) => vec![&**a],
        Node::FileWrite(a, b) => vec![&**a, &**b],
        Node::FSRead(a) => vec![&**a],
        Node::FSWrite(a, b) => vec![&**a, &**b],
        Node::Call(_, args) => args.iter().collect(),
        _ => vec![], // Fallback for complex nodes
    }
}

fn emit_node_name(node: &Node) -> String {
    let s = format!("{:?}", node);
    s.split('(').next().unwrap_or(&s).to_string()
}
