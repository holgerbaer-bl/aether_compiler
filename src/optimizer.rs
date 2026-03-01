use crate::ast::Node;

pub fn count_nodes(node: &Node) -> usize {
    let mut count = 1;
    match node {
        Node::IntLiteral(_)
        | Node::FloatLiteral(_)
        | Node::BoolLiteral(_)
        | Node::StringLiteral(_)
        | Node::Identifier(_)
        | Node::Time
        | Node::InitGraphics
        | Node::InitVoxelMap
        | Node::InitAudio
        | Node::GetLastKeypress
        | Node::ArrayLen(_)
        | Node::Import(_) => {}

        Node::Add(l, r)
        | Node::Sub(l, r)
        | Node::Mul(l, r)
        | Node::Div(l, r)
        | Node::Eq(l, r)
        | Node::Lt(l, r)
        | Node::Gt(l, r)
        | Node::BitAnd(l, r)
        | Node::BitShiftLeft(l, r)
        | Node::BitShiftRight(l, r)
        | Node::Index(l, r)
        | Node::Concat(l, r)
        | Node::Mat4Mul(l, r)
        | Node::ArraySet(_, l, r)
        | Node::FileWrite(l, r)
        | Node::UIWindow(l, r)
        | Node::LoadTextureAtlas(l, r)
        | Node::LoadSample(l, r) => {
            count += count_nodes(l) + count_nodes(r);
        }

        Node::Assign(_, val)
        | Node::ArrayGet(_, val)
        | Node::ArrayPush(_, val)
        | Node::FileRead(val)
        | Node::Print(val)
        | Node::EvalJSONNative(val)
        | Node::ToString(val)
        | Node::LoadShader(val)
        | Node::PollEvents(val)
        | Node::PropertyGet(val, _)
        | Node::PropertySet(_, _, val)
        | Node::StopNote(val)
        | Node::LoadMesh(val)
        | Node::LoadTexture(val)
        | Node::PlayAudioFile(val)
        | Node::LoadFont(val)
        | Node::UILabel(val)
        | Node::UIButton(val)
        | Node::UITextInput(val)
        | Node::InitCamera(val)
        | Node::DrawVoxelGrid(val)
        | Node::EnableInteraction(val)
        | Node::EnablePhysics(val)
        | Node::Return(val)
        | Node::Sin(val)
        | Node::Cos(val) => {
            count += count_nodes(val);
        }

        Node::If(cond, then_b, else_b) => {
            count += count_nodes(cond) + count_nodes(then_b);
            if let Some(eb) = else_b {
                count += count_nodes(eb);
            }
        }
        Node::While(cond, body) => {
            count += count_nodes(cond) + count_nodes(body);
        }
        Node::Block(nodes)
        | Node::ArrayLiteral(nodes)
        | Node::Call(_, nodes)
        | Node::NativeCall(_, nodes) => {
            for n in nodes {
                count += count_nodes(n);
            }
        }
        Node::ObjectLiteral(map) => {
            for v in map.values() {
                count += count_nodes(v);
            }
        }
        Node::ExternCall {
            module: _,
            function: _,
            args,
        } => {
            for n in args {
                count += count_nodes(n);
            }
        }
        Node::FnDef(_, _, body) => {
            count += count_nodes(body);
        }
        Node::InitWindow(w, h, t)
        | Node::RenderMesh(w, h, t)
        | Node::PlayNote(w, h, t)
        | Node::PlaySample(w, h, t) => {
            count += count_nodes(w) + count_nodes(h) + count_nodes(t);
        }
        Node::RenderAsset(a, b, c, d) | Node::SetVoxel(a, b, c, d) => {
            count += count_nodes(a) + count_nodes(b) + count_nodes(c) + count_nodes(d);
        }
        Node::DrawText(a, b, c, d, e) => {
            count +=
                count_nodes(a) + count_nodes(b) + count_nodes(c) + count_nodes(d) + count_nodes(e);
        }
    }
    count
}

