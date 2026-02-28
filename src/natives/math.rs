use super::NativeModule;
use crate::executor::{ExecResult, RelType};
use noise::{NoiseFn, Perlin};

pub struct MathModule;

impl NativeModule for MathModule {
    fn handle(&self, func_name: &str, args: &[RelType]) -> Option<ExecResult> {
        match func_name {
            "Math.Random" => Some(ExecResult::Value(RelType::Float(rand::random::<f64>()))),
            "Math.Sin" => {
                if args.len() != 1 {
                    return Some(ExecResult::Fault("Math.Sin expects 1 argument".to_string()));
                }
                match args[0] {
                    RelType::Float(f) => Some(ExecResult::Value(RelType::Float(f.sin()))),
                    RelType::Int(i) => Some(ExecResult::Value(RelType::Float((i as f64).sin()))),
                    _ => Some(ExecResult::Fault("Math.Sin expects a Number".to_string())),
                }
            }
            "Math.Cos" => {
                if args.len() != 1 {
                    return Some(ExecResult::Fault("Math.Cos expects 1 argument".to_string()));
                }
                match args[0] {
                    RelType::Float(f) => Some(ExecResult::Value(RelType::Float(f.cos()))),
                    RelType::Int(i) => Some(ExecResult::Value(RelType::Float((i as f64).cos()))),
                    _ => Some(ExecResult::Fault("Math.Cos expects a Number".to_string())),
                }
            }
            "Math.Floor" => {
                if args.len() != 1 {
                    return Some(ExecResult::Fault(
                        "Math.Floor expects 1 argument".to_string(),
                    ));
                }
                match args[0] {
                    RelType::Float(f) => Some(ExecResult::Value(RelType::Float(f.floor()))),
                    RelType::Int(i) => Some(ExecResult::Value(RelType::Int(i))),
                    _ => Some(ExecResult::Fault("Math.Floor expects a Number".to_string())),
                }
            }
            "Math.Ceil" => {
                if args.len() != 1 {
                    return Some(ExecResult::Fault(
                        "Math.Ceil expects 1 argument".to_string(),
                    ));
                }
                match args[0] {
                    RelType::Float(f) => Some(ExecResult::Value(RelType::Float(f.ceil()))),
                    RelType::Int(i) => Some(ExecResult::Value(RelType::Int(i))),
                    _ => Some(ExecResult::Fault("Math.Ceil expects a Number".to_string())),
                }
            }
            "Math.Perlin2D" => {
                if args.len() != 2 {
                    return Some(ExecResult::Fault(
                        "Math.Perlin2D expects 2 arguments (x, y)".to_string(),
                    ));
                }
                let x = match args[0] {
                    RelType::Float(f) => f,
                    RelType::Int(i) => i as f64,
                    _ => {
                        return Some(ExecResult::Fault(
                            "Math.Perlin2D arg 1 must be a Number".to_string(),
                        ));
                    }
                };
                let y = match args[1] {
                    RelType::Float(f) => f,
                    RelType::Int(i) => i as f64,
                    _ => {
                        return Some(ExecResult::Fault(
                            "Math.Perlin2D arg 2 must be a Number".to_string(),
                        ));
                    }
                };
                let perlin = Perlin::new(1); // Explicit seed for stability
                let val = perlin.get([x, y]);
                Some(ExecResult::Value(RelType::Float(val)))
            }
            _ => None,
        }
    }
}
