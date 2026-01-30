pub mod backup;
pub mod commands;
pub mod core;
pub mod detect;
pub mod error;
pub mod install;
pub mod language;
pub mod symlink;
pub mod validate;

// Re-export commonly used types
pub use core::config::{Config, LanguageManager, SymlinkMethod};
pub use error::{DotfilesError, Result};
