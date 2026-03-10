use crate::executor::{ExecResult, RelType, AgentPermissions};
use crate::natives::NativeModule;

pub struct IoModule;

impl NativeModule for IoModule {
    fn handle(&self, func_name: &str, args: &[RelType], permissions: &AgentPermissions) -> Option<ExecResult> {
        match func_name {
            "IO.WriteFile" => {
                if !permissions.allow_fs_write {
                    return Some(ExecResult::Fault { 
                        msg: "Permission Denied: IO.WriteFile requires FS_WRITE".to_string(), 
                        node: "Native::IO.WriteFile".into() 
                    });
                }
                if args.len() != 2 {
                    return Some(ExecResult::Fault {
                        msg: "IO.WriteFile expects 2 arguments (path, content)".to_string(),
                        node: "Native::IO.WriteFile".into()
                    });
                }
                if let (RelType::Str(path), RelType::Str(content)) = (&args[0], &args[1]) {
                    match std::fs::write(path, content) {
                        Ok(_) => Some(ExecResult::Value(RelType::Bool(true))),
                        Err(_) => Some(ExecResult::Value(RelType::Bool(false))),
                    }
                } else {
                    Some(ExecResult::Fault {
                        msg: "IO.WriteFile expects (String, String)".to_string(),
                        node: "Native::IO.WriteFile".into()
                    })
                }
            }
            "IO.ReadFile" => {
                if !permissions.allow_fs_read {
                    return Some(ExecResult::Fault { 
                        msg: "Permission Denied: IO.ReadFile requires FS_READ".to_string(), 
                        node: "Native::IO.ReadFile".into() 
                    });
                }
                if args.len() != 1 {
                    return Some(ExecResult::Fault {
                        msg: "IO.ReadFile expects 1 argument (path)".to_string(),
                        node: "Native::IO.ReadFile".into()
                    });
                }
                if let RelType::Str(path) = &args[0] {
                    match std::fs::read_to_string(path) {
                        Ok(content) => Some(ExecResult::Value(RelType::Str(content))),
                        Err(_) => Some(ExecResult::Value(RelType::Str("".to_string()))),
                    }
                } else {
                    Some(ExecResult::Fault {
                        msg: "IO.ReadFile expects a String".to_string(),
                        node: "Native::IO.ReadFile".into()
                    })
                }
            }
            "IO.AppendFile" => {
                if !permissions.allow_fs_write {
                    return Some(ExecResult::Fault { 
                        msg: "Permission Denied: IO.AppendFile requires FS_WRITE".to_string(), 
                        node: "Native::IO.AppendFile".into() 
                    });
                }
                if args.len() != 2 {
                    return Some(ExecResult::Fault {
                        msg: "IO.AppendFile expects 2 arguments (path, content)".to_string(),
                        node: "Native::IO.AppendFile".into()
                    });
                }
                if let (RelType::Str(path), RelType::Str(content)) = (&args[0], &args[1]) {
                    use std::io::Write;
                    let mut file = match std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)
                    {
                        Ok(f) => f,
                        Err(_) => return Some(ExecResult::Value(RelType::Bool(false))),
                    };
                    match write!(file, "{}", content) {
                        Ok(_) => Some(ExecResult::Value(RelType::Bool(true))),
                        Err(_) => Some(ExecResult::Value(RelType::Bool(false))),
                    }
                } else {
                    Some(ExecResult::Fault {
                        msg: "IO.AppendFile expects (String, String)".to_string(),
                        node: "Native::IO.AppendFile".into()
                    })
                }
            }
            "IO.FileExists" => {
                if !permissions.allow_fs_read {
                    return Some(ExecResult::Fault { 
                        msg: "Permission Denied: IO.FileExists requires FS_READ".to_string(), 
                        node: "Native::IO.FileExists".into() 
                    });
                }
                if args.len() != 1 {
                    return Some(ExecResult::Fault {
                        msg: "IO.FileExists expects 1 argument (path)".to_string(),
                        node: "Native::IO.FileExists".into()
                    });
                }
                if let RelType::Str(path) = &args[0] {
                    Some(ExecResult::Value(RelType::Bool(
                        std::path::Path::new(path).exists(),
                    )))
                } else {
                    Some(ExecResult::Fault {
                        msg: "IO.FileExists expects a String".to_string(),
                        node: "Native::IO.FileExists".into()
                    })
                }
            }
            _ => None,
        }
    }
}
