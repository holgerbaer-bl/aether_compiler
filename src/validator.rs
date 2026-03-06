use crate::ast::Node;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct Validator {
    pub errors: Vec<String>,
    import_stack: HashSet<String>,
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            import_stack: HashSet::new(),
        }
    }

    pub fn validate(&mut self, node: &Node) -> Result<(), Vec<String>> {
        self.errors.clear();
        self.import_stack.clear();
        self.check_node(node);
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn check_node(&mut self, node: &Node) {
        match node {
            Node::Assign(name, val) => {
                if name.is_empty() {
                    self.errors
                        .push("Assign: Identifier name cannot be empty".to_string());
                }
                self.check_node(val);
            }
            Node::Store { key, value } => {
                if key.is_empty() {
                    self.errors.push("Store: Key cannot be empty".to_string());
                }
                self.check_node(value);
            }
            Node::Load { key } => {
                if key.is_empty() {
                    self.errors.push("Load: Key cannot be empty".to_string());
                }
            }
            Node::Add(l, r)
            | Node::Sub(l, r)
            | Node::Mul(l, r)
            | Node::Div(l, r)
            | Node::Mat4Mul(l, r)
            | Node::Eq(l, r)
            | Node::Lt(l, r)
            | Node::Gt(l, r)
            | Node::Concat(l, r)
            | Node::BitAnd(l, r)
            | Node::BitShiftLeft(l, r)
            | Node::BitShiftRight(l, r)
            | Node::LoadTextureAtlas(l, r)
            | Node::LoadSample(l, r) => {
                self.check_node(l);
                self.check_node(r);
            }
            Node::Fetch {
                method: _,
                url: _,
                callback,
            } => {
                self.check_node(callback);
            }
            Node::Extract { source, path } => {
                self.check_node(source);
                self.check_node(path);
            }
            Node::ObjectLiteral(map) => {
                for v in map.values() {
                    self.check_node(v);
                }
            }
            Node::PropertyGet(obj, _) => {
                self.check_node(obj);
            }
            Node::PropertySet(obj, _, val) => {
                self.check_node(obj);
                self.check_node(val);
            }
            Node::ArrayGet(arr, idx) => {
                self.check_node(arr);
                self.check_node(idx);
            }
            Node::ArraySet(arr, idx, val) => {
                self.check_node(arr);
                self.check_node(idx);
                self.check_node(val);
            }
            Node::ArrayPush(arr, val) => {
                self.check_node(arr);
                self.check_node(val);
            }
            Node::ArrayLen(arr) => {
                self.check_node(arr);
            }
            Node::MapGet(map, key) | Node::MapHasKey(map, key) => {
                self.check_node(map);
                self.check_node(key);
            }
            Node::MapSet(map, key, val) => {
                self.check_node(map);
                self.check_node(key);
                self.check_node(val);
            }
            Node::Sin(n)
            | Node::Cos(n)
            | Node::FileRead(n)
            | Node::FSRead(n)
            | Node::Print(n)
            | Node::EvalJSONNative(n)
            | Node::ToString(n)
            | Node::LoadShader(n)
            | Node::PollEvents(n)
            | Node::PlayAudioFile(n)
            | Node::LoadMesh(n)
            | Node::LoadTexture(n)
            | Node::LoadFont(n)
            | Node::UILabel(n)
            | Node::UIButton(n)
            | Node::UITextInput(n)
            | Node::InitCamera(n)
            | Node::DrawVoxelGrid(n)
            | Node::EnableInteraction(n)
            | Node::EnablePhysics(n)
            | Node::Return(n) => {
                self.check_node(n);
            }
            Node::FileWrite(f, d) | Node::FSWrite(f, d) => {
                self.check_node(f);
                self.check_node(d);
            }
            Node::FnDef(name, params, body) => {
                if name.is_empty() {
                    self.errors
                        .push("FnDef: Function name cannot be empty".to_string());
                }
                for param in params {
                    if param.is_empty() {
                        self.errors
                            .push(format!("FnDef ({}): Parameter name cannot be empty", name));
                    }
                }
                self.check_node(body);
            }
            Node::Call(name, args) | Node::NativeCall(name, args) => {
                if name.is_empty() {
                    self.errors
                        .push("Call/NativeCall: Function name cannot be empty".to_string());
                }
                for arg in args {
                    self.check_node(arg);
                }
            }
            Node::ExternCall {
                module,
                function,
                args,
            } => {
                if module.is_empty() || function.is_empty() {
                    self.errors
                        .push("ExternCall: Module and function cannot be empty".to_string());
                }
                for arg in args {
                    self.check_node(arg);
                }
            }
            Node::Block(nodes) | Node::ArrayCreate(nodes) => {
                for n in nodes {
                    self.check_node(n);
                }
            }
            Node::If(cond, then_b, else_b) => {
                self.check_node(cond);
                self.check_node(then_b);
                if let Some(eb) = else_b {
                    self.check_node(eb);
                }
            }
            Node::While(cond, body) => {
                self.check_node(cond);
                self.check_node(body);
            }
            Node::Import(path) => {
                if !Path::new(path).exists() {
                    self.errors
                        .push(format!("Import: File does not exist: {}", path));
                } else {
                    // Simple circular import check
                    if self.import_stack.contains(path) {
                        self.errors
                            .push(format!("Import: Circular dependency detected: {}", path));
                        return;
                    }

                    self.import_stack.insert(path.clone());
                    match fs::read_to_string(path) {
                        Ok(json) => match serde_json::from_str::<Node>(&json) {
                            Ok(parsed) => self.check_node(&parsed),
                            Err(e) => self
                                .errors
                                .push(format!("Import ({}): JSON Parse Error: {}", path, e)),
                        },
                        Err(e) => self
                            .errors
                            .push(format!("Import ({}): File Read Error: {}", path, e)),
                    }
                    self.import_stack.remove(path);
                }
            }
            Node::Index(target, idx) => {
                self.check_node(target);
                self.check_node(idx);
            }
            Node::RenderMesh(s, v, m)
            | Node::PlayNote(s, v, m)
            | Node::PlaySample(s, v, m)
            | Node::InitWindow(s, v, m) => {
                self.check_node(s);
                self.check_node(v);
                self.check_node(m);
            }
            Node::RenderAsset(s, m, t, u) | Node::SetVoxel(s, m, t, u) => {
                self.check_node(s);
                self.check_node(m);
                self.check_node(t);
                self.check_node(u);
            }
            Node::UIWindow(_, title, body) => {
                self.check_node(title);
                self.check_node(body);
            }
            Node::UISetStyle(r, s, a, f, bi, bh) => {
                self.check_node(r);
                self.check_node(s);
                self.check_node(a);
                self.check_node(f);
                if let Some(n) = bi {
                    self.check_node(n);
                }
                if let Some(n) = bh {
                    self.check_node(n);
                }
            }
            Node::UIHorizontal(b)
            | Node::UIFullscreen(b)
            | Node::UIGrid(_, _, b)
            | Node::UIScrollArea(_, b) => {
                self.check_node(b);
            }
            Node::DrawText(t, x, y, s, c) => {
                self.check_node(t);
                self.check_node(x);
                self.check_node(y);
                self.check_node(s);
                self.check_node(c);
            }
            // Literals & Constants
            Node::IntLiteral(_)
            | Node::FloatLiteral(_)
            | Node::BoolLiteral(_)
            | Node::StringLiteral(_)
            | Node::Identifier(_)
            | Node::Time
            | Node::MapCreate
            | Node::InitGraphics
            | Node::InitAudio
            | Node::GetLastKeypress
            | Node::InitVoxelMap
            | Node::StopNote(_) => {}
        }
    }
}
