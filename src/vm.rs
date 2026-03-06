pub mod storage;

use crate::ast::Node;
use crate::executor::RelType;

#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    PushInt(i64),
    PushFloat(f64),
    PushBool(bool),
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
}

pub struct VMCompiler {
    pub code: Vec<Opcode>,
}

impl VMCompiler {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    /// Recursively flattens an AST math/logic tree into linear opcodes.
    /// Returns false if the node cannot be compiled (e.g. it contains side-effects or variables).
    pub fn compile(&mut self, node: &Node) -> bool {
        match node {
            Node::IntLiteral(v) => {
                self.code.push(Opcode::PushInt(*v));
                true
            }
            Node::FloatLiteral(v) => {
                self.code.push(Opcode::PushFloat(*v));
                true
            }
            Node::BoolLiteral(v) => {
                self.code.push(Opcode::PushBool(*v));
                true
            }
            Node::Add(l, r) => {
                if !self.compile(l) || !self.compile(r) {
                    return false;
                }
                self.code.push(Opcode::Add);
                true
            }
            Node::Sub(l, r) => {
                if !self.compile(l) || !self.compile(r) {
                    return false;
                }
                self.code.push(Opcode::Sub);
                true
            }
            Node::Mul(l, r) => {
                if !self.compile(l) || !self.compile(r) {
                    return false;
                }
                self.code.push(Opcode::Mul);
                true
            }
            Node::Div(l, r) => {
                if !self.compile(l) || !self.compile(r) {
                    return false;
                }
                self.code.push(Opcode::Div);
                true
            }
            Node::Eq(l, r) => {
                if !self.compile(l) || !self.compile(r) {
                    return false;
                }
                self.code.push(Opcode::Eq);
                true
            }
            Node::Lt(l, r) => {
                if !self.compile(l) || !self.compile(r) {
                    return false;
                }
                self.code.push(Opcode::Lt);
                true
            }
            Node::Gt(l, r) => {
                if !self.compile(l) || !self.compile(r) {
                    return false;
                }
                self.code.push(Opcode::Gt);
                true
            }
            // Variables, function calls, arrays, UI nodes cannot be compiled to this basic math VM yet.
            _ => false,
        }
    }
}

pub struct VM {
    stack: Vec<RelType>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256), // Pre-allocate for speed
        }
    }

    #[inline(always)]
    pub fn execute(&mut self, code: &[Opcode]) -> RelType {
        self.stack.clear();

        for op in code {
            match op {
                Opcode::PushInt(v) => self.stack.push(RelType::Int(*v)),
                Opcode::PushFloat(v) => self.stack.push(RelType::Float(*v)),
                Opcode::PushBool(v) => self.stack.push(RelType::Bool(*v)),
                Opcode::Add => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    match (l, r) {
                        (RelType::Int(a), RelType::Int(b)) => self.stack.push(RelType::Int(a + b)),
                        (RelType::Float(a), RelType::Float(b)) => {
                            self.stack.push(RelType::Float(a + b))
                        }
                        (RelType::Int(a), RelType::Float(b)) => {
                            self.stack.push(RelType::Float(a as f64 + b))
                        }
                        (RelType::Float(a), RelType::Int(b)) => {
                            self.stack.push(RelType::Float(a + b as f64))
                        }
                        _ => panic!("VM TypeError: Add requires num"),
                    }
                }
                Opcode::Sub => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    match (l, r) {
                        (RelType::Int(a), RelType::Int(b)) => self.stack.push(RelType::Int(a - b)),
                        (RelType::Float(a), RelType::Float(b)) => {
                            self.stack.push(RelType::Float(a - b))
                        }
                        (RelType::Int(a), RelType::Float(b)) => {
                            self.stack.push(RelType::Float(a as f64 - b))
                        }
                        (RelType::Float(a), RelType::Int(b)) => {
                            self.stack.push(RelType::Float(a - b as f64))
                        }
                        _ => panic!("VM TypeError: Sub requires num"),
                    }
                }
                Opcode::Mul => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    match (l, r) {
                        (RelType::Int(a), RelType::Int(b)) => self.stack.push(RelType::Int(a * b)),
                        (RelType::Float(a), RelType::Float(b)) => {
                            self.stack.push(RelType::Float(a * b))
                        }
                        (RelType::Int(a), RelType::Float(b)) => {
                            self.stack.push(RelType::Float(a as f64 * b))
                        }
                        (RelType::Float(a), RelType::Int(b)) => {
                            self.stack.push(RelType::Float(a * b as f64))
                        }
                        _ => panic!("VM TypeError: Mul requires num"),
                    }
                }
                Opcode::Div => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    match (l, r) {
                        (RelType::Int(a), RelType::Int(b)) => self
                            .stack
                            .push(RelType::Int(if b != 0 { a / b } else { 0 })),
                        (RelType::Float(a), RelType::Float(b)) => self
                            .stack
                            .push(RelType::Float(if b != 0.0 { a / b } else { 0.0 })),
                        _ => panic!("VM TypeError: Div requires num"),
                    }
                }
                Opcode::Eq => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    self.stack.push(RelType::Bool(l == r));
                }
                Opcode::Lt => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    match (l, r) {
                        (RelType::Int(a), RelType::Int(b)) => self.stack.push(RelType::Bool(a < b)),
                        (RelType::Float(a), RelType::Float(b)) => {
                            self.stack.push(RelType::Bool(a < b))
                        }
                        _ => self.stack.push(RelType::Bool(false)),
                    }
                }
                Opcode::Gt => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    match (l, r) {
                        (RelType::Int(a), RelType::Int(b)) => self.stack.push(RelType::Bool(a > b)),
                        (RelType::Float(a), RelType::Float(b)) => {
                            self.stack.push(RelType::Bool(a > b))
                        }
                        _ => self.stack.push(RelType::Bool(false)),
                    }
                }
            }
        }

        self.stack.pop().unwrap_or(RelType::Void)
    }
}
