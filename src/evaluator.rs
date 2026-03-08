use crate::executor::{ExecutionEngine, RelType, ExecResult, StackFrame};
use crate::ast::Node;
use std::collections::HashMap;

impl ExecutionEngine {
    pub fn evaluate(&mut self, node: &Node) -> ExecResult {
        let res = self.evaluate_inner(node);
        if let ExecResult::Fault(ref err) = res {
            if err.contains("Permission Denied") || err.contains("Sandbox") {
                self.permission_fault = Some(err.clone());
            }
        }
        res
    }

    pub fn evaluate_inner(&mut self, node: &Node) -> ExecResult {
        match node {
            // Literals
            Node::IntLiteral(v) => ExecResult::Value(RelType::Int(*v)),
            Node::FloatLiteral(v) => ExecResult::Value(RelType::Float(*v)),
            Node::BoolLiteral(v) => ExecResult::Value(RelType::Bool(*v)),
            Node::StringLiteral(v) => ExecResult::Value(RelType::Str(v.clone())),

            // Memory & Variables
            Node::Identifier(name) => {
                if let Some(v) = self.get_var(name) { ExecResult::Value(v) }
                else { ExecResult::Fault(format!("Variable '{}' not found", name)) }
            }
            Node::Assign(name, expr) => {
                match self.evaluate_inner(expr) {
                    ExecResult::Value(v) => { self.set_var(name.clone(), v.clone()); ExecResult::Value(v) }
                    ExecResult::ReturnBlockInfo(v) => { self.set_var(name.clone(), v.clone()); ExecResult::Value(v) }
                    err => err,
                }
            }

            // Math & Logic
            Node::Add(l, r) => self.do_math(l, '+', r),
            Node::Sub(l, r) => self.do_math(l, '-', r),
            Node::Mul(l, r) => self.do_math(l, '*', r),
            Node::Div(l, r) => self.do_math(l, '/', r),
            Node::Abs(expr) => {
                match self.evaluate_inner(expr) {
                    ExecResult::Value(RelType::Int(v)) => ExecResult::Value(RelType::Int(v.abs())),
                    ExecResult::Value(RelType::Float(v)) => ExecResult::Value(RelType::Float(v.abs())),
                    ExecResult::Value(_) => ExecResult::Fault("Abs expects number".into()),
                    err => err,
                }
            }
            Node::Sin(expr) => {
                match self.evaluate_inner(expr) {
                    ExecResult::Value(RelType::Float(v)) => ExecResult::Value(RelType::Float(v.sin())),
                    ExecResult::Value(_) => ExecResult::Fault("Sin expects float".into()),
                    err => err,
                }
            }
            Node::Cos(expr) => {
                match self.evaluate_inner(expr) {
                    ExecResult::Value(RelType::Float(v)) => ExecResult::Value(RelType::Float(v.cos())),
                    ExecResult::Value(_) => ExecResult::Fault("Cos expects float".into()),
                    err => err,
                }
            }
            Node::Eq(l, r) => self.do_compare(l, "==", r),
            Node::Lt(l, r) => self.do_compare(l, "<", r),
            Node::Gt(l, r) => self.do_compare(l, ">", r),
            Node::Time | Node::GlobalTime => ExecResult::Value(RelType::Float(self.startup_time.elapsed().as_secs_f64())),
            Node::Mat4Mul(_l, _r) => ExecResult::Fault("Mat4Mul logic not yet restored".into()),

            // Data Structures: Arrays
            Node::ArrayCreate(nodes) => {
                let mut vals = Vec::with_capacity(nodes.len());
                for n in nodes {
                    match self.evaluate_inner(n) {
                        ExecResult::Value(v) => vals.push(v),
                        err => return err,
                    }
                }
                ExecResult::Value(RelType::Array(vals))
            }
            Node::ArrayGet(arr, idx) => {
                let a = match self.evaluate_inner(arr) { ExecResult::Value(RelType::Array(v)) => v, _ => return ExecResult::Fault("Target is not an array".into()) };
                let i = match self.evaluate_inner(idx) { ExecResult::Value(RelType::Int(v)) => v as usize, _ => return ExecResult::Fault("Index is not an integer".into()) };
                if i < a.len() { ExecResult::Value(a[i].clone()) }
                else { ExecResult::Fault(format!("Index {} out of bounds", i)) }
            }
            Node::ArraySet(arr_expr, idx_expr, val_expr) => {
                let val = match self.evaluate_inner(val_expr) { ExecResult::Value(v) => v, err => return err };
                if let Node::Identifier(name) = &**arr_expr {
                    let mut a = match self.get_var(name) { Some(RelType::Array(v)) => v, _ => return ExecResult::Fault("Target is not an array".into()) };
                    let i = match self.evaluate_inner(idx_expr) { ExecResult::Value(RelType::Int(v)) => v as usize, _ => return ExecResult::Fault("Index is not an integer".into()) };
                    if i < a.len() { 
                        let old = std::mem::replace(&mut a[i], val.clone());
                        self.release_handles(&old);
                        self.set_var(name.clone(), RelType::Array(a));
                        ExecResult::Value(val)
                    } else { ExecResult::Fault(format!("Index {} out of bounds", i)) }
                } else { ExecResult::Fault("ArraySet only supported on identifiers currently".into()) }
            }
            Node::ArrayPush(arr_expr, val_expr) => {
                let val = match self.evaluate_inner(val_expr) { ExecResult::Value(v) => v, err => return err };
                if let Node::Identifier(name) = &**arr_expr {
                    let mut a = match self.get_var(name) { Some(RelType::Array(v)) => v, _ => return ExecResult::Fault("Target is not an array".into()) };
                    a.push(val.clone());
                    self.set_var(name.clone(), RelType::Array(a));
                    ExecResult::Value(val)
                } else { ExecResult::Fault("ArrayPush only supported on identifiers currently".into()) }
            }
            Node::ArrayLen(arr) => {
                let a = match self.evaluate_inner(arr) { ExecResult::Value(RelType::Array(v)) => v, _ => return ExecResult::Fault("Target is not an array".into()) };
                ExecResult::Value(RelType::Int(a.len() as i64))
            }

            // Data Structures: Maps & Objects
            Node::MapCreate => ExecResult::Value(RelType::Object(HashMap::new())),
            Node::MapGet(map_expr, key_expr) => {
                let m = match self.evaluate_inner(map_expr) { ExecResult::Value(RelType::Object(v)) => v, _ => return ExecResult::Fault("Target is not a map/object".into()) };
                let k = match self.evaluate_inner(key_expr) { ExecResult::Value(RelType::Str(v)) => v, _ => return ExecResult::Fault("Key is not a string".into()) };
                if let Some(v) = m.get(&k) { ExecResult::Value(v.clone()) }
                else { ExecResult::Value(RelType::Void) }
            }
            Node::MapSet(map_expr, key_expr, val_expr) => {
                let val = match self.evaluate_inner(val_expr) { ExecResult::Value(v) => v, err => return err };
                if let Node::Identifier(name) = &**map_expr {
                    let mut m = match self.get_var(name) { Some(RelType::Object(v)) => v, _ => return ExecResult::Fault("Target is not a map/object".into()) };
                    let k = match self.evaluate_inner(key_expr) { ExecResult::Value(RelType::Str(v)) => v, _ => return ExecResult::Fault("Key is not a string".into()) };
                    if let Some(old) = m.insert(k, val.clone()) { self.release_handles(&old); }
                    self.set_var(name.clone(), RelType::Object(m));
                    ExecResult::Value(val)
                } else { ExecResult::Fault("MapSet only supported on identifiers currently".into()) }
            }
            Node::MapHasKey(map_expr, key_expr) => {
                let m = match self.evaluate_inner(map_expr) { ExecResult::Value(RelType::Object(v)) => v, _ => return ExecResult::Fault("Target is not a map/object".into()) };
                let k = match self.evaluate_inner(key_expr) { ExecResult::Value(RelType::Str(v)) => v, _ => return ExecResult::Fault("Key is not a string".into()) };
                ExecResult::Value(RelType::Bool(m.contains_key(&k)))
            }
            Node::ObjectLiteral(map) => {
                let mut res = HashMap::with_capacity(map.len());
                for (k, v_node) in map {
                    match self.evaluate_inner(v_node) {
                        ExecResult::Value(v) => { res.insert(k.clone(), v); }
                        err => return err,
                    }
                }
                ExecResult::Value(RelType::Object(res))
            }
            Node::PropertyGet(obj_expr, prop) => {
                let o = match self.evaluate_inner(obj_expr) { ExecResult::Value(RelType::Object(v)) => v, _ => return ExecResult::Fault("Target is not an object".into()) };
                ExecResult::Value(o.get(prop).cloned().unwrap_or(RelType::Void))
            }
            Node::PropertySet(obj_expr, prop, val_expr) => {
                let val = match self.evaluate_inner(val_expr) { ExecResult::Value(v) => v, err => return err };
                if let Node::Identifier(name) = &**obj_expr {
                    let mut o = match self.get_var(name) { Some(RelType::Object(v)) => v, _ => return ExecResult::Fault("Target is not an object".into()) };
                    if let Some(old) = o.insert(prop.clone(), val.clone()) { self.release_handles(&old); }
                    self.set_var(name.clone(), RelType::Object(o));
                    ExecResult::Value(val)
                } else { ExecResult::Fault("PropertySet only supported on identifiers currently".into()) }
            }
            Node::Index(container, idx) => {
                let c = match self.evaluate_inner(container) { ExecResult::Value(v) => v, err => return err };
                let i = match self.evaluate_inner(idx) { ExecResult::Value(v) => v, err => return err };
                match (c, i) {
                    (RelType::Array(a), RelType::Int(idx)) => {
                        if (idx as usize) < a.len() { ExecResult::Value(a[idx as usize].clone()) }
                        else { ExecResult::Fault("Index out of bounds".into()) }
                    }
                    (RelType::Object(m), RelType::Str(key)) => {
                        ExecResult::Value(m.get(&key).cloned().unwrap_or(RelType::Void))
                    }
                    (RelType::Str(s), RelType::Int(idx)) => {
                        if let Some(ch) = s.chars().nth(idx as usize) { ExecResult::Value(RelType::Str(ch.to_string())) }
                        else { ExecResult::Fault("String index out of bounds".into()) }
                    }
                    _ => ExecResult::Fault("Invalid index operation".into()),
                }
            }
            Node::Concat(l, r) => {
                let lv = match self.evaluate_inner(l) { ExecResult::Value(v) => v, err => return err };
                let rv = match self.evaluate_inner(r) { ExecResult::Value(v) => v, err => return err };
                match (lv, rv) {
                    (RelType::Str(a), RelType::Str(b)) => ExecResult::Value(RelType::Str(a + &b)),
                    (RelType::Array(mut a), RelType::Array(b)) => { a.extend(b); ExecResult::Value(RelType::Array(a)) }
                    _ => ExecResult::Fault("Concat expects strings or arrays".into()),
                }
            }

            // Bitwise
            Node::BitAnd(l, r) => {
                match (self.evaluate_inner(l), self.evaluate_inner(r)) {
                    (ExecResult::Value(RelType::Int(a)), ExecResult::Value(RelType::Int(b))) => ExecResult::Value(RelType::Int(a & b)),
                    _ => ExecResult::Fault("Bitwise AND expects integers".into()),
                }
            }
            Node::BitShiftLeft(l, r) => {
                match (self.evaluate_inner(l), self.evaluate_inner(r)) {
                    (ExecResult::Value(RelType::Int(a)), ExecResult::Value(RelType::Int(b))) => ExecResult::Value(RelType::Int(a << b)),
                    _ => ExecResult::Fault("Bitwise SHL expects integers".into()),
                }
            }
            Node::BitShiftRight(l, r) => {
                match (self.evaluate_inner(l), self.evaluate_inner(r)) {
                    (ExecResult::Value(RelType::Int(a)), ExecResult::Value(RelType::Int(b))) => ExecResult::Value(RelType::Int(a >> b)),
                    _ => ExecResult::Fault("Bitwise SHR expects integers".into()),
                }
            }

            // Control Flow
            Node::If(cond, then_b, else_b) => {
                match self.evaluate_inner(cond) {
                    ExecResult::Value(RelType::Bool(true)) => self.evaluate_inner(then_b),
                    ExecResult::Value(RelType::Bool(false)) => {
                        if let Some(eb) = else_b { self.evaluate_inner(eb) }
                        else { ExecResult::Value(RelType::Void) }
                    }
                    _ => ExecResult::Fault("If condition must be boolean".into()),
                }
            }
            Node::While(cond, body) => {
                while let ExecResult::Value(RelType::Bool(true)) = self.evaluate_inner(cond) {
                    match self.evaluate_inner(body) {
                        ExecResult::Value(v) => self.release_handles(&v),
                        ExecResult::ReturnBlockInfo(v) => return ExecResult::ReturnBlockInfo(v),
                        ExecResult::Fault(e) => return ExecResult::Fault(e),
                    }
                }
                ExecResult::Value(RelType::Void)
            }
            Node::Block(nodes) => {
                let mut last_val = RelType::Void;
                let len = nodes.len();
                for (i, n) in nodes.iter().enumerate() {
                    match self.evaluate_inner(n) {
                        ExecResult::Value(v) => {
                            if i < len - 1 { self.release_handles(&v); }
                            else { last_val = v; }
                        }
                        ExecResult::ReturnBlockInfo(v) => return ExecResult::ReturnBlockInfo(v),
                        ExecResult::Fault(e) => return ExecResult::Fault(e),
                    }
                }
                ExecResult::Value(last_val)
            }
            Node::Return(expr) => {
                let v = match self.evaluate_inner(&*expr) { ExecResult::Value(v) => v, err => return err };
                ExecResult::ReturnBlockInfo(v)
            }

            // Functions
            Node::FnDef(name, params, body) => {
                self.set_var(name.clone(), RelType::FnDef(name.clone(), params.clone(), body.clone()));
                ExecResult::Value(RelType::Void)
            }
            Node::Call(name, args) => {
                let func = if let Some(f) = self.get_var(name) { f } else { return ExecResult::Fault(format!("Function '{}' not found", name)) };
                match func {
                    RelType::FnDef(_, params, body) => {
                        if params.len() != args.len() { return ExecResult::Fault(format!("'{}' expects {} args, got {}", name, params.len(), args.len())) }
                        let mut locals = HashMap::with_capacity(params.len());
                        for (p, a) in params.iter().zip(args.iter()) {
                            match self.evaluate_inner(a) {
                                ExecResult::Value(v) => { locals.insert(p.clone(), v); }
                                err => return err,
                            }
                        }
                        self.call_stack.push(StackFrame { locals });
                        let res = self.evaluate_inner(&body);
                        if let Some(frame) = self.call_stack.pop() {
                            for (_, val) in frame.locals { self.release_handles(&val); }
                        }
                        match res {
                            ExecResult::ReturnBlockInfo(v) => ExecResult::Value(v),
                            other => other,
                        }
                    }
                    _ => ExecResult::Fault(format!("'{}' is not a function", name)),
                }
            }

            // Special Physics
            Node::CheckCollision { a_min, a_max, b_min, b_max } => {
                let am = match self.evaluate_inner(a_min) { ExecResult::Value(v) => v, err => return err };
                let ax = match self.evaluate_inner(a_max) { ExecResult::Value(v) => v, err => return err };
                let bm = match self.evaluate_inner(b_min) { ExecResult::Value(v) => v, err => return err };
                let bx = match self.evaluate_inner(b_max) { ExecResult::Value(v) => v, err => return err };
                let v_am = if let Some(v) = self.to_vec3(am) { v } else { return ExecResult::Fault("a_min must be array".into()) };
                let v_ax = if let Some(v) = self.to_vec3(ax) { v } else { return ExecResult::Fault("a_max must be array".into()) };
                let v_bm = if let Some(v) = self.to_vec3(bm) { v } else { return ExecResult::Fault("b_min must be array".into()) };
                let v_bx = if let Some(v) = self.to_vec3(bx) { v } else { return ExecResult::Fault("b_max must be array".into()) };
                let aabb_a = crate::math::AABB::new(v_am, v_ax);
                let aabb_b = crate::math::AABB::new(v_bm, v_bx);
                ExecResult::Value(RelType::Int(if aabb_a.intersects(&aabb_b) { 1 } else { 0 }))
            }

            // Delegation
            Node::ToString(expr) => {
                match self.evaluate_inner(expr) {
                    ExecResult::Value(v) => ExecResult::Value(RelType::Str(v.to_string())),
                    err => err,
                }
            }

            // Exhaustive Delegation of Effectful / System nodes to executor
            Node::FileRead(_) | Node::FileWrite(_, _) | Node::FSRead(_) | Node::FSWrite(_, _) |
            Node::Print(_) | Node::Store { .. } | Node::Load { .. } |
            Node::DrawRect { .. } | Node::UIFixed { .. } | Node::UIFillParent |
            Node::RenderCanvas { .. } | Node::Transform2D { .. } | Node::Sprite2D { .. } |
            Node::Camera3D { .. } | Node::Mesh3D { .. } | Node::PointLight3D { .. } | Node::Material3D { .. } |
            Node::MeshInstance3D { .. } | Node::FPSCamera { .. } | Node::MouseGrab { .. } | Node::RaycastSimple |
            Node::WeaponViewModel { .. } | Node::Fetch { .. } | Node::Extract { .. } |
            Node::EvalJSONNative(_) | Node::NativeCall(_, _) | Node::ExternCall { .. } |
            Node::InitWindow(_, _, _) | Node::InitGraphics | Node::LoadShader(_) | Node::RenderMesh(_, _, _) |
            Node::PollEvents(_) | Node::InitAudio | Node::PlayNote(_, _, _) | Node::StopNote(_) |
            Node::PlayAudioFile(_) | Node::LoadMesh(_) | Node::LoadTexture(_) | Node::RenderAsset(_, _, _, _) |
            Node::LoadFont(_) | Node::DrawText(_, _, _, _, _) | Node::GetLastKeypress |
            Node::UIWindow(_, _, _) | Node::UILabel(_) | Node::UIButton(_) | Node::UITextInput(_) |
            Node::UISetStyle(_, _, _, _, _, _) | Node::UIHorizontal(_) | Node::UIFullscreen(_) |
            Node::UIGrid(_, _, _) | Node::UIScrollArea(_, _) | Node::InitCamera(_) |
            Node::DrawVoxelGrid(_) | Node::LoadTextureAtlas(_, _) | Node::LoadSample(_, _) |
            Node::PlaySample(_, _, _) | Node::InitVoxelMap | Node::SetVoxel(_, _, _, _) |
            Node::EnableInteraction(_) | Node::EnablePhysics(_) | Node::Import(_) |
            Node::AddWorldAABB { .. } => self.evaluate_extra(node),
        }
    }