pub fn optimize(node: Node) -> Node {
    match node {
        Node::IntLiteral(v) => Node::IntLiteral(v),
        Node::FloatLiteral(v) => Node::FloatLiteral(v),
        Node::BoolLiteral(v) => Node::BoolLiteral(v),
        Node::StringLiteral(v) => Node::StringLiteral(v),
        Node::Identifier(name) => Node::Identifier(name),
        Node::Import(path) => Node::Import(path),
        Node::Time => Node::Time,
        Node::InitGraphics => Node::InitGraphics,
        Node::InitVoxelMap => Node::InitVoxelMap,
        Node::InitAudio => Node::InitAudio,
        Node::GetLastKeypress => Node::GetLastKeypress,

        // Math Folding
        Node::Add(l, r) => optimize_math_op(*l, *r, '+'),
        Node::Sub(l, r) => optimize_math_op(*l, *r, '-'),
        Node::Mul(l, r) => optimize_math_op(*l, *r, '*'),
        Node::Div(l, r) => optimize_math_op(*l, *r, '/'),

        // Logic Folding
        Node::Eq(l, r) => optimize_eq(*l, *r),
        Node::Lt(l, r) => optimize_lt(*l, *r),
        Node::Gt(l, r) => optimize_gt(*l, *r),

        // Bitwise Folding
        Node::BitAnd(l, r) => optimize_bitwise(*l, *r, '&'),
        Node::BitShiftLeft(l, r) => optimize_bitwise(*l, *r, '<'),
        Node::BitShiftRight(l, r) => optimize_bitwise(*l, *r, '>'),

        // Dead Code Elimination
        Node::If(cond, then_branch, else_branch) => {
            let opt_cond = optimize(*cond);
            match opt_cond {
                Node::BoolLiteral(true) => optimize(*then_branch),
                Node::BoolLiteral(false) => {
                    if let Some(eb) = else_branch {
                        optimize(*eb)
                    } else {
                        Node::Block(vec![])
                    }
                }
                _ => Node::If(
                    Box::new(opt_cond),
                    Box::new(optimize(*then_branch)),
                    else_branch.map(|eb| Box::new(optimize(*eb))),
                ),
            }
        }
        Node::While(cond, body) => {
            let opt_cond = optimize(*cond);
            match opt_cond {
                Node::BoolLiteral(false) => Node::Block(vec![]),
                _ => Node::While(Box::new(opt_cond), Box::new(optimize(*body))),
            }
        }
        Node::Block(nodes) => {
            let opt_nodes: Vec<Node> = nodes.into_iter().map(optimize).collect();
            Node::Block(opt_nodes)
        }

        // Standard Traversals
        Node::FnDef(name, params, body) => Node::FnDef(name, params, Box::new(optimize(*body))),
        Node::Call(name, args) => Node::Call(name, args.into_iter().map(optimize).collect()),
        Node::NativeCall(name, args) => {
            Node::NativeCall(name, args.into_iter().map(optimize).collect())
        }
        Node::ExternCall {
            module,
            function,
            args,
        } => Node::ExternCall {
            module,
            function,
            args: args.into_iter().map(optimize).collect(),
        },

        Node::Assign(name, val) => Node::Assign(name, Box::new(optimize(*val))),
        Node::ArrayLiteral(elements) => {
            Node::ArrayLiteral(elements.into_iter().map(optimize).collect())
        }
        Node::ArrayGet(name, idx) => Node::ArrayGet(name, Box::new(optimize(*idx))),
        Node::ArraySet(name, idx, val) => {
            Node::ArraySet(name, Box::new(optimize(*idx)), Box::new(optimize(*val)))
        }
        Node::ArrayPush(name, val) => Node::ArrayPush(name, Box::new(optimize(*val))),
        Node::ArrayLen(name) => Node::ArrayLen(name),
        Node::Index(arr, idx) => Node::Index(Box::new(optimize(*arr)), Box::new(optimize(*idx))),
        Node::Concat(l, r) => Node::Concat(Box::new(optimize(*l)), Box::new(optimize(*r))),

        Node::ObjectLiteral(map) => {
            let mut opt_map = std::collections::HashMap::new();
            for (k, v) in map {
                opt_map.insert(k, optimize(v));
            }
            Node::ObjectLiteral(opt_map)
        }
        Node::PropertyGet(obj, prop) => Node::PropertyGet(Box::new(optimize(*obj)), prop),
        Node::PropertySet(obj, prop, val) => {
            Node::PropertySet(Box::new(optimize(*obj)), prop, Box::new(optimize(*val)))
        }

        Node::Return(val) => Node::Return(Box::new(optimize(*val))),
        Node::Sin(val) => Node::Sin(Box::new(optimize(*val))),
        Node::Cos(val) => Node::Cos(Box::new(optimize(*val))),

        Node::Mat4Mul(l, r) => Node::Mat4Mul(Box::new(optimize(*l)), Box::new(optimize(*r))),
        Node::FileRead(path) => Node::FileRead(Box::new(optimize(*path))),
        Node::FileWrite(path, content) => {
            Node::FileWrite(Box::new(optimize(*path)), Box::new(optimize(*content)))
        }
        Node::Print(val) => Node::Print(Box::new(optimize(*val))),
        Node::EvalJSONNative(val) => Node::EvalJSONNative(Box::new(optimize(*val))),
        Node::ToString(val) => Node::ToString(Box::new(optimize(*val))),

        Node::InitWindow(w, h, t) => Node::InitWindow(
            Box::new(optimize(*w)),
            Box::new(optimize(*h)),
            Box::new(optimize(*t)),
        ),
        Node::LoadShader(val) => Node::LoadShader(Box::new(optimize(*val))),
        Node::RenderMesh(s, v, m) => Node::RenderMesh(
            Box::new(optimize(*s)),
            Box::new(optimize(*v)),
            Box::new(optimize(*m)),
        ),
        Node::PollEvents(body) => Node::PollEvents(Box::new(optimize(*body))),

        Node::PlayNote(c, f, w) => Node::PlayNote(
            Box::new(optimize(*c)),
            Box::new(optimize(*f)),
            Box::new(optimize(*w)),
        ),
        Node::StopNote(c) => Node::StopNote(Box::new(optimize(*c))),

        Node::LoadMesh(p) => Node::LoadMesh(Box::new(optimize(*p))),
        Node::LoadTexture(p) => Node::LoadTexture(Box::new(optimize(*p))),
        Node::PlayAudioFile(p) => Node::PlayAudioFile(Box::new(optimize(*p))),
        Node::RenderAsset(s, m, t, u) => Node::RenderAsset(
            Box::new(optimize(*s)),
            Box::new(optimize(*m)),
            Box::new(optimize(*t)),
            Box::new(optimize(*u)),
        ),

        Node::LoadFont(p) => Node::LoadFont(Box::new(optimize(*p))),
        Node::DrawText(t, x, y, s, c) => Node::DrawText(
            Box::new(optimize(*t)),
            Box::new(optimize(*x)),
            Box::new(optimize(*y)),
            Box::new(optimize(*s)),
            Box::new(optimize(*c)),
        ),

        Node::UIWindow(t, b) => Node::UIWindow(Box::new(optimize(*t)), Box::new(optimize(*b))),
        Node::UILabel(t) => Node::UILabel(Box::new(optimize(*t))),
        Node::UIButton(t) => Node::UIButton(Box::new(optimize(*t))),
        Node::UITextInput(v) => Node::UITextInput(Box::new(optimize(*v))),

        Node::InitCamera(f) => Node::InitCamera(Box::new(optimize(*f))),
        Node::DrawVoxelGrid(v) => Node::DrawVoxelGrid(Box::new(optimize(*v))),
        Node::LoadTextureAtlas(p, s) => {
            Node::LoadTextureAtlas(Box::new(optimize(*p)), Box::new(optimize(*s)))
        }
        Node::LoadSample(id, p) => {
            Node::LoadSample(Box::new(optimize(*id)), Box::new(optimize(*p)))
        }
        Node::PlaySample(id, v, p) => Node::PlaySample(
            Box::new(optimize(*id)),
            Box::new(optimize(*v)),
            Box::new(optimize(*p)),
        ),
        Node::SetVoxel(x, y, z, id) => Node::SetVoxel(
            Box::new(optimize(*x)),
            Box::new(optimize(*y)),
            Box::new(optimize(*z)),
            Box::new(optimize(*id)),
        ),
        Node::EnableInteraction(b) => Node::EnableInteraction(Box::new(optimize(*b))),
        Node::EnablePhysics(b) => Node::EnablePhysics(Box::new(optimize(*b))),
    }
}

