use crate::executor::{ExecResult, RelType, AgentPermissions};

pub mod bridge;
pub mod fs;
pub mod io;
pub mod math;
pub mod registry;
pub mod ui;

pub trait NativeModule: Send {
    fn handle(&self, func_name: &str, args: &[RelType], permissions: &AgentPermissions) -> Option<ExecResult>;
}