    pub fn do_math(&mut self, left: &Node, op: char, right: &Node) -> ExecResult {
        let lv = match self.evaluate_inner(left) { ExecResult::Value(v) => v, err => return err };
        let rv = match self.evaluate_inner(right) { ExecResult::Value(v) => v, err => return err };
        let res = match op {
            '+' => match (lv, rv) {
                (RelType::Int(a), RelType::Int(b)) => RelType::Int(a + b),
                (RelType::Float(a), RelType::Float(b)) => RelType::Float(a + b),
                (RelType::Str(a), RelType::Str(b)) => RelType::Str(a + &b),
                _ => return ExecResult::Fault("Invalid types for +".into()),
            },
            '-' => match (lv, rv) {
                (RelType::Int(a), RelType::Int(b)) => RelType::Int(a - b),
                (RelType::Float(a), RelType::Float(b)) => RelType::Float(a - b),
                _ => return ExecResult::Fault("Invalid types for -".into()),
            },
            '*' => match (lv, rv) {
                (RelType::Int(a), RelType::Int(b)) => RelType::Int(a * b),
                (RelType::Float(a), RelType::Float(b)) => RelType::Float(a * b),
                _ => return ExecResult::Fault("Invalid types for *".into()),
            },
            '/' => match (lv, rv) {
                (RelType::Int(a), RelType::Int(b)) => { if b == 0 { return ExecResult::Fault("Div by zero".into()) } RelType::Int(a / b) },
                (RelType::Float(a), RelType::Float(b)) => RelType::Float(a / b),
                _ => return ExecResult::Fault("Invalid types for /".into()),
            },
            _ => return ExecResult::Fault(format!("Unknown operator: {}", op)),
        };
        ExecResult::Value(res)
    }

