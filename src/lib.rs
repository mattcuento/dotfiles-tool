pub mod core;
pub mod detect;
pub mod error;

// Re-export commonly used types
pub use core::config::{Config, LanguageManager, SymlinkMethod};
pub use error::{DotfilesError, Result};
