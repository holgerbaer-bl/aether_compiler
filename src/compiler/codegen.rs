use crate::ast::Node;
use std::collections::HashSet;

pub struct Codegen {
    pub scopes: Vec<HashSet<String>>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashSet::new()],
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

                // Push new scope
                self.scopes.push(HashSet::new());

                for n in nodes {
                    let line = self.generate(n, false);
                    out.push_str(&format!("    {};\n", line));
                }

                // Pop scope
                self.scopes.pop();

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

                let already_exists = self.scopes.iter().any(|s| s.contains(name));

                if already_exists {
                    // Variable already exists in an outer or current scope
                    format!("{} = {}", name, inner)
                } else {
                    // Variable was not in any HashSet, declare it in current scope
                    if let Some(current_scope) = self.scopes.last_mut() {
                        current_scope.insert(name.clone());
                    }
                    format!("let mut {} = {}", name, inner)
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
            Node::Eq(l, r) => format!(
                "({} == {})",
                self.generate(l, false),
                self.generate(r, false)
            ),
            Node::Lt(l, r) => format!(
                "({} < {})",
                self.generate(l, false),
                self.generate(r, false)
            ),
            Node::Gt(l, r) => format!(
                "({} > {})",
                self.generate(l, false),
                self.generate(r, false)
            ),
            Node::If(cond, then_b, else_b) => {
                let cond_str = self.generate(cond, false);
                let then_str = self.generate(then_b, false);
                if let Some(e) = else_b {
                    format!(
                        "if {} {} else {}",
                        cond_str,
                        then_str,
                        self.generate(e, false)
                    )
                } else {
                    format!("if {} {}", cond_str, then_str)
                }
            }
            Node::While(cond, body) => {
                format!(
                    "while {} {}",
                    self.generate(cond, false),
                    self.generate(body, false)
                )
            }
            // Sprint 38/39 MVP support boundary
            _ => format!("/* Unsupported node in Sprint 39 codegen: {:?} */", node),
        }
    }
}

pub fn generate_rust_code(ast: &Node) -> String {
    let mut cg = Codegen::new();
    cg.generate(ast, true)
}
