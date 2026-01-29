use thiserror::Error;

#[derive(Error, Debug)]
pub enum DotfilesError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Symlink conflict: {0}")]
    SymlinkConflict(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Installation failed: {0}")]
    InstallFailed(String),

    #[error("Installation failed: {0}")]
    InstallationFailed(String),

    #[error("Dependency missing: {0}")]
    DependencyMissing(String),

    #[error("Symlink operation failed: {0}")]
    SymlinkFailed(String),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, DotfilesError>;
