use crate::ast::Node;
use std::collections::HashMap;

pub struct Codegen {
    pub scopes: Vec<HashMap<String, bool>>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn generate(&mut self, node: &Node, is_root: bool) -> String {
        match node {
            Node::Block(nodes) => {
                let mut out = String::new();
                if is_root {
                    out.push_str("use knoten_core::natives::registry;\n\n");
                    out.push_str("fn main() {\n");
                } else {
                    out.push_str("{\n");
                }

                // Push new scope
                self.scopes.push(HashMap::new());

                for n in nodes {
                    let line = self.generate(n, false);
                    out.push_str(&format!("    {};\n", line));
                }

                // Identify handles to drop
                let current_scope = self.scopes.last().unwrap();
                for (var_name, is_handle) in current_scope {
                    if *is_handle {
                        out.push_str(&format!("    registry::registry_release({});\n", var_name));
                    }
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
                let already_exists = self.scopes.iter().any(|s| s.contains_key(name));

                let mut is_handle = false;
                if let Node::NativeCall(fn_name, _) = &**expr {
                    if fn_name == "registry_create_counter"
                        || fn_name == "registry_create_window"
                        || fn_name == "registry_file_create"
                        || fn_name == "registry_now"
                        || fn_name == "registry_gpu_init"
                    {
                        is_handle = true;
                    }
                }

                if already_exists {
                    let mut previously_was_handle = false;
                    for scope in self.scopes.iter_mut().rev() {
                        if let Some(was_handle) = scope.get(name) {
                            previously_was_handle = *was_handle;
                            scope.insert(name.clone(), is_handle);
                            break;
                        }
                    }

                    if previously_was_handle {
                        // Drop former handle before reassignment
                        format!(
                            "registry::registry_release({});\n    {} = {}",
                            name, name, inner
                        )
                    } else {
                        format!("{} = {}", name, inner)
                    }
                } else {
                    if let Some(current_scope) = self.scopes.last_mut() {
                        current_scope.insert(name.clone(), is_handle);
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
            Node::NativeCall(fn_name, args) => {
                let mut arg_strs = Vec::new();
                for a in args {
                    arg_strs.push(self.generate(a, false));
                }
                format!("registry::{}({})", fn_name, arg_strs.join(", "))
            }
            // Sprint 38/39/40 MVP support boundary
            _ => format!("/* Unsupported node in Sprint 40 codegen: {:?} */", node),
        }
    }
}

pub fn generate_rust_code(ast: &Node) -> String {
    let mut cg = Codegen::new();
    cg.generate(ast, true)
}
