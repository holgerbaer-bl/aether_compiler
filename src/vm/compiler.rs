use crate::ast::Node;
use crate::executor::RelType;
use crate::vm::opcode::OpCode;

#[derive(Default)]
pub struct Compiler {
    pub instructions: Vec<OpCode>,
    pub constants: Vec<RelType>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Recursively flattens an AST math/logic tree into linear opcodes.
    /// Returns false if the node cannot be compiled (e.g. it contains side-effects or variables).
    pub fn compile(&mut self, node: &Node) -> bool {
        match node {
            Node::IntLiteral(v) => {
                let idx = self.add_constant(RelType::Int(*v));
                self.instructions.push(OpCode::Constant(idx));
                true
            }
            Node::FloatLiteral(v) => {
                let idx = self.add_constant(RelType::Float(*v));
                self.instructions.push(OpCode::Constant(idx));
                true
            }
            Node::StringLiteral(v) => {
                let idx = self.add_constant(RelType::Str(v.clone()));
                self.instructions.push(OpCode::Constant(idx));
                true
            }
            Node::BoolLiteral(v) => {
                let idx = self.add_constant(RelType::Bool(*v));
                self.instructions.push(OpCode::Constant(idx));
                true
            }
            Node::Add(l, r) => {
                if !self.compile(l) || !self.compile(r) { return false; }
                self.instructions.push(OpCode::Add);
                true
            }
            Node::Sub(l, r) => {
                if !self.compile(l) || !self.compile(r) { return false; }
                self.instructions.push(OpCode::Subtract);
                true
            }
            Node::Mul(l, r) => {
                if !self.compile(l) || !self.compile(r) { return false; }
                self.instructions.push(OpCode::Multiply);
                true
            }
            Node::Div(l, r) => {
                if !self.compile(l) || !self.compile(r) { return false; }
                self.instructions.push(OpCode::Divide);
                true
            }
            Node::Print(expr) => {
                if !self.compile(expr) { return false; }
                self.instructions.push(OpCode::Print);
                true
            }
            Node::Return(expr) => {
                if !self.compile(expr) { return false; }
                self.instructions.push(OpCode::Return);
                true
            }
            _ => false,
        }
    }

    fn add_constant(&mut self, val: RelType) -> usize {
        self.constants.push(val);
        self.constants.len() - 1
    }
}
