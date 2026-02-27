use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    // Control Flow
    If(Box<Node>, Box<Node>, Option<Box<Node>>),
    While(Box<Node>, Box<Node>),
    Block(Vec<Node>),
    Return(Box<Node>),
}
