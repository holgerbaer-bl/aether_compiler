use crate::ast::Node;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RelType {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Array(Vec<RelType>),
    Function(Vec<String>, Box<Node>), // Parameters, Body Block
    Void,
}

impl std::fmt::Display for RelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelType::Int(v) => write!(f, "{} (i64)", v),
            RelType::Float(v) => write!(f, "{:?} (f64)", v), // Using Debug to avoid dropping .0 on integers formatting like floats
            RelType::Bool(v) => write!(f, "{} (bool)", v),
            RelType::Str(v) => write!(f, "\"{}\" (String)", v),
            RelType::Array(v) => {
                let s: Vec<String> = v.iter().map(|i| i.to_string()).collect();
                write!(f, "[{}] (Array)", s.join(", "))
            }
            RelType::Function(_, _) => write!(f, "<Function>"),
            RelType::Void => write!(f, "void"),
        }
    }
}

pub struct ExecutionEngine {
    pub memory: HashMap<String, RelType>,
}

pub enum ExecResult {
    Value(RelType),
    ReturnBlockInfo(RelType), // Explicit return triggered
    Fault(String),
}

impl ExecutionEngine {
    pub fn new() -> Self {
        Self {
            memory: HashMap::new(),
        }
    }

    pub fn execute(&mut self, root: &Node) -> String {
        self.memory.clear();
        let res = self.evaluate(root);

        let mut out = String::new();
        match res {
            ExecResult::Value(val) | ExecResult::ReturnBlockInfo(val) => {
                out.push_str(&format!("Return: {}", val));
            }
            ExecResult::Fault(err) => {
                // Return exactly "Fault: ..." as tests expect it
                return format!("Fault: {}", err);
            }
        }

        if !self.memory.is_empty() {
            let mut keys: Vec<&String> = self.memory.keys().collect();
            // Deterministic state output order is important, albeit tests don't strictly assert the var sequence format,
            // they do exact equality of string matching on simple cases.
            // Better to sort just in case. However, some tests define order implicitly:
            // "Return: 42 (i64), Memory: x = 42, y = 42" implies sequential matching or loose containing.
            // Let's defer sorting and match the specific structure if we can.
            // We'll see how tests fail.
            out.push_str(", Memory: ");

            // To ensure 100% deterministic test behavior, sort variables.
            keys.sort();
            let mem_str: Vec<String> = keys
                .iter()
                .map(|k| {
                    let v = self.memory.get(*k).unwrap();
                    match v {
                        RelType::Str(s) => format!("{} = \"{}\"", k, s),
                        RelType::Float(f) => format!("{} = {:?}", k, f),
                        RelType::Array(_) => format!("{} = [...]", k),
                        RelType::Function(_, _) => format!("{} = <fn>", k),
                        _ => format!(
                            "{} = {}",
                            k,
                            match v {
                                RelType::Int(i) => i.to_string(),
                                RelType::Bool(b) => b.to_string(),
                                _ => unreachable!(),
                            }
                        ),
                    }
                })
                .collect();

            out.push_str(&mem_str.join(", "));
        }

        out
    }

