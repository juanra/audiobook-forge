//! CLI module

mod commands;
mod handlers;

pub use commands::{Cli, Commands};
pub use handlers::{handle_build, handle_check, handle_config, handle_organize, handle_metadata};
