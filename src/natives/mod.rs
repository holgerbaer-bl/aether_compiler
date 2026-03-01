use crate::executor::{ExecResult, RelType};

pub mod bridge;
pub mod fs;
pub mod io;
pub mod math;
pub mod ui;

pub trait NativeModule {
    fn handle(&self, func_name: &str, args: &[RelType]) -> Option<ExecResult>;
}
