use crate::executor::RelType;
use crate::vm::opcode::OpCode;

#[derive(Default)]
pub struct VM {
    stack: Vec<RelType>,
    pub ip: usize,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            ip: 0,
        }
    }

    #[inline(always)]
    pub fn run(&mut self, instructions: &[OpCode], constants: &[RelType]) -> Result<RelType, String> {
        self.stack.clear();
        self.ip = 0;

        while self.ip < instructions.len() {
            let op = &instructions[self.ip];
            self.ip += 1;

            match op {
                OpCode::Constant(idx) => {
                    if *idx < constants.len() {
                        self.stack.push(constants[*idx].clone());
                    } else {
                        return Err("Constant index out of bounds".into());
                    }
                }
                OpCode::Add => {
                    let r = self.stack.pop().unwrap_or(RelType::Void);
                    let l = self.stack.pop().unwrap_or(RelType::Void);
                    match (l, r) {
                        (RelType::Int(a), RelType::Int(b)) => self.stack.push(RelType::Int(a + b)),
                        (RelType::Float(a), RelType::Float(b)) => self.stack.push(RelType::Float(a + b)),
                        (RelType::Int(a), RelType::Float(b)) => self.stack.push(RelType::Float(a as f64 + b)),
                        (RelType::Float(a), RelType::Int(b)) => self.stack.push(RelType::Float(a + b as f64)),
                        (RelType::Str(a), RelType::Str(b)) => self.stack.push(RelType::Str(a + &b)),
                        _ => return Err("Invalid types for Add".into()),
                    }
                }
                OpCode::Subtract => {
                    let r = self.stack.pop().unwrap_or(RelType::Void);
                    let l = self.stack.pop().unwrap_or(RelType::Void);
                    match (l, r) {
                        (RelType::Int(a), RelType::Int(b)) => self.stack.push(RelType::Int(a - b)),
                        (RelType::Float(a), RelType::Float(b)) => self.stack.push(RelType::Float(a - b)),
                        (RelType::Int(a), RelType::Float(b)) => self.stack.push(RelType::Float(a as f64 - b)),
                        (RelType::Float(a), RelType::Int(b)) => self.stack.push(RelType::Float(a - b as f64)),
                        _ => return Err("Invalid types for Subtract".into()),
                    }
                }
                OpCode::Multiply => {
                    let r = self.stack.pop().unwrap_or(RelType::Void);
                    let l = self.stack.pop().unwrap_or(RelType::Void);
                    match (l, r) {
                        (RelType::Int(a), RelType::Int(b)) => self.stack.push(RelType::Int(a * b)),
                        (RelType::Float(a), RelType::Float(b)) => self.stack.push(RelType::Float(a * b)),
                        (RelType::Int(a), RelType::Float(b)) => self.stack.push(RelType::Float(a as f64 * b)),
                        (RelType::Float(a), RelType::Int(b)) => self.stack.push(RelType::Float(a * b as f64)),
                        _ => return Err("Invalid types for Multiply".into()),
                    }
                }
                OpCode::Divide => {
                    let r = self.stack.pop().unwrap_or(RelType::Void);
                    let l = self.stack.pop().unwrap_or(RelType::Void);
                    match (l, r) {
                        (RelType::Int(a), RelType::Int(b)) => {
                            if b == 0 { return Err("Div by zero".into()); }
                            self.stack.push(RelType::Int(a / b))
                        },
                        (RelType::Float(a), RelType::Float(b)) => {
                            if b == 0.0 { return Err("Div by zero".into()); }
                            self.stack.push(RelType::Float(a / b))
                        },
                        _ => return Err("Invalid types for Divide".into()),
                    }
                }
                OpCode::Print => {
                    let val = self.stack.pop().unwrap_or(RelType::Void);
                    println!("{}", val);
                }
                OpCode::Return => {
                    return Ok(self.stack.pop().unwrap_or(RelType::Void));
                }
            }
        }

        Ok(self.stack.pop().unwrap_or(RelType::Void))
    }
}
