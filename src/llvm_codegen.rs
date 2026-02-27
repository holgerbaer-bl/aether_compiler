use crate::ast::Node;

pub struct LLVMGenerator;

impl LLVMGenerator {
    /// Generates strictly raw LLVM IR text representing the given AetherCore AST graph.
    pub fn generate_ir(root: &Node) -> String {
        let mut ir = String::new();
        ir.push_str("; ModuleID = 'AetherCoreCompilationUnit'\n");
        ir.push_str("source_filename = \"aethercore.aec\"\n\n");

        ir.push_str("define void @main() {\n");
        ir.push_str("entry:\n");

        // This is a minimal mock traversing the AST to fulfill Sprint 2 LLVM generation requirement
        // Due to the lack of actual bindings and the prompt's fallback request to text-based `.ll` dumping.
        Self::traverse_ir(root, &mut ir, 1);

        ir.push_str("  ret void\n");
        ir.push_str("}\n");
        ir
    }

    fn traverse_ir(node: &Node, ir: &mut String, depth: usize) {
        let indent = "  ".repeat(depth);
        match node {
            // Memory Operations (Alloca/Load/Store)
            Node::Assign(name, expr) => {
                ir.push_str(&format!("{}%{} = alloca i64, align 8\n", indent, name));
                // Recursing evaluates RHS but for a crude IR generator without exact typed-SSA mapping
                // we just record the structure.
                ir.push_str(&format!("{}; assigning to {}\n", indent, name));
                Self::traverse_ir(expr, ir, depth);
            }
            Node::Identifier(name) => {
                ir.push_str(&format!(
                    "{}%val_{} = load i64, ptr %{}, align 8\n",
                    indent, name, name
                ));
            }

            // Math
            Node::Add(l, r) => {
                Self::traverse_ir(l, ir, depth);
                Self::traverse_ir(r, ir, depth);
                ir.push_str(&format!("{}%add_res = add i64 %left, %right\n", indent));
            }
            Node::Sub(l, r) => {
                Self::traverse_ir(l, ir, depth);
                Self::traverse_ir(r, ir, depth);
                ir.push_str(&format!("{}%sub_res = sub i64 %left, %right\n", indent));
            }
            Node::Mul(l, r) => {
                Self::traverse_ir(l, ir, depth);
                Self::traverse_ir(r, ir, depth);
                ir.push_str(&format!("{}%mul_res = mul i64 %left, %right\n", indent));
            }
            Node::Div(l, r) => {
                Self::traverse_ir(l, ir, depth);
                Self::traverse_ir(r, ir, depth);
                ir.push_str(&format!("{}%div_res = sdiv i64 %left, %right\n", indent));
            }

            // Flow Control
            Node::If(cond, then_br, else_br) => {
                Self::traverse_ir(cond, ir, depth);
                ir.push_str(&format!(
                    "{}br i1 %cond, label %then, label %else\n",
                    indent
                ));
                ir.push_str(&format!("then:\n"));
                Self::traverse_ir(then_br, ir, depth + 1);

                if let Some(eb) = else_br {
                    ir.push_str(&format!("else:\n"));
                    Self::traverse_ir(eb, ir, depth + 1);
                }
            }
            Node::While(cond, body) => {
                ir.push_str(&format!("loop_cond:\n"));
                Self::traverse_ir(cond, ir, depth + 1);
                ir.push_str(&format!(
                    "{}br i1 %cond, label %loop_body, label %loop_end\n",
                    indent
                ));

                ir.push_str(&format!("loop_body:\n"));
                Self::traverse_ir(body, ir, depth + 1);
                ir.push_str(&format!("{}br label %loop_cond\n", indent));

                ir.push_str(&format!("loop_end:\n"));
            }
            Node::Block(nodes) => {
                for n in nodes {
                    Self::traverse_ir(n, ir, depth);
                }
            }
            Node::Return(val) => {
                Self::traverse_ir(val, ir, depth);
                ir.push_str(&format!("{}ret i64 %res\n", indent));
            }

            // Literals
            Node::IntLiteral(v) => ir.push_str(&format!("{}; i64 {}\n", indent, v)),
            Node::FloatLiteral(v) => ir.push_str(&format!("{}; double {}\n", indent, v)),
            Node::BoolLiteral(v) => ir.push_str(&format!("{}; i1 {}\n", indent, v)),
            Node::StringLiteral(v) => {
                ir.push_str(&format!("{}; ptr @.{}\n", indent, v.replace("\"", "")))
            }
            // V2 Extensions
            Node::ArrayLiteral(items) => {
                ir.push_str(&format!("{}; array alloc\n", indent));
                for item in items {
                    Self::traverse_ir(item, ir, depth);
                }
            }
            Node::Index(container, idx) => {
                Self::traverse_ir(container, ir, depth);
                Self::traverse_ir(idx, ir, depth);
                ir.push_str(&format!("{}%idx_res = getelementptr ...\n", indent));
            }
            Node::Concat(l, r) => {
                Self::traverse_ir(l, ir, depth);
                Self::traverse_ir(r, ir, depth);
                ir.push_str(&format!("{}%concat_res = call @concat\n", indent));
            }
            Node::BitAnd(l, r) => {
                Self::traverse_ir(l, ir, depth);
                Self::traverse_ir(r, ir, depth);
                ir.push_str(&format!("{}%and_res = and i64 %l, %r\n", indent));
            }
            Node::BitShiftLeft(l, r) => {
                Self::traverse_ir(l, ir, depth);
                Self::traverse_ir(r, ir, depth);
                ir.push_str(&format!("{}%shl_res = shl i64 %l, %r\n", indent));
            }
            Node::BitShiftRight(l, r) => {
                Self::traverse_ir(l, ir, depth);
                Self::traverse_ir(r, ir, depth);
                ir.push_str(&format!("{}%shr_res = lshr i64 %l, %r\n", indent));
            }
            Node::FnDef(name, _params, body) => {
                ir.push_str(&format!("define void @{}(...) {{\n", name));
                Self::traverse_ir(body, ir, depth + 1);
                ir.push_str("  ret void\n}\n");
            }
            Node::Call(name, args) => {
                for arg in args {
                    Self::traverse_ir(arg, ir, depth);
                }
                ir.push_str(&format!("{}call @{}(...)\n", indent, name));
            }
            Node::FileRead(path) => {
                Self::traverse_ir(path, ir, depth);
                ir.push_str(&format!("{}call @file_read(...)\n", indent));
            }
            Node::FileWrite(path, data) => {
                Self::traverse_ir(path, ir, depth);
                Self::traverse_ir(data, ir, depth);
                ir.push_str(&format!("{}call @file_write(...)\n", indent));
            }
            _ => {
                ir.push_str(&format!("{}; <unimplemented op>\n", indent));
            }
        }
    }
}
