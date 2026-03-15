#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    Constant(usize),
    Add,
    Subtract,
    Multiply,
    Divide,
    Print,
    Return,
}
