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
                    if args.len() == 1
                        && let RelType::Str(data) = &args[0]
                    {
                        let result = crate::test_lib::calculate_hash(data.clone());
                        return Some(ExecResult::Value(RelType::Int(result)));
                    }
                    Some(ExecResult::Fault(
                        "calculate_hash expects 1 String argument".to_string(),
                    ))
                }
                "greet_user" => {
                    if args.len() == 1
                        && let RelType::Str(name) = &args[0]
                    {
                        let result = crate::test_lib::greet_user(name.clone());
                        return Some(ExecResult::Value(RelType::Str(result)));
                    }
                    Some(ExecResult::Fault(
                        "greet_user expects 1 String argument".to_string(),
                    ))
                }
                "normalize_vector" => {
                    if args.len() == 1
                        && let RelType::Object(map) = &args[0]
                    {
                        let x = if let Some(RelType::Float(v)) = map.get("x") {
                            *v
                        } else {
                            return Some(ExecResult::Fault(
                                "[FFI Error] normalize_vector missing required float field 'x'"
                                    .to_string(),
                            ));
                        };
                        let y = if let Some(RelType::Float(v)) = map.get("y") {
                            *v
                        } else {
                            return Some(ExecResult::Fault(
                                "[FFI Error] normalize_vector missing required float field 'y'"
                                    .to_string(),
                            ));
                        };
                        let z = if let Some(RelType::Float(v)) = map.get("z") {
                            *v
                        } else {
                            return Some(ExecResult::Fault(
                                "[FFI Error] normalize_vector missing required float field 'z'"
                                    .to_string(),
                            ));
                        };

                        let input_vec = crate::test_lib::Vector3 { x, y, z };
                        let out_vec = crate::test_lib::normalize_vector(input_vec);

                        let mut out_map = std::collections::HashMap::new();
                        out_map.insert("x".to_string(), RelType::Float(out_vec.x));
                        out_map.insert("y".to_string(), RelType::Float(out_vec.y));
                        out_map.insert("z".to_string(), RelType::Float(out_vec.z));

                        return Some(ExecResult::Value(RelType::Object(out_map)));
                    }
                    Some(ExecResult::Fault(
                        "normalize_vector expects 1 Vector3 Object argument".to_string(),
                    ))
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