    fn evaluate(&mut self, node: &Node) -> ExecResult {
        match node {
            // Literals
            Node::IntLiteral(v) => ExecResult::Value(RelType::Int(*v)),
            Node::FloatLiteral(v) => ExecResult::Value(RelType::Float(*v)),
            Node::BoolLiteral(v) => ExecResult::Value(RelType::Bool(*v)),
            Node::StringLiteral(v) => ExecResult::Value(RelType::Str(v.clone())),

            // Mem
            Node::Identifier(name) => {
                if let Some(val) = self.memory.get(name) {
                    ExecResult::Value(val.clone())
                } else {
                    ExecResult::Fault("Undefined identifier".to_string())
                }
            }
            Node::Assign(name, expr_node) => match self.evaluate(expr_node) {
                ExecResult::Value(val) => {
                    self.memory.insert(name.clone(), val.clone());
                    ExecResult::Value(val)
                }
                ExecResult::ReturnBlockInfo(val) => {
                    self.memory.insert(name.clone(), val.clone());
                    ExecResult::Value(val)
                }
                fault => fault,
            },

            // Math
            Node::Add(l, r) => self.do_math(l, r, '+'),
            Node::Sub(l, r) => self.do_math(l, r, '-'),
            Node::Mul(l, r) => self.do_math(l, r, '*'),
            Node::Div(l, r) => self.do_math(l, r, '/'),

            // Logic
            Node::Eq(l, r) => {
                let lv = self.evaluate(l);
                let rv = self.evaluate(r);
                match (lv, rv) {
                    (ExecResult::Value(l_val), ExecResult::Value(r_val)) => {
                        ExecResult::Value(RelType::Bool(l_val == r_val))
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid Eq semantics".to_string()),
                }
            }
            Node::Lt(l, r) => {
                let lv = self.evaluate(l);
                let rv = self.evaluate(r);
                match (lv, rv) {
                    (ExecResult::Value(RelType::Int(li)), ExecResult::Value(RelType::Int(ri))) => {
                        ExecResult::Value(RelType::Bool(li < ri))
                    }
                    (
                        ExecResult::Value(RelType::Float(lf)),
                        ExecResult::Value(RelType::Float(rf)),
                    ) => ExecResult::Value(RelType::Bool(lf < rf)),
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid Lt semantics".to_string()),
                }
            }

            // Arrays & Strings
            Node::ArrayLiteral(items) => {
                let mut vals = Vec::new();
                for item in items {
                    match self.evaluate(item) {
                        ExecResult::Value(v) => vals.push(v),
                        fault => return fault,
                    }
                }
                ExecResult::Value(RelType::Array(vals))
            }
            Node::Index(container, index) => {
                let cv = self.evaluate(container);
                let iv = self.evaluate(index);
                match (cv, iv) {
                    (
                        ExecResult::Value(RelType::Array(arr)),
                        ExecResult::Value(RelType::Int(idx)),
                    ) => {
                        if idx >= 0 && (idx as usize) < arr.len() {
                            ExecResult::Value(arr[idx as usize].clone())
                        } else {
                            ExecResult::Fault("Index out of bounds".to_string())
                        }
                    }
                    (ExecResult::Value(RelType::Str(s)), ExecResult::Value(RelType::Int(idx))) => {
                        if idx >= 0 && (idx as usize) < s.len() {
                            let ch = s.chars().nth(idx as usize).unwrap();
                            ExecResult::Value(RelType::Str(ch.to_string()))
                        } else {
                            ExecResult::Fault("Index out of bounds".to_string())
                        }
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid Index semantics".to_string()),
                }
            }
            Node::Concat(l, r) => {
                let lv = self.evaluate(l);
                let rv = self.evaluate(r);
                match (lv, rv) {
                    (ExecResult::Value(RelType::Str(ls)), ExecResult::Value(RelType::Str(rs))) => {
                        ExecResult::Value(RelType::Str(ls + &rs))
                    }
                    (
                        ExecResult::Value(RelType::Array(mut la)),
                        ExecResult::Value(RelType::Array(ra)),
                    ) => {
                        la.extend(ra);
                        ExecResult::Value(RelType::Array(la))
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid Concat semantics".to_string()),
                }
            }

            // Bitwise
            Node::BitAnd(l, r) => {
                let lv = self.evaluate(l);
                let rv = self.evaluate(r);
                match (lv, rv) {
                    (ExecResult::Value(RelType::Int(li)), ExecResult::Value(RelType::Int(ri))) => {
                        ExecResult::Value(RelType::Int(li & ri))
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid BitAnd semantics".to_string()),
                }
            }
            Node::BitShiftLeft(l, r) => {
                let lv = self.evaluate(l);
                let rv = self.evaluate(r);
                match (lv, rv) {
                    (ExecResult::Value(RelType::Int(li)), ExecResult::Value(RelType::Int(ri))) => {
                        ExecResult::Value(RelType::Int(li << ri))
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid BitShiftLeft semantics".to_string()),
                }
            }
            Node::BitShiftRight(l, r) => {
                let lv = self.evaluate(l);
                let rv = self.evaluate(r);
                match (lv, rv) {
                    (ExecResult::Value(RelType::Int(li)), ExecResult::Value(RelType::Int(ri))) => {
                        ExecResult::Value(RelType::Int(li >> ri))
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("Invalid BitShiftRight semantics".to_string()),
                }
            }

            // Functions
            Node::FnDef(name, params, body) => {
                let func = RelType::Function(params.clone(), body.clone());
                self.memory.insert(name.clone(), func.clone());
                ExecResult::Value(func)
            }
            Node::Call(name, args) => {
                let func_val = match self.memory.get(name) {
                    Some(val) => val.clone(),
                    None => return ExecResult::Fault(format!("Undefined function '{}'", name)),
                };

                match func_val {
                    RelType::Function(params, body) => {
                        if args.len() != params.len() {
                            return ExecResult::Fault("Argument count mismatch".to_string());
                        }

                        let mut evaluated_args = Vec::new();
                        for arg in args {
                            match self.evaluate(arg) {
                                ExecResult::Value(v) => evaluated_args.push(v),
                                ExecResult::ReturnBlockInfo(v) => evaluated_args.push(v),
                                fault => return fault,
                            }
                        }

                        let old_memory = self.memory.clone();
                        for (i, p) in params.iter().enumerate() {
                            self.memory.insert(p.clone(), evaluated_args[i].clone());
                        }

                        let mut call_res = self.evaluate(&body);
                        if let ExecResult::ReturnBlockInfo(v) = call_res {
                            call_res = ExecResult::Value(v);
                        }

                        self.memory = old_memory; // Pop scope
                        call_res
                    }
                    _ => ExecResult::Fault(format!("Identifier '{}' is not a function", name)),
                }
            }

            // I/O
            Node::FileRead(path_node) => match self.evaluate(path_node) {
                ExecResult::Value(RelType::Str(path)) => match std::fs::read(&path) {
                    Ok(bytes) => {
                        let arr = bytes.into_iter().map(|b| RelType::Int(b as i64)).collect();
                        ExecResult::Value(RelType::Array(arr))
                    }
                    Err(e) => ExecResult::Fault(format!("FileRead error: {}", e)),
                },
                ExecResult::Fault(err) => ExecResult::Fault(err),
                _ => ExecResult::Fault("FileRead semantic error: path not a string".to_string()),
            },
            Node::FileWrite(path_node, data_node) => {
                let p_val = self.evaluate(path_node);
                let d_val = self.evaluate(data_node);
                match (p_val, d_val) {
                    (
                        ExecResult::Value(RelType::Str(path)),
                        ExecResult::Value(RelType::Array(arr)),
                    ) => {
                        let mut bytes = Vec::new();
                        for item in arr {
                            if let RelType::Int(b) = item {
                                bytes.push(b as u8);
                            } else {
                                return ExecResult::Fault(
                                    "FileWrite error: data array contains non-integer".to_string(),
                                );
                            }
                        }
                        if let Err(e) = std::fs::write(&path, bytes) {
                            return ExecResult::Fault(format!("FileWrite error: {}", e));
                        }
                        ExecResult::Value(RelType::Void)
                    }
                    (ExecResult::Value(RelType::Str(path)), ExecResult::Value(RelType::Str(s))) => {
                        if let Err(e) = std::fs::write(&path, s.as_bytes()) {
                            return ExecResult::Fault(format!("FileWrite error: {}", e));
                        }
                        ExecResult::Value(RelType::Void)
                    }
                    (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => {
                        ExecResult::Fault(err)
                    }
                    _ => ExecResult::Fault("FileWrite semantic error".to_string()),
                }
            }

            // FFI / Reflection
            Node::EvalBincodeNative(bytes_node) => match self.evaluate(bytes_node) {
                ExecResult::Value(RelType::Array(arr)) => {
                    let bytes: Vec<u8> = arr
                        .into_iter()
                        .map(|v| match v {
                            RelType::Int(i) => i as u8,
                            _ => 0,
                        })
                        .collect();

                    match bincode::deserialize::<Node>(&bytes) {
                        Ok(parsed) => {
                            let mut sub_engine = ExecutionEngine::new();
                            let output = sub_engine.execute(&parsed);
                            ExecResult::Value(RelType::Str(output))
                        }
                        Err(e) => ExecResult::Fault(format!("Bincode Native Eval Fault: {}", e)),
                    }
                }
                fault => fault,
            },
            Node::ToString(n) => {
                match self.evaluate(n) {
                    ExecResult::Value(v) => {
                        let s = format!("{}", v);
                        // Clean up type signatures "42 (i64)" -> "42" so it can be combined easily
                        // Wait, no. We just use standard format. If it matches test output needs, it shouldn't have signatures.
                        // Actually, our RelType::Display has signatures. The evaluator output string matches Display.
                        // For building arbitrary strings to file we might need raw conversions, but we just want Display format for tests.
                        ExecResult::Value(RelType::Str(s))
                    }
                    fault => fault,
                }
            }

            // Flow
            Node::If(cond, then_br, else_br) => {
                let cv = self.evaluate(cond);
                match cv {
                    ExecResult::Value(RelType::Bool(true)) => self.evaluate(then_br),
                    ExecResult::Value(RelType::Bool(false)) => {
                        if let Some(eb) = else_br {
                            self.evaluate(eb)
                        } else {
                            ExecResult::Value(RelType::Void)
                        }
                    }
                    ExecResult::Fault(err) => ExecResult::Fault(err),
                    _ => ExecResult::Fault("If condition not a boolean".to_string()),
                }
            }
            Node::While(cond, body) => {
                loop {
                    match self.evaluate(cond) {
                        ExecResult::Value(RelType::Bool(true)) => match self.evaluate(body) {
                            ExecResult::ReturnBlockInfo(r) => {
                                return ExecResult::ReturnBlockInfo(r);
                            }
                            ExecResult::Fault(err) => return ExecResult::Fault(err),
                            _ => {}
                        },
                        ExecResult::Value(RelType::Bool(false)) => break,
                        ExecResult::Fault(err) => return ExecResult::Fault(err),
                        _ => return ExecResult::Fault("While condition not a boolean".to_string()),
                    }
                }
                ExecResult::Value(RelType::Void) // while evaluate returns void naturally unless return hits
            }
            Node::Block(nodes) => {
                let mut last_val = RelType::Void;
                for n in nodes {
                    match self.evaluate(n) {
                        ExecResult::ReturnBlockInfo(val) => {
                            return ExecResult::ReturnBlockInfo(val);
                        }
                        ExecResult::Fault(err) => return ExecResult::Fault(err),
                        ExecResult::Value(val) => {
                            last_val = val;
                        }
                    }
                }
                ExecResult::Value(last_val)
            }
            Node::Return(val_node) => match self.evaluate(val_node) {
                ExecResult::Value(v) => ExecResult::ReturnBlockInfo(v),
                fault => fault,
            },
        }
    }

    fn do_math(&mut self, l: &Node, r: &Node, op: char) -> ExecResult {
        let lv = self.evaluate(l);
        let rv = self.evaluate(r);

        match (lv, rv) {
            (ExecResult::Value(RelType::Int(li)), ExecResult::Value(RelType::Int(ri))) => {
                match op {
                    '+' => ExecResult::Value(RelType::Int(li + ri)),
                    '-' => ExecResult::Value(RelType::Int(li - ri)),
                    '*' => ExecResult::Value(RelType::Int(li * ri)),
                    '/' => {
                        if ri == 0 {
                            ExecResult::Fault("Division by zero".to_string())
                        } else {
                            ExecResult::Value(RelType::Int(li / ri))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (ExecResult::Value(RelType::Float(lf)), ExecResult::Value(RelType::Float(rf))) => {
                match op {
                    '+' => ExecResult::Value(RelType::Float(lf + rf)),
                    '-' => ExecResult::Value(RelType::Float(lf - rf)),
                    '*' => ExecResult::Value(RelType::Float(lf * rf)),
                    '/' => {
                        if rf == 0.0 {
                            ExecResult::Fault("Division by zero".to_string())
                        } else {
                            ExecResult::Value(RelType::Float(lf / rf))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (ExecResult::Fault(err), _) | (_, ExecResult::Fault(err)) => ExecResult::Fault(err),
            _ => ExecResult::Fault("Mathematical type mismatch".to_string()),
        }
    }
}
