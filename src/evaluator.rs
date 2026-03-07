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
            Node::Identifier(name) => {
                if let Some(v) = self.get_var(name) { ExecResult::Value(v) }
                else { ExecResult::Fault(format!("Variable '{}' not found", name)) }
            }
            Node::IntLiteral(v) => ExecResult::Value(RelType::Int(*v)),
            Node::FloatLiteral(v) => ExecResult::Value(RelType::Float(*v)),
            Node::BoolLiteral(v) => ExecResult::Value(RelType::Bool(*v)),
            Node::Add(l, r) => self.do_math(l, '+', r),
            Node::Assign(name, expr) => {
                match self.evaluate_inner(expr) {
                    ExecResult::Value(v) => { self.set_var(name.clone(), v.clone()); ExecResult::Value(v) }
                    ExecResult::ReturnBlockInfo(v) => { self.set_var(name.clone(), v.clone()); ExecResult::Value(v) }
                    err => err,
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
            
            Node::StringLiteral(v) => ExecResult::Value(RelType::Str(v.clone())),
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
            Node::Time | Node::GlobalTime => ExecResult::Value(RelType::Float(self.startup_time.elapsed().as_secs_f64())),
            
            Node::Sub(l, r) => self.do_math(l, '-', r),
            Node::Mul(l, r) => self.do_math(l, '*', r),
            Node::Div(l, r) => self.do_math(l, '/', r),

            Node::Eq(l, r) => self.do_compare(l, "==", r),
            Node::Lt(l, r) => self.do_compare(l, "<", r),
            Node::Gt(l, r) => self.do_compare(l, ">", r),

            Node::Print(expr) => {
                match self.evaluate_inner(expr) {
                    ExecResult::Value(v) => { println!("{}", v); ExecResult::Value(RelType::Void) }
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
                        
                        // Clean up scope
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

            Node::Return(expr) => {
                let v = match self.evaluate_inner(&*expr) { ExecResult::Value(v) => v, err => return err };
                ExecResult::ReturnBlockInfo(v)
            }

            Node::CheckCollision { a_min, a_max, b_min, b_max } => {
                let am = match self.evaluate_inner(a_min) { ExecResult::Value(v) => v, err => return err };
                let ax = match self.evaluate_inner(a_max) { ExecResult::Value(v) => v, err => return err };
                let bm = match self.evaluate_inner(b_min) { ExecResult::Value(v) => v, err => return err };
                let bx = match self.evaluate_inner(b_max) { ExecResult::Value(v) => v, err => return err };
                
                let v_am = if let Some(v) = self.to_vec3(am) { v } else { return ExecResult::Fault("a_min must be [x, y, z]".into()) };
                let v_ax = if let Some(v) = self.to_vec3(ax) { v } else { return ExecResult::Fault("a_max must be [x, y, z]".into()) };
                let v_bm = if let Some(v) = self.to_vec3(bm) { v } else { return ExecResult::Fault("b_min must be [x, y, z]".into()) };
                let v_bx = if let Some(v) = self.to_vec3(bx) { v } else { return ExecResult::Fault("b_max must be [x, y, z]".into()) };
                
                let aabb_a = crate::math::AABB::new(v_am, v_ax);
                let aabb_b = crate::math::AABB::new(v_bm, v_bx);
                
                ExecResult::Value(RelType::Int(if aabb_a.intersects(&aabb_b) { 1 } else { 0 }))
            }

            _ => self.evaluate_extra(node),
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

    fn to_vec3(&self, val: RelType) -> Option<[f32; 3]> {
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
