//! CLI module
//!
//! Command-line interface for scripting, automation, and daemon control.

mod args;
mod commands;

pub use args::Args;
pub use commands::*;
