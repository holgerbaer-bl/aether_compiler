use crate::ast::Node;
use std::collections::HashSet;

pub struct Codegen {
    pub declared_vars: HashSet<String>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            declared_vars: HashSet::new(),
        }
    }

    pub fn generate(&mut self, node: &Node, is_root: bool) -> String {
        match node {
            Node::Block(nodes) => {
                let mut out = String::new();
                if is_root {
                    out.push_str("fn main() {\n");
                } else {
                    out.push_str("{\n");
                }

                for n in nodes {
                    let line = self.generate(n, false);
                    out.push_str(&format!("    {};\n", line));
                }

                if is_root {
                    out.push_str("}\n");
                } else {
                    out.push_str("}");
                }
                out
            }
            Node::Print(expr) => {
                let inner = self.generate(expr, false);
                format!("println!(\"{{}}\", {})", inner)
            }
            Node::Assign(name, expr) => {
                let inner = self.generate(expr, false);
                if self.declared_vars.insert(name.clone()) {
                    // Variable was not in HashSet, this is its first declaration in this scope tracking
                    format!("let mut {} = {}", name, inner)
                } else {
                    // Variable already exists
                    format!("{} = {}", name, inner)
                }
            }
            Node::IntLiteral(v) => format!("{}", v),
            Node::FloatLiteral(v) => format!("{}_f64", v),
            Node::BoolLiteral(v) => format!("{}", v),
            Node::StringLiteral(v) => format!("String::from(\"{}\")", v),
            Node::Identifier(name) => name.clone(),
            Node::Add(l, r) => format!(
                "({} + {})",
                self.generate(l, false),
                self.generate(r, false)
            ),
            Node::Sub(l, r) => format!(
                "({} - {})",
                self.generate(l, false),
                self.generate(r, false)
            ),
            Node::Mul(l, r) => format!(
                "({} * {})",
                self.generate(l, false),
                self.generate(r, false)
            ),
            Node::Div(l, r) => format!(
                "({} / {})",
                self.generate(l, false),
                self.generate(r, false)
            ),
            // Sprint 38 MVP support boundary
            _ => format!("/* Unsupported node in Sprint 38 codegen: {:?} */", node),
        }
    }
}

pub fn generate_rust_code(ast: &Node) -> String {
    let mut cg = Codegen::new();
    cg.generate(ast, true)
}
