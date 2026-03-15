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
                OpCode::Equal => {
                    let r = self.stack.pop().unwrap_or(RelType::Void);
                    let l = self.stack.pop().unwrap_or(RelType::Void);
                    self.stack.push(RelType::Bool(l == r));
                }
                OpCode::Less => {
                    let r = self.stack.pop().unwrap_or(RelType::Void);
                    let l = self.stack.pop().unwrap_or(RelType::Void);
                    match (l, r) {
                        (RelType::Int(a), RelType::Int(b)) => self.stack.push(RelType::Bool(a < b)),
                        (RelType::Float(a), RelType::Float(b)) => self.stack.push(RelType::Bool(a < b)),
                        _ => self.stack.push(RelType::Bool(false)),
                    }
                }
                OpCode::Greater => {
                    let r = self.stack.pop().unwrap_or(RelType::Void);
                    let l = self.stack.pop().unwrap_or(RelType::Void);
                    match (l, r) {
                        (RelType::Int(a), RelType::Int(b)) => self.stack.push(RelType::Bool(a > b)),
                        (RelType::Float(a), RelType::Float(b)) => self.stack.push(RelType::Bool(a > b)),
                        _ => self.stack.push(RelType::Bool(false)),
                    }
                }
                OpCode::JumpIfFalse(target_ip) => {
                    let cond = self.stack.pop().unwrap_or(RelType::Void);
                    let is_true = match cond {
                        RelType::Bool(b) => b,
                        RelType::Int(i) => i != 0,
                        _ => false,
                    };
                    if !is_true {
                        self.ip = *target_ip;
                    }
                }
                OpCode::Jump(target_ip) => {
                    self.ip = *target_ip;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::RelType;
    use crate::vm::opcode::OpCode;

    #[test]
    fn test_vm_execution_add() {
        let mut vm = VM::new();
        // Represents: 10 + 5
        let instructions = vec![
            OpCode::Constant(0), // Push 10
            OpCode::Constant(1), // Push 5
            OpCode::Add,         // Pop 5, Pop 10, Push 15
            OpCode::Return,
        ];
        let constants = vec![RelType::Int(10), RelType::Int(5)];

        let result = vm.run(&instructions, &constants).unwrap();
        assert_eq!(result, RelType::Int(15));
    }

    #[test]
    fn test_vm_execution_complex() {
        let mut vm = VM::new();
        // Represents: (10 - 2) * 3
        let instructions = vec![
            OpCode::Constant(0), // Push 10
            OpCode::Constant(1), // Push 2
            OpCode::Subtract,    // Pop 2, Pop 10, Push 8
            OpCode::Constant(2), // Push 3
            OpCode::Multiply,    // Pop 3, Pop 8, Push 24
            OpCode::Return,
        ];
        let constants = vec![RelType::Int(10), RelType::Int(2), RelType::Int(3)];

        let result = vm.run(&instructions, &constants).unwrap();
        assert_eq!(result, RelType::Int(24));
    }

    #[test]
    fn test_vm_jump_if_false() {
        let mut vm = VM::new();
        // Represents: if (false) { 10 } else { 20 }
        let instructions = vec![
            OpCode::Constant(0),       // Push false
            OpCode::JumpIfFalse(4),    // If false, jump to index 4
            OpCode::Constant(1),       // Push 10
            OpCode::Jump(5),           // Jump to end (index 5)
            OpCode::Constant(2),       // Push 20 (index 4)
            OpCode::Return,            // Return (index 5)
        ];
        let constants = vec![RelType::Bool(false), RelType::Int(10), RelType::Int(20)];

        let result = vm.run(&instructions, &constants).unwrap();
        assert_eq!(result, RelType::Int(20));
    }
}
