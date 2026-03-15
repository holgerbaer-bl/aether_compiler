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
    pub fn compile_node(&mut self, node: &Node) -> bool {
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
                if !self.compile_node(l) || !self.compile_node(r) { return false; }
                self.instructions.push(OpCode::Add);
                true
            }
            Node::Sub(l, r) => {
                if !self.compile_node(l) || !self.compile_node(r) { return false; }
                self.instructions.push(OpCode::Subtract);
                true
            }
            Node::Mul(l, r) => {
                if !self.compile_node(l) || !self.compile_node(r) { return false; }
                self.instructions.push(OpCode::Multiply);
                true
            }
            Node::Div(l, r) => {
                if !self.compile_node(l) || !self.compile_node(r) { return false; }
                self.instructions.push(OpCode::Divide);
                true
            }
            Node::Print(expr) => {
                if !self.compile_node(expr) { return false; }
                self.instructions.push(OpCode::Print);
                true
            }
            Node::Return(expr) => {
                if !self.compile_node(expr) { return false; }
                self.instructions.push(OpCode::Return);
                true
            }
            _ => false,
        }
    }

    fn add_constant(&mut self, val: RelType) -> usize {
        if let Some(idx) = self.constants.iter().position(|c| c == &val) {
            return idx;
        }
        self.constants.push(val);
        self.constants.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Node;

    #[test]
    fn test_compile_add() {
        let mut compiler = Compiler::new();
        let ast = Node::Add(
            Box::new(Node::IntLiteral(10)),
            Box::new(Node::IntLiteral(5)),
        );
        assert!(compiler.compile_node(&ast));
        
        assert_eq!(
            compiler.instructions,
            vec![OpCode::Constant(0), OpCode::Constant(1), OpCode::Add]
        );
        assert_eq!(compiler.constants, vec![RelType::Int(10), RelType::Int(5)]);
    }

    #[test]
    fn test_deduplicate_constants() {
        let mut compiler = Compiler::new();
        let ast = Node::Add(
            Box::new(Node::IntLiteral(10)),
            Box::new(Node::IntLiteral(10)),
        );
        assert!(compiler.compile_node(&ast));
        
        assert_eq!(
            compiler.instructions,
            vec![OpCode::Constant(0), OpCode::Constant(0), OpCode::Add]
        );
        assert_eq!(compiler.constants, vec![RelType::Int(10)]);
    }
}
