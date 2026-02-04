//! Tools module - agent capabilities for interacting with the environment.

pub mod base;
pub mod filesystem;
pub mod registry;
pub mod shell;

// Tool trait is used internally but not exported to Python
pub use filesystem::{EditFileTool, ListDirTool, ReadFileTool, WriteFileTool};
pub use registry::ToolRegistry;
pub use shell::ExecTool;
