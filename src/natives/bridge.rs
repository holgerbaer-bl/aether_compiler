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
        } else if module == "ui" {
            match function {
                "ui_init_window" => {
                    if args.len() == 3 {
                        let w = match &args[0] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault(
                                    "[FFI] ui_init_window: arg 1 must be Int (width)".to_string(),
                                ));
                            }
                        };
                        let h = match &args[1] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault(
                                    "[FFI] ui_init_window: arg 2 must be Int (height)".to_string(),
                                ));
                            }
                        };
                        let title = match &args[2] {
                            RelType::Str(v) => v.clone(),
                            _ => {
                                return Some(ExecResult::Fault(
                                    "[FFI] ui_init_window: arg 3 must be String (title)"
                                        .to_string(),
                                ));
                            }
                        };
                        let ok = crate::natives::ui::ui_init_window(w, h, title);
                        Some(ExecResult::Value(RelType::Bool(ok)))
                    } else {
                        Some(ExecResult::Fault(
                            "[FFI] ui_init_window expects 3 args (width, height, title)"
                                .to_string(),
                        ))
                    }
                }
                "ui_clear" => {
                    if args.len() == 1 {
                        if let RelType::Int(c) = &args[0] {
                            crate::natives::ui::ui_clear(*c);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] ui_clear expects 1 Int arg (color)".to_string(),
                    ))
                }
                "ui_draw_rect" => {
                    if args.len() == 5 {
                        let x = match &args[0] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault(
                                    "[FFI] ui_draw_rect: x must be Int".to_string(),
                                ));
                            }
                        };
                        let y = match &args[1] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault(
                                    "[FFI] ui_draw_rect: y must be Int".to_string(),
                                ));
                            }
                        };
                        let w = match &args[2] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault(
                                    "[FFI] ui_draw_rect: w must be Int".to_string(),
                                ));
                            }
                        };
                        let h = match &args[3] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault(
                                    "[FFI] ui_draw_rect: h must be Int".to_string(),
                                ));
                            }
                        };
                        let c = match &args[4] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault(
                                    "[FFI] ui_draw_rect: color must be Int".to_string(),
                                ));
                            }
                        };
                        crate::natives::ui::ui_draw_rect(x, y, w, h, c);
                        Some(ExecResult::Value(RelType::Void))
                    } else {
                        Some(ExecResult::Fault(
                            "[FFI] ui_draw_rect expects 5 args (x, y, w, h, color)".to_string(),
                        ))
                    }
                }
                "ui_draw_text" => {
                    if args.len() == 4 {
                        let x = match &args[0] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault(
                                    "[FFI] ui_draw_text: x must be Int".to_string(),
                                ));
                            }
                        };
                        let y = match &args[1] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault(
                                    "[FFI] ui_draw_text: y must be Int".to_string(),
                                ));
                            }
                        };
                        let text = match &args[2] {
                            RelType::Str(v) => v.clone(),
                            _ => {
                                return Some(ExecResult::Fault(
                                    "[FFI] ui_draw_text: text must be String".to_string(),
                                ));
                            }
                        };
                        let c = match &args[3] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault(
                                    "[FFI] ui_draw_text: color must be Int".to_string(),
                                ));
                            }
                        };
                        crate::natives::ui::ui_draw_text(x, y, text, c);
                        Some(ExecResult::Value(RelType::Void))
                    } else {
                        Some(ExecResult::Fault(
                            "[FFI] ui_draw_text expects 4 args (x, y, text, color)".to_string(),
                        ))
                    }
                }
                "ui_present" => {
                    let open = crate::natives::ui::ui_present();
                    Some(ExecResult::Value(RelType::Bool(open)))
                }
                "ui_is_key_down" => {
                    if args.len() == 1 {
                        if let RelType::Str(key) = &args[0] {
                            let down = crate::natives::ui::ui_is_key_down(key.clone());
                            return Some(ExecResult::Value(RelType::Bool(down)));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] ui_is_key_down expects 1 String arg".to_string(),
                    ))
                }
                "ui_get_key_pressed" => {
                    let key = crate::natives::ui::ui_get_key_pressed();
                    Some(ExecResult::Value(RelType::Str(key)))
                }
                _ => None,
            }
        } else if module == "fs" {
            match function {
                "fs_read_file" => {
                    if args.len() == 1 {
                        if let RelType::Str(path) = &args[0] {
                            let content = crate::natives::fs::fs_read_file(path.clone());
                            return Some(ExecResult::Value(RelType::Str(content)));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] fs_read_file expects 1 String arg (path)".to_string(),
                    ))
                }
                "fs_parse_json" => {
                    if args.len() == 1 {
                        if let RelType::Str(json_str) = &args[0] {
                            let result = crate::natives::fs::fs_parse_json(json_str);
                            return Some(ExecResult::Value(result));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] fs_parse_json expects 1 String arg (json)".to_string(),
                    ))
                }
                "obj_has_key" => {
                    if args.len() == 2 {
                        if let (RelType::Object(map), RelType::Str(key)) = (&args[0], &args[1]) {
                            return Some(ExecResult::Value(RelType::Bool(map.contains_key(key))));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] obj_has_key expects (Object, String)".to_string(),
                    ))
                }
                "obj_set" => {
                    if args.len() == 3 {
                        if let (RelType::Object(map), RelType::Str(key)) = (&args[0], &args[1]) {
                            let mut new_map = map.clone();
                            new_map.insert(key.clone(), args[2].clone());
                            return Some(ExecResult::Value(RelType::Object(new_map)));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] obj_set expects (Object, String, Any)".to_string(),
                    ))
                }
                "obj_get" => {
                    if args.len() == 2 {
                        if let (RelType::Object(map), RelType::Str(key)) = (&args[0], &args[1]) {
                            return Some(ExecResult::Value(
                                map.get(key).cloned().unwrap_or(RelType::Void),
                            ));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] obj_get expects (Object, String)".to_string(),
                    ))
                }
                "array_length" => {
                    if args.len() == 1 {
                        if let RelType::Array(arr) = &args[0] {
                            return Some(ExecResult::Value(RelType::Int(arr.len() as i64)));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] array_length expects 1 Array arg".to_string(),
                    ))
                }
                "array_get" => {
                    if args.len() == 2 {
                        if let (RelType::Array(arr), RelType::Int(idx)) = (&args[0], &args[1]) {
                            let i = *idx as usize;
                            if i < arr.len() {
                                return Some(ExecResult::Value(arr[i].clone()));
                            }
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] array_get expects (Array, Int)".to_string(),
                    ))
                }
                _ => None,
            }
        } else if module == "registry" {
            match function {
                "registry_create_counter" => {
                    let id = crate::natives::registry::registry_create_counter();
                    Some(ExecResult::Value(RelType::Handle(id)))
                }
                "registry_increment" => {
                    if args.len() == 1 {
                        if let RelType::Handle(id) = &args[0] {
                            crate::natives::registry::registry_increment(*id);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] registry_increment expects 1 Handle arg".to_string(),
                    ))
                }
                "registry_get_value" => {
                    if args.len() == 1 {
                        if let RelType::Handle(id) = &args[0] {
                            let val = crate::natives::registry::registry_get_value(*id);
                            return Some(ExecResult::Value(RelType::Int(val)));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] registry_get_value expects 1 Handle arg".to_string(),
                    ))
                }
                "registry_free" => {
                    if args.len() == 1 {
                        if let RelType::Handle(id) = &args[0] {
                            crate::natives::registry::registry_free(*id);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] registry_free expects 1 Handle arg".to_string(),
                    ))
                }
                "registry_retain" => {
                    if args.len() == 1 {
                        if let RelType::Handle(id) = &args[0] {
                            crate::natives::registry::registry_retain(*id);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] registry_retain expects 1 Handle arg".to_string(),
                    ))
                }
                "registry_release" => {
                    if args.len() == 1 {
                        if let RelType::Handle(id) = &args[0] {
                            crate::natives::registry::registry_release(*id);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] registry_release expects 1 Handle arg".to_string(),
                    ))
                }
                "registry_create_window" => {
                    if args.len() == 3 {
                        if let (RelType::Int(w), RelType::Int(h), RelType::Str(title)) =
                            (&args[0], &args[1], &args[2])
                        {
                            let id = crate::natives::registry::registry_create_window(
                                *w,
                                *h,
                                title.clone(),
                            );
                            return Some(ExecResult::Value(RelType::Handle(id)));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] registry_create_window expects (Int, Int, String)".to_string(),
                    ))
                }
                "registry_window_update" => {
                    if args.len() == 1 {
                        if let RelType::Handle(id) = &args[0] {
                            let open = crate::natives::registry::registry_window_update(*id);
                            return Some(ExecResult::Value(RelType::Bool(open)));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] registry_window_update expects 1 Handle arg".to_string(),
                    ))
                }
                "registry_window_close" => {
                    if args.len() == 1 {
                        if let RelType::Handle(id) = &args[0] {
                            crate::natives::registry::registry_window_close(*id);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] registry_window_close expects 1 Handle arg".to_string(),
                    ))
                }
                "registry_dump" => {
                    let total = crate::natives::registry::registry_dump();
                    Some(ExecResult::Value(RelType::Int(total)))
                }
                "registry_file_create" => {
                    if args.len() == 1 {
                        if let RelType::Str(path) = &args[0] {
                            let id = crate::natives::registry::registry_file_create(path.clone());
                            return Some(ExecResult::Value(RelType::Handle(id)));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] registry_file_create expects 1 String arg".to_string(),
                    ))
                }
                "registry_file_write" => {
                    if args.len() == 2 {
                        if let (RelType::Handle(id), RelType::Str(content)) = (&args[0], &args[1]) {
                            crate::natives::registry::registry_file_write(*id, content.clone());
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault(
                        "[FFI] registry_file_write expects (Handle, String)".to_string(),
                    ))
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
