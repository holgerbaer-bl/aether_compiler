use crate::executor::{ExecResult, RelType, AgentPermissions};

pub trait BridgeModule: Send {
    fn handle(&self, module: &str, function: &str, args: &[RelType], permissions: &AgentPermissions) -> Option<ExecResult>;
}

pub struct CoreBridge;

impl BridgeModule for CoreBridge {
    fn handle(&self, module: &str, function: &str, args: &[RelType], permissions: &AgentPermissions) -> Option<ExecResult> {
        if module == "test_lib" {
            match function {
                "calculate_hash" => {
                    if args.len() == 1
                        && let RelType::Str(data) = &args[0]
                    {
                        let result = crate::test_lib::calculate_hash(data.clone());
                        return Some(ExecResult::Value(RelType::Int(result)));
                    }
                    Some(ExecResult::Fault {
                        msg: "calculate_hash expects 1 String argument".to_string(),
                        node: "Native::Bridge::calculate_hash".into()
                    })
                }
                "greet_user" => {
                    if args.len() == 1
                        && let RelType::Str(name) = &args[0]
                    {
                        let result = crate::test_lib::greet_user(name.clone());
                        return Some(ExecResult::Value(RelType::Str(result)));
                    }
                    Some(ExecResult::Fault {
                        msg: "greet_user expects 1 String argument".to_string(),
                        node: "Native::Bridge::greet_user".into()
                    })
                }
                "normalize_vector" => {
                    if args.len() == 1
                        && let RelType::Object(map) = &args[0]
                    {
                        let x = if let Some(RelType::Float(v)) = map.get("x") {
                            *v
                        } else {
                            return Some(ExecResult::Fault {
                                msg: "[FFI Error] normalize_vector missing required float field 'x'"
                                    .to_string(),
                                node: "Native::Bridge::normalize_vector".into()
                            });
                        };
                        let y = if let Some(RelType::Float(v)) = map.get("y") {
                            *v
                        } else {
                            return Some(ExecResult::Fault {
                                msg: "[FFI Error] normalize_vector missing required float field 'y'"
                                    .to_string(),
                                node: "Native::Bridge::normalize_vector".into()
                            });
                        };
                        let z = if let Some(RelType::Float(v)) = map.get("z") {
                            *v
                        } else {
                            return Some(ExecResult::Fault {
                                msg: "[FFI Error] normalize_vector missing required float field 'z'"
                                    .to_string(),
                                node: "Native::Bridge::normalize_vector".into()
                            });
                        };

                        let input_vec = crate::test_lib::Vector3 { x, y, z };
                        let out_vec = crate::test_lib::normalize_vector(input_vec);

                        let mut out_map = std::collections::HashMap::new();
                        out_map.insert("x".to_string(), RelType::Float(out_vec.x));
                        out_map.insert("y".to_string(), RelType::Float(out_vec.y));
                        out_map.insert("z".to_string(), RelType::Float(out_vec.z));

                        return Some(ExecResult::Value(RelType::Object(out_map)));
                    }
                    Some(ExecResult::Fault {
                        msg: "normalize_vector expects 1 Vector3 Object argument".to_string(),
                        node: "Native::Bridge::normalize_vector".into()
                    })
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
                                return Some(ExecResult::Fault {
                                    msg: "[FFI] ui_init_window: arg 1 must be Int (width)".to_string(),
                                    node: "Native::Bridge::ui_init_window".into()
                                });
                            }
                        };
                        let h = match &args[1] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault {
                                    msg: "[FFI] ui_init_window: arg 2 must be Int (height)".to_string(),
                                    node: "Native::Bridge::ui_init_window".into()
                                });
                            }
                        };
                        let title = match &args[2] {
                            RelType::Str(v) => v.clone(),
                            _ => {
                                return Some(ExecResult::Fault {
                                    msg: "[FFI] ui_init_window: arg 3 must be String (title)"
                                        .to_string(),
                                    node: "Native::Bridge::ui_init_window".into()
                                });
                            }
                        };
                        let ok = crate::natives::ui::ui_init_window(w, h, title);
                        Some(ExecResult::Value(RelType::Bool(ok)))
                    } else {
                        Some(ExecResult::Fault {
                            msg: "[FFI] ui_init_window expects 3 args (width, height, title)"
                                .to_string(),
                            node: "Native::Bridge::ui_init_window".into()
                        })
                    }
                }
                "ui_clear" => {
                    if args.len() == 1 {
                        if let RelType::Int(c) = &args[0] {
                            crate::natives::ui::ui_clear(*c);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] ui_clear expects 1 Int arg (color)".to_string(),
                        node: "Native::Bridge::ui_clear".into()
                    })
                }
                "ui_draw_rect" => {
                    if args.len() == 5 {
                        let x = match &args[0] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault {
                                    msg: "[FFI] ui_draw_rect: x must be Int".to_string(),
                                    node: "Native::Bridge::ui_draw_rect".into()
                                });
                            }
                        };
                        let y = match &args[1] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault {
                                    msg: "[FFI] ui_draw_rect: y must be Int".to_string(),
                                    node: "Native::Bridge::ui_draw_rect".into()
                                });
                            }
                        };
                        let w = match &args[2] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault {
                                    msg: "[FFI] ui_draw_rect: w must be Int".to_string(),
                                    node: "Native::Bridge::ui_draw_rect".into()
                                });
                            }
                        };
                        let h = match &args[3] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault {
                                    msg: "[FFI] ui_draw_rect: h must be Int".to_string(),
                                    node: "Native::Bridge::ui_draw_rect".into()
                                });
                            }
                        };
                        let c = match &args[4] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault {
                                    msg: "[FFI] ui_draw_rect: color must be Int".to_string(),
                                    node: "Native::Bridge::ui_draw_rect".into()
                                });
                            }
                        };
                        crate::natives::ui::ui_draw_rect(x, y, w, h, c);
                        Some(ExecResult::Value(RelType::Void))
                    } else {
                        Some(ExecResult::Fault {
                            msg: "[FFI] ui_draw_rect expects 5 args (x, y, w, h, color)".to_string(),
                            node: "Native::Bridge::ui_draw_rect".into()
                        })
                    }
                }
                "ui_draw_text" => {
                    if args.len() == 4 {
                        let x = match &args[0] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault {
                                    msg: "[FFI] ui_draw_text: x must be Int".to_string(),
                                    node: "Native::Bridge::ui_draw_text".into()
                                });
                            }
                        };
                        let y = match &args[1] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault {
                                    msg: "[FFI] ui_draw_text: y must be Int".to_string(),
                                    node: "Native::Bridge::ui_draw_text".into()
                                });
                            }
                        };
                        let text = match &args[2] {
                            RelType::Str(v) => v.clone(),
                            _ => {
                                return Some(ExecResult::Fault {
                                    msg: "[FFI] ui_draw_text: text must be String".to_string(),
                                    node: "Native::Bridge::ui_draw_text".into()
                                });
                            }
                        };
                        let c = match &args[3] {
                            RelType::Int(v) => *v,
                            _ => {
                                return Some(ExecResult::Fault {
                                    msg: "[FFI] ui_draw_text: color must be Int".to_string(),
                                    node: "Native::Bridge::ui_draw_text".into()
                                });
                            }
                        };
                        crate::natives::ui::ui_draw_text(x, y, text, c);
                        Some(ExecResult::Value(RelType::Void))
                    } else {
                        Some(ExecResult::Fault {
                            msg: "[FFI] ui_draw_text expects 4 args (x, y, text, color)".to_string(),
                            node: "Native::Bridge::ui_draw_text".into()
                        })
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
                    Some(ExecResult::Fault {
                        msg: "[FFI] ui_is_key_down expects 1 String arg".to_string(),
                        node: "Native::Bridge::ui_is_key_down".into()
                    })
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
                    if !permissions.allow_fs_read {
                        return Some(ExecResult::Fault { 
                            msg: "Permission Denied: fs.fs_read_file requires FS_READ".to_string(), 
                            node: "Bridge::fs.fs_read_file".into() 
                        });
                    }
                    if args.len() == 1 {
                        if let RelType::Str(path) = &args[0] {
                            let content = crate::natives::fs::fs_read_file(path.clone());
                            return Some(ExecResult::Value(RelType::Str(content)));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] fs_read_file expects 1 String arg (path)".to_string(),
                        node: "Native::Bridge::fs_read_file".into()
                    })
                }
                "fs_parse_json" => {
                    if args.len() == 1 {
                        if let RelType::Str(json_str) = &args[0] {
                            let result = crate::natives::fs::fs_parse_json(json_str);
                            return Some(ExecResult::Value(result));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] fs_parse_json expects 1 String arg (json)".to_string(),
                        node: "Native::Bridge::fs_parse_json".into()
                    })
                }
                "obj_has_key" => {
                    if args.len() == 2 {
                        if let (RelType::Object(map), RelType::Str(key)) = (&args[0], &args[1]) {
                            return Some(ExecResult::Value(RelType::Bool(map.contains_key(key))));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] obj_has_key expects (Object, String)".to_string(),
                        node: "Native::Bridge::obj_has_key".into()
                    })
                }
                "obj_set" => {
                    if args.len() == 3 {
                        if let (RelType::Object(map), RelType::Str(key)) = (&args[0], &args[1]) {
                            let mut new_map = map.clone();
                            new_map.insert(key.clone(), args[2].clone());
                            return Some(ExecResult::Value(RelType::Object(new_map)));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] obj_set expects (Object, String, Any)".to_string(),
                        node: "Native::Bridge::obj_set".into()
                    })
                }
                "obj_get" => {
                    if args.len() == 2 {
                        if let (RelType::Object(map), RelType::Str(key)) = (&args[0], &args[1]) {
                            return Some(ExecResult::Value(
                                map.get(key).cloned().unwrap_or(RelType::Void),
                            ));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] obj_get expects (Object, String)".to_string(),
                        node: "Native::Bridge::obj_get".into()
                    })
                }
                "array_length" => {
                    if args.len() == 1 {
                        if let RelType::Array(arr) = &args[0] {
                            return Some(ExecResult::Value(RelType::Int(arr.len() as i64)));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] array_length expects 1 Array arg".to_string(),
                        node: "Native::Bridge::array_length".into()
                    })
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
                    Some(ExecResult::Fault {
                        msg: "[FFI] array_get expects (Array, Int)".to_string(),
                        node: "Native::Bridge::array_get".into()
                    })
                }
                _ => None,
            }
        } else if module == "registry" {
            match function {
                "registry_create_counter" => {
                    let id = crate::natives::registry::registry_create_counter();
                    Some(ExecResult::Value(RelType::Handle(crate::executor::NativeHandle(id))))
                }
                "registry_increment" => {
                    if args.len() == 1 {
                        if let RelType::Handle(crate::executor::NativeHandle(id)) = &args[0] {
                            crate::natives::registry::registry_increment(*id);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_increment expects 1 Handle arg".to_string(),
                        node: "Native::Bridge::registry_increment".into()
                    })
                }
                "registry_get_value" => {
                    if args.len() == 1 {
                        if let RelType::Handle(crate::executor::NativeHandle(id)) = &args[0] {
                            let val = crate::natives::registry::registry_get_value(*id);
                            return Some(ExecResult::Value(RelType::Int(val)));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_get_value expects 1 Handle arg".to_string(),
                        node: "Native::Bridge::registry_get_value".into()
                    })
                }
                "registry_free" => {
                    if args.len() == 1 {
                        if let RelType::Handle(crate::executor::NativeHandle(id)) = &args[0] {
                            crate::natives::registry::registry_free(*id);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_free expects 1 Handle arg".to_string(),
                        node: "Native::Bridge::registry_free".into()
                    })
                }
                "registry_retain" => {
                    if args.len() == 1 {
                        if let RelType::Handle(crate::executor::NativeHandle(id)) = &args[0] {
                            crate::natives::registry::registry_retain(*id);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_retain expects 1 Handle arg".to_string(),
                        node: "Native::Bridge::registry_retain".into()
                    })
                }
                "registry_release" => {
                    if args.len() == 1 {
                        if let RelType::Handle(crate::executor::NativeHandle(id)) = &args[0] {
                            crate::natives::registry::registry_release(*id);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_release expects 1 Handle arg".to_string(),
                        node: "Native::Bridge::registry_release".into()
                    })
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
                            return Some(ExecResult::Value(RelType::Handle(crate::executor::NativeHandle(id))));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_create_window expects (Int, Int, String)".to_string(),
                        node: "Native::Bridge::registry_create_window".into()
                    })
                }
                "registry_window_update" => {
                    if args.len() == 1 {
                        if let RelType::Handle(crate::executor::NativeHandle(id)) = &args[0] {
                            let open = crate::natives::registry::registry_window_update(*id);
                            return Some(ExecResult::Value(RelType::Bool(open)));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_window_update expects 1 Handle arg".to_string(),
                        node: "Native::Bridge::registry_window_update".into()
                    })
                }
                "registry_window_close" => {
                    if args.len() == 1 {
                        if let RelType::Handle(crate::executor::NativeHandle(id)) = &args[0] {
                            crate::natives::registry::registry_window_close(*id);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_window_close expects 1 Handle arg".to_string(),
                        node: "Native::Bridge::registry_window_close".into()
                    })
                }
                "registry_dump" => {
                    let total = crate::natives::registry::registry_dump();
                    Some(ExecResult::Value(RelType::Int(total)))
                }
                "registry_file_create" => {
                    if !permissions.allow_fs_write {
                        return Some(ExecResult::Fault { 
                            msg: "Permission Denied: registry.registry_file_create requires FS_WRITE".to_string(), 
                            node: "Bridge::registry.registry_file_create".into() 
                        });
                    }
                    if args.len() == 1 {
                        if let RelType::Str(path) = &args[0] {
                            let id = crate::natives::registry::registry_file_create(path.clone());
                            return Some(ExecResult::Value(RelType::Handle(crate::executor::NativeHandle(id))));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_file_create expects 1 String arg".to_string(),
                        node: "Native::Bridge::registry_file_create".into()
                    })
                }
                "registry_file_write" => {
                    if !permissions.allow_fs_write {
                        return Some(ExecResult::Fault { 
                            msg: "Permission Denied: registry.registry_file_write requires FS_WRITE".to_string(), 
                            node: "Bridge::registry.registry_file_write".into() 
                        });
                    }
                    if args.len() == 2 {
                        if let (RelType::Handle(crate::executor::NativeHandle(id)), RelType::Str(content)) = (&args[0], &args[1]) {
                            crate::natives::registry::registry_file_write(*id, content.clone());
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_file_write expects (Handle, String)".to_string(),
                        node: "Native::Bridge::registry_file_write".into()
                    })
                }
                "registry_now" => {
                    let id = crate::natives::registry::registry_now();
                    Some(ExecResult::Value(RelType::Handle(crate::executor::NativeHandle(id))))
                }
                "registry_elapsed_ms" => {
                    if args.len() == 1 {
                        if let RelType::Handle(crate::executor::NativeHandle(id)) = &args[0] {
                            let ms = crate::natives::registry::registry_elapsed_ms(*id);
                            return Some(ExecResult::Value(RelType::Int(ms)));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_elapsed_ms expects 1 Handle arg".to_string(),
                        node: "Native::Bridge::registry_elapsed_ms".into()
                    })
                }
                "registry_gpu_init" => {
                    let id = crate::natives::registry::registry_gpu_init();
                    Some(ExecResult::Value(RelType::Handle(crate::executor::NativeHandle(id))))
                }
                "registry_fill_color" => {
                    if args.len() == 4 {
                        if let (
                            RelType::Handle(crate::executor::NativeHandle(win)),
                            RelType::Int(r),
                            RelType::Int(g),
                            RelType::Int(b),
                        ) = (&args[0], &args[1], &args[2], &args[3])
                        {
                            crate::natives::registry::registry_fill_color(*win, *r, *g, *b);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_fill_color expects (Handle, Int, Int, Int)".to_string(),
                        node: "Native::Bridge::registry_fill_color".into()
                    })
                }
                "registry_voxel_world_create" => {
                    if args.len() == 3 {
                        if let (RelType::Int(w), RelType::Int(h), RelType::Str(title)) =
                            (&args[0], &args[1], &args[2])
                        {
                            let id = crate::natives::registry::registry_voxel_world_create(
                                *w,
                                *h,
                                title.clone(),
                            );
                            return Some(ExecResult::Value(RelType::Handle(crate::executor::NativeHandle(id))));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_voxel_world_create expects (Int, Int, String)".to_string(),
                        node: "Native::Bridge::registry_voxel_world_create".into()
                    })
                }
                "registry_voxel_add_block" => {
                    if args.len() == 4 {
                        if let (
                            RelType::Handle(crate::executor::NativeHandle(world)),
                            RelType::Int(x),
                            RelType::Int(y),
                            RelType::Int(z),
                        ) = (&args[0], &args[1], &args[2], &args[3])
                        {
                            crate::natives::registry::registry_voxel_add_block(*world, *x, *y, *z);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_voxel_add_block expects (Handle, Int, Int, Int)"
                            .to_string(),
                        node: "Native::Bridge::registry_voxel_add_block".into()
                    })
                }
                "registry_voxel_render_frame" => {
                    if args.len() == 1 {
                        if let RelType::Handle(crate::executor::NativeHandle(world)) = &args[0] {
                            let open =
                                crate::natives::registry::registry_voxel_render_frame(*world);
                            return Some(ExecResult::Value(RelType::Bool(open)));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_voxel_render_frame expects 1 Handle arg".to_string(),
                        node: "Native::Bridge::registry_voxel_render_frame".into()
                    })
                }
                "registry_texture_load" => {
                    if !permissions.allow_fs_read {
                        return Some(ExecResult::Fault { 
                            msg: "Permission Denied: registry.registry_texture_load requires FS_READ".to_string(), 
                            node: "Bridge::registry.registry_texture_load".into() 
                        });
                    }
                    if args.len() == 1 {
                        if let RelType::Str(path) = &args[0] {
                            let id = crate::natives::registry::registry_texture_load(path.clone());
                            return Some(ExecResult::Value(RelType::Handle(crate::executor::NativeHandle(id))));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_texture_load expects 1 String arg".to_string(),
                        node: "Native::Bridge::registry_texture_load".into()
                    })
                }
                "registry_draw_quad_3d" => {
                    if args.len() == 7 {
                        let get_float = |arg: &RelType| -> Option<f32> {
                            match arg {
                                RelType::Float(f) => Some(*f as f32),
                                RelType::Int(i) => Some(*i as f32),
                                _ => None,
                            }
                        };
                        if let RelType::Handle(crate::executor::NativeHandle(win)) = &args[0] {
                            if let (Some(x), Some(y), Some(z), Some(sx), Some(sy)) = (
                                get_float(&args[2]),
                                get_float(&args[3]),
                                get_float(&args[4]),
                                get_float(&args[5]),
                                get_float(&args[6]),
                            ) {
                                if let RelType::Handle(crate::executor::NativeHandle(tex)) = &args[1] {
                                    crate::natives::registry::registry_draw_quad_3d(
                                        *win, *tex, x, y, z, sx, sy,
                                    );
                                    return Some(ExecResult::Value(RelType::Void));
                                }
                            }
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_draw_quad_3d expects (Handle, Handle, Float, Float, Float, Float, Float)"
                            .to_string(),
                        node: "Native::Bridge::registry_draw_quad_3d".into()
                    })
                }
                "registry_draw_sphere" => {
                    if args.len() == 8 {
                        let get_float = |arg: &RelType| -> Option<f32> {
                            match arg {
                                RelType::Float(f) => Some(*f as f32),
                                RelType::Int(i) => Some(*i as f32),
                                _ => None,
                            }
                        };
                        let get_int = |arg: &RelType| -> Option<i64> {
                            match arg {
                                RelType::Int(i) => Some(*i),
                                _ => None,
                            }
                        };

                        if let RelType::Handle(crate::executor::NativeHandle(win)) = &args[0] {
                            if let RelType::Handle(crate::executor::NativeHandle(tex)) = &args[1] {
                                if let (Some(r), Some(rings), Some(sectors), Some(x), Some(y), Some(z)) = (
                                    get_float(&args[2]),
                                    get_int(&args[3]),
                                    get_int(&args[4]),
                                    get_float(&args[5]),
                                    get_float(&args[6]),
                                    get_float(&args[7]),
                                ) {
                                    crate::natives::registry::registry_draw_sphere(
                                        *win, *tex, r, rings as i32, sectors as i32, x, y, z,
                                    );
                                    return Some(ExecResult::Value(RelType::Void));
                                }
                            }
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_draw_sphere expects (Handle win, Handle tex, Float r, Int rings, Int sectors, Float x, Float y, Float z)"
                            .to_string(),
                        node: "Native::Bridge::registry_draw_sphere".into()
                    })
                }
                "registry_draw_cube" => {
                    if args.len() == 8 {
                        let get_float = |arg: &RelType| -> Option<f32> {
                            match arg {
                                RelType::Float(f) => Some(*f as f32),
                                RelType::Int(i) => Some(*i as f32),
                                _ => None,
                            }
                        };
                        if let (RelType::Handle(crate::executor::NativeHandle(win)), RelType::Handle(crate::executor::NativeHandle(tex))) = (&args[0], &args[1]) {
                            if let (Some(w), Some(h), Some(d), Some(x), Some(y), Some(z)) = (
                                get_float(&args[2]),
                                get_float(&args[3]),
                                get_float(&args[4]),
                                get_float(&args[5]),
                                get_float(&args[6]),
                                get_float(&args[7]),
                            ) {
                                crate::natives::registry::registry_draw_cube(
                                    *win, *tex, w, h, d, x, y, z,
                                );
                                return Some(ExecResult::Value(RelType::Void));
                            }
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_draw_cube expects (Handle win, Handle tex, Float w, Float h, Float d, Float x, Float y, Float z)"
                            .to_string(),
                        node: "Native::Bridge::registry_draw_cube".into()
                    })
                }
                "registry_draw_cylinder" => {
                    if args.len() == 8 {
                        let get_float = |arg: &RelType| -> Option<f32> {
                            match arg {
                                RelType::Float(f) => Some(*f as f32),
                                RelType::Int(i) => Some(*i as f32),
                                _ => None,
                            }
                        };
                        let get_int = |arg: &RelType| -> Option<i64> {
                            match arg {
                                RelType::Int(i) => Some(*i),
                                _ => None,
                            }
                        };
                        if let (RelType::Handle(crate::executor::NativeHandle(win)), RelType::Handle(crate::executor::NativeHandle(tex))) = (&args[0], &args[1]) {
                            if let (Some(r), Some(h), Some(s), Some(x), Some(y), Some(z)) = (
                                get_float(&args[2]),
                                get_float(&args[3]),
                                get_int(&args[4]),
                                get_float(&args[5]),
                                get_float(&args[6]),
                                get_float(&args[7]),
                            ) {
                                crate::natives::registry::registry_draw_cylinder(
                                    *win, *tex, r, h, s as i32, x, y, z,
                                );
                                return Some(ExecResult::Value(RelType::Void));
                            }
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_draw_cylinder expects (Handle win, Handle tex, Float r, Float h, Int segments, Float x, Float y, Float z)"
                            .to_string(),
                        node: "Native::Bridge::registry_draw_cylinder".into()
                    })
                }
                "registry_set_camera" => {
                    if args.len() == 4 {
                        let get_float = |arg: &RelType| -> Option<f32> {
                            match arg {
                                RelType::Float(f) => Some(*f as f32),
                                RelType::Int(i) => Some(*i as f32),
                                _ => None,
                            }
                        };
                        if let (Some(fov), Some(x), Some(y), Some(z)) = (
                            get_float(&args[0]),
                            get_float(&args[1]),
                            get_float(&args[2]),
                            get_float(&args[3]),
                        ) {
                            crate::natives::registry::registry_set_camera(fov, x, y, z);
                            return Some(ExecResult::Value(RelType::Void));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_set_camera expects (Float, Float, Float, Float)"
                            .to_string(),
                        node: "Native::Bridge::registry_set_camera".into()
                    })
                }
                "registry_is_key_pressed" => {
                    if args.len() == 1 {
                        if let RelType::Int(code) = &args[0] {
                            let pressed = crate::natives::registry::registry_is_key_pressed(*code);
                            return Some(ExecResult::Value(RelType::Float(pressed as f64)));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_is_key_pressed expects 1 Int arg".to_string(),
                        node: "Native::Bridge::registry_is_key_pressed".into()
                    })
                }
                "registry_get_mouse_delta_x" => {
                    if args.is_empty() {
                        let dx = crate::natives::registry::registry_get_mouse_delta_x();
                        return Some(ExecResult::Value(RelType::Float(dx as f64)));
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_get_mouse_delta_x expects 0 args".to_string(),
                        node: "Native::Bridge::registry_get_mouse_delta_x".into()
                    })
                }
                "registry_get_mouse_delta_y" => {
                    if args.is_empty() {
                        let dy = crate::natives::registry::registry_get_mouse_delta_y();
                        return Some(ExecResult::Value(RelType::Float(dy as f64)));
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_get_mouse_delta_y expects 0 args".to_string(),
                        node: "Native::Bridge::registry_get_mouse_delta_y".into()
                    })
                }
                "registry_get_last_char" => {
                    if args.is_empty() {
                        let c = crate::natives::registry::registry_get_last_char();
                        return Some(ExecResult::Value(RelType::Int(c)));
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_get_last_char expects 0 args".to_string(),
                        node: "Native::Bridge::registry_get_last_char".into()
                    })
                }
                "registry_read_file" => {
                    if !permissions.allow_fs_read {
                        return Some(ExecResult::Fault { 
                            msg: "Permission Denied: registry.registry_read_file requires FS_READ".to_string(), 
                            node: "Bridge::registry.registry_read_file".into() 
                        });
                    }
                    if args.len() == 1 {
                        if let RelType::Str(path) = &args[0] {
                            let content = crate::natives::registry::registry_read_file(path.clone());
                            return Some(ExecResult::Value(RelType::Str(content)));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_read_file expects 1 String arg".to_string(),
                        node: "Native::Bridge::registry_read_file".into()
                    })
                }
                "registry_write_file" => {
                    if !permissions.allow_fs_write {
                        return Some(ExecResult::Fault { 
                            msg: "Permission Denied: registry.registry_write_file requires FS_WRITE".to_string(), 
                            node: "Bridge::registry.registry_write_file".into() 
                        });
                    }
                    if args.len() == 2 {
                        if let (RelType::Str(path), RelType::Str(content)) = (&args[0], &args[1]) {
                            let ok = crate::natives::registry::registry_write_file(path.clone(), content.clone());
                            return Some(ExecResult::Value(RelType::Bool(ok)));
                        }
                    }
                    Some(ExecResult::Fault {
                        msg: "[FFI] registry_write_file expects (String, String)".to_string(),
                        node: "Native::Bridge::registry_write_file".into()
                    })
                }
                "registry_get_ultimate_answer" => {
                    Some(ExecResult::Value(RelType::Int(crate::natives::registry::registry_get_ultimate_answer())))
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
