use crate::executor::{ExecResult, RelType};

pub trait BridgeModule {
    fn handle(&self, module: &str, function: &str, args: &[RelType]) -> Option<ExecResult>;
}

pub struct CoreBridge;

impl BridgeModule for CoreBridge {
    fn handle(&self, module: &str, function: &str, args: &[RelType]) -> Option<ExecResult> {
        if module == "test_lib" {
            match function {
                "calculate_hash" => {
                    if args.len() == 1 {
                        if let RelType::Str(data) = &args[0] {
                            let result = crate::test_lib::calculate_hash(data.clone());
                            return Some(ExecResult::Value(RelType::Int(result)));
                        }
                    }
                    Some(ExecResult::Fault(
                        "calculate_hash expects 1 String argument".to_string(),
                    ))
                }
                "greet_user" => {
                    if args.len() == 1 {
                        if let RelType::Str(name) = &args[0] {
                            let result = crate::test_lib::greet_user(name.clone());
                            return Some(ExecResult::Value(RelType::Str(result)));
                        }
                    }
                    Some(ExecResult::Fault(
                        "greet_user expects 1 String argument".to_string(),
                    ))
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