fn optimize_math_op(left: Node, right: Node, op: char) -> Node {
    let opt_l = optimize(left);
    let opt_r = optimize(right);

    match (&opt_l, &opt_r) {
        (Node::IntLiteral(l), Node::IntLiteral(r)) => match op {
            '+' => Node::IntLiteral(l + r),
            '-' => Node::IntLiteral(l - r),
            '*' => Node::IntLiteral(l * r),
            '/' => {
                if *r != 0 {
                    Node::IntLiteral(l / r)
                } else {
                    Node::Div(Box::new(opt_l), Box::new(opt_r))
                }
            }
            _ => unreachable!(),
        },
        (Node::FloatLiteral(l), Node::FloatLiteral(r)) => match op {
            '+' => Node::FloatLiteral(l + r),
            '-' => Node::FloatLiteral(l - r),
            '*' => Node::FloatLiteral(l * r),
            '/' => {
                if *r != 0.0 {
                    Node::FloatLiteral(l / r)
                } else {
                    Node::Div(Box::new(opt_l), Box::new(opt_r))
                }
            }
            _ => unreachable!(),
        },
        _ => match op {
            '+' => Node::Add(Box::new(opt_l), Box::new(opt_r)),
            '-' => Node::Sub(Box::new(opt_l), Box::new(opt_r)),
            '*' => Node::Mul(Box::new(opt_l), Box::new(opt_r)),
            '/' => Node::Div(Box::new(opt_l), Box::new(opt_r)),
            _ => unreachable!(),
        },
    }
}

