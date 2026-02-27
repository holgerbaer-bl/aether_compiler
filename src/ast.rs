use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),

    // Memory
    Identifier(String),
    Assign(String, Box<Node>),

    // Math & Logic
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Eq(Box<Node>, Box<Node>),
    Lt(Box<Node>, Box<Node>),

    // Arrays & Strings
    ArrayLiteral(Vec<Node>),
    Index(Box<Node>, Box<Node>),
    Concat(Box<Node>, Box<Node>),

    // Bitwise
    BitAnd(Box<Node>, Box<Node>),
    BitShiftLeft(Box<Node>, Box<Node>),
    BitShiftRight(Box<Node>, Box<Node>),

    // Functions
    FnDef(String, Vec<String>, Box<Node>),
    Call(String, Vec<Node>),

    // I/O
    FileRead(Box<Node>),
    FileWrite(Box<Node>, Box<Node>),

    // FFI / Reflection
    EvalBincodeNative(Box<Node>),
    ToString(Box<Node>),

    // Control Flow
    If(Box<Node>, Box<Node>, Option<Box<Node>>),
    While(Box<Node>, Box<Node>),
    Block(Vec<Node>),
    Return(Box<Node>),
}
