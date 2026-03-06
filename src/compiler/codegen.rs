use crate::ast::Node;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
pub enum VarKind {
    Normal,
    Handle,
    HandleArray,
    HandleMap,
}

pub struct Codegen {
    pub scopes: Vec<HashMap<String, VarKind>>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn is_handle_expr(&self, n: &Node) -> bool {
        match n {
            Node::NativeCall(fn_name, _) => {
                matches!(
                    fn_name.as_str(),
                    "registry_create_counter"
                        | "registry_create_window"
                        | "registry_file_create"
                        | "registry_now"
                        | "registry_gpu_init"
                        | "registry_voxel_world_create"
                        | "registry_texture_load"
                )
            }
            Node::Identifier(name) => {
                for scope in self.scopes.iter().rev() {
                    if let Some(kind) = scope.get(name) {
                        return *kind != VarKind::Normal;
                    }
                }
                false
            }
            Node::ArrayCreate(nodes) => nodes.iter().any(|node| self.is_handle_expr(node)),
            _ => false,
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
                for (var_name, kind) in current_scope {
                    if *kind == VarKind::Handle {
                        out.push_str(&format!("    registry::registry_release({});\n", var_name));
                    } else if *kind == VarKind::HandleArray {
                        out.push_str(&format!("    for item in {} {{\n        registry::registry_release(item);\n    }}\n", var_name));
                    } else if *kind == VarKind::HandleMap {
                        out.push_str(&format!("    for item in {}.into_values() {{\n        registry::registry_release(item);\n    }}\n", var_name));
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

                let mut kind = VarKind::Normal;
                if self.is_handle_expr(&**expr) {
                    if let Node::ArrayCreate(_) = &**expr {
                        kind = VarKind::HandleArray;
                    } else if let Node::Identifier(name) = &**expr {
                        for scope in self.scopes.iter().rev() {
                            if let Some(k) = scope.get(name) {
                                kind = *k;
                                break;
                            }
                        }
                    } else {
                        kind = VarKind::Handle;
                    }
                }

                if already_exists {
                    let mut previously_was = VarKind::Normal;
                    for scope in self.scopes.iter_mut().rev() {
                        if let Some(was_kind) = scope.get(name) {
                            previously_was = *was_kind;
                            scope.insert(name.clone(), kind);
                            break;
                        }
                    }

                    if previously_was == VarKind::Handle {
                        // Drop former handle before reassignment
                        format!(
                            "registry::registry_release({});\n    {} = {}",
                            name, name, inner
                        )
                    } else if previously_was == VarKind::HandleArray {
                        format!(
                            "for item in {} {{\n        registry::registry_release(item);\n    }}\n    {} = {}",
                            name, name, inner
                        )
                    } else if previously_was == VarKind::HandleMap {
                        format!(
                            "for item in {}.into_values() {{\n        registry::registry_release(item);\n    }}\n    {} = {}",
                            name, name, inner
                        )
                    } else {
                        format!("{} = {}", name, inner)
                    }
                } else {
                    if let Some(current_scope) = self.scopes.last_mut() {
                        current_scope.insert(name.clone(), kind);
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
            Node::ArrayCreate(nodes) => {
                let mut elem_strs = Vec::new();
                for n in nodes {
                    elem_strs.push(self.generate(n, false));
                }
                format!("vec![{}]", elem_strs.join(", "))
            }
            Node::ArrayGet(arr, index) => {
                format!(
                    "{}[{} as usize]",
                    self.generate(arr, false),
                    self.generate(index, false)
                )
            }
            Node::ArraySet(arr, index, val) => {
                let is_handle = self.is_handle_expr(&**val);
                let arr_code = self.generate(arr, false);
                let idx_code = self.generate(index, false);
                let val_code = self.generate(val, false);

                if is_handle {
                    format!(
                        "{{ registry::registry_release({}[{} as usize]); {}[{} as usize] = {}; }}",
                        arr_code, idx_code, arr_code, idx_code, val_code
                    )
                } else {
                    format!("{}[{} as usize] = {}", arr_code, idx_code, val_code)
                }
            }
            Node::ArrayPush(arr, val) => {
                if self.is_handle_expr(&**val) {
                    if let Node::Identifier(name) = &**arr {
                        for scope in self.scopes.iter_mut().rev() {
                            if scope.contains_key(name) {
                                scope.insert(name.clone(), VarKind::HandleArray);
                                break;
                            }
                        }
                    }
                }
                format!(
                    "{}.push({})",
                    self.generate(arr, false),
                    self.generate(val, false)
                )
            }
            Node::ArrayLen(arr) => {
                format!("{}.len() as i64", self.generate(arr, false))
            }
            Node::MapCreate => "std::collections::HashMap::new()".to_string(),
            Node::MapGet(map, key) => {
                format!(
                    "{}.get(&{}).cloned().unwrap()",
                    self.generate(map, false),
                    self.generate(key, false)
                )
            }
            Node::MapSet(map, key, val) => {
                if self.is_handle_expr(&**val) {
                    if let Node::Identifier(name) = &**map {
                        for scope in self.scopes.iter_mut().rev() {
                            if scope.contains_key(name) {
                                scope.insert(name.clone(), VarKind::HandleMap);
                                break;
                            }
                        }
                    }
                }
                format!(
                    "{}.insert({}, {})",
                    self.generate(map, false),
                    self.generate(key, false),
                    self.generate(val, false)
                )
            }
            Node::MapHasKey(map, key) => {
                format!(
                    "{}.contains_key(&{})",
                    self.generate(map, false),
                    self.generate(key, false)
                )
            }
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
            Node::Store { key, value } => {
                let inner = self.generate(value, false);
                format!("knoten_core::vm::storage::store_value(\"{}\", &serde_json::json!({})).unwrap()", key, inner)
            }
            Node::Load { key } => {
                if key == "chess_turn" {
                    format!("knoten_core::vm::storage::load_value(\"{}\").ok().and_then(|v| v.as_i64()).unwrap_or(-1)", key)
                } else if key == "chess_board" {
                    format!("knoten_core::vm::storage::load_value(\"{}\").ok().and_then(|v| v.as_array().map(|arr| arr.iter().map(|s| s.as_str().unwrap_or(\" \").to_string()).collect::<Vec<String>>())).unwrap_or_else(|| vec![])", key)
                } else {
                    format!("knoten_core::vm::storage::load_value(\"{}\").unwrap()", key)
                }
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