fn optimize_eq(left: Node, right: Node) -> Node {
    let opt_l = optimize(left);
    let opt_r = optimize(right);
    match (&opt_l, &opt_r) {
        (Node::IntLiteral(l), Node::IntLiteral(r)) => Node::BoolLiteral(l == r),
        (Node::FloatLiteral(l), Node::FloatLiteral(r)) => Node::BoolLiteral(l == r),
        (Node::BoolLiteral(l), Node::BoolLiteral(r)) => Node::BoolLiteral(l == r),
        (Node::StringLiteral(l), Node::StringLiteral(r)) => Node::BoolLiteral(l == r),
        _ => Node::Eq(Box::new(opt_l), Box::new(opt_r)),
    }
}

fn optimize_lt(left: Node, right: Node) -> Node {
    let opt_l = optimize(left);
    let opt_r = optimize(right);
    match (&opt_l, &opt_r) {
        (Node::IntLiteral(l), Node::IntLiteral(r)) => Node::BoolLiteral(l < r),
        (Node::FloatLiteral(l), Node::FloatLiteral(r)) => Node::BoolLiteral(l < r),
        _ => Node::Lt(Box::new(opt_l), Box::new(opt_r)),
    }
}

fn optimize_gt(left: Node, right: Node) -> Node {
    let opt_l = optimize(left);
    let opt_r = optimize(right);
    match (&opt_l, &opt_r) {
        (Node::IntLiteral(l), Node::IntLiteral(r)) => Node::BoolLiteral(l > r),
        (Node::FloatLiteral(l), Node::FloatLiteral(r)) => Node::BoolLiteral(l > r),
        _ => Node::Gt(Box::new(opt_l), Box::new(opt_r)),
    }
}

fn optimize_bitwise(left: Node, right: Node, op: char) -> Node {
    let opt_l = optimize(left);
    let opt_r = optimize(right);
    match (&opt_l, &opt_r) {
        (Node::IntLiteral(l), Node::IntLiteral(r)) => match op {
            '&' => Node::IntLiteral(l & r),
            '<' => Node::IntLiteral(l << r),
            '>' => Node::IntLiteral(l >> r),
            _ => unreachable!(),
        },
        _ => match op {
            '&' => Node::BitAnd(Box::new(opt_l), Box::new(opt_r)),
            '<' => Node::BitShiftLeft(Box::new(opt_l), Box::new(opt_r)),
            '>' => Node::BitShiftRight(Box::new(opt_l), Box::new(opt_r)),
            _ => unreachable!(),
        },
    }
}

// ---------------------------------------------------------
// TYPE INFERENCE ENGINE (SPRINT 26)
// ---------------------------------------------------------
use crate::ast::Type;
use std::collections::HashMap;