    pub fn do_compare(&mut self, left: &Node, op: &str, right: &Node) -> ExecResult {
        let lv = match self.evaluate_inner(left) { ExecResult::Value(v) => v, err => return err };
        let rv = match self.evaluate_inner(right) { ExecResult::Value(v) => v, err => return err };
        let res = match op {
            "==" => RelType::Bool(lv == rv),
            "<" => match (lv, rv) {
                (RelType::Int(a), RelType::Int(b)) => RelType::Bool(a < b),
                (RelType::Float(a), RelType::Float(b)) => RelType::Bool(a < b),
                _ => return ExecResult::Fault("Invalid types for <".into()),
            },
            ">" => match (lv, rv) {
                (RelType::Int(a), RelType::Int(b)) => RelType::Bool(a > b),
                (RelType::Float(a), RelType::Float(b)) => RelType::Bool(a > b),
                _ => return ExecResult::Fault("Invalid types for >".into()),
            },
            _ => return ExecResult::Fault(format!("Unknown comparison: {}", op)),
        };
        ExecResult::Value(res)
    }

    pub(crate) fn to_vec3(&self, val: RelType) -> Option<[f32; 3]> {
        if let RelType::Array(arr) = val {
            if arr.len() >= 3 {
                let x = match arr[0] { RelType::Float(f) => f as f32, RelType::Int(i) => i as f32, _ => 0.0 };
                let y = match arr[1] { RelType::Float(f) => f as f32, RelType::Int(i) => i as f32, _ => 0.0 };
                let z = match arr[2] { RelType::Float(f) => f as f32, RelType::Int(i) => i as f32, _ => 0.0 };
                return Some([x, y, z]);
            }
        }
        None
    }
}