pub struct TypeChecker {
    pub scopes: Vec<HashMap<String, Type>>,
    pub errors: Vec<String>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            errors: Vec::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn set_var(&mut self, name: &str, t: Type) {
        // If it exists in any scope, check if the type matches. But we need to find where it is.
        for scope in self.scopes.iter_mut().rev() {
            if let Some(existing_type) = scope.get(name) {
                if *existing_type != t && *existing_type != Type::Any && t != Type::Any {
                    self.errors.push(format!(
                        "TypeError: Variable '{}' was previously assigned as {:?} but is now being assigned {:?}",
                        name, existing_type, t
                    ));
                }
                return; // Updated or conflicted
            }
        }
        // Is a new variable
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), t);
        }
    }

    pub fn get_var(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t.clone());
            }
        }
        None
    }

    pub fn check(&mut self, node: &Node) -> Result<Type, String> {
        match node {
            Node::IntLiteral(_) => Ok(Type::Int),
            Node::FloatLiteral(_) => Ok(Type::Float),
            Node::BoolLiteral(_) => Ok(Type::Bool),
            Node::StringLiteral(_) => Ok(Type::String),
            Node::ObjectLiteral(_) => Ok(Type::Object),
            Node::ArrayLiteral(_) => Ok(Type::Array),
            Node::Identifier(name) => {
                if let Some(t) = self.get_var(name) {
                    Ok(t)
                } else {
                    Ok(Type::Any) // Unknown variables shouldn't aggressively fail if dynamically placed, or fail. Wait, let's treat as Any
                }
            }
            Node::Time | Node::GetLastKeypress => Ok(Type::Float),

            Node::Assign(name, val_node) => {
                let expr_type = self.check(val_node)?;
                self.set_var(name, expr_type);
                Ok(Type::Void) // Assign doesn't traditionally return type in strict checks
            }

            Node::Add(l, r) | Node::Sub(l, r) | Node::Mul(l, r) | Node::Div(l, r) => {
                let lt = self.check(l)?;
                let rt = self.check(r)?;
                if lt == Type::Handle || rt == Type::Handle {
                    self.errors.push(format!(
                        "TypeError: Cannot perform mathematics on Handle pointers"
                    ));
                }
                if lt != rt && lt != Type::Any && rt != Type::Any {
                    self.errors
                        .push(format!("TypeError: Math mismatch {:?} and {:?}", lt, rt));
                }
                Ok(lt) // Assume left type dominant for now
            }
            Node::Eq(l, r) | Node::Lt(l, r) | Node::Gt(l, r) => {
                let _lt = self.check(l)?;
                let _rt = self.check(r)?;
                Ok(Type::Bool)
            }
            Node::If(cond, then_b, else_b) => {
                let ct = self.check(cond)?;
                if ct != Type::Bool && ct != Type::Any {
                    self.errors.push(format!(
                        "TypeError: 'If' condition expects Bool, found {:?}",
                        ct
                    ));
                }
                self.push_scope();
                self.check(then_b)?;
                self.pop_scope();

                if let Some(eb) = else_b {
                    self.push_scope();
                    self.check(eb)?;
                    self.pop_scope();
                }
                Ok(Type::Void)
            }
            Node::While(cond, body) => {
                let ct = self.check(cond)?;
                if ct != Type::Bool && ct != Type::Any {
                    self.errors.push(format!(
                        "TypeError: 'While' condition expects Bool, found {:?}",
                        ct
                    ));
                }
                self.push_scope();
                self.check(body)?;
                self.pop_scope();
                Ok(Type::Void)
            }
            Node::Block(nodes) => {
                self.push_scope();
                for n in nodes {
                    self.check(n)?;
                }
                self.pop_scope();
                Ok(Type::Void)
            }

            // FFI Extern Call
            Node::ExternCall {
                module: _module,
                function: _function,
                args,
            } => {
                // To safely implement this, we normally look up a signature.
                // For Sprint 26 rules: Argument types must match what NativeModule says.
                // We'll trust run_aec.rs to bind signatures, or for now, we just traverse args to mark them.
                for arg in args {
                    self.check(arg)?;
                }
                Ok(Type::Any)
            }

            // ToString always produces a String
            Node::ToString(inner) => {
                self.check(inner)?;
                Ok(Type::String)
            }

            _ => {
                // Fallback catch-all for node types we haven't strictly typed yet
                // The optimizer shouldn't block Graphics or Arrays without specific rules
                Ok(Type::Any)
            }
        }
    }
}
