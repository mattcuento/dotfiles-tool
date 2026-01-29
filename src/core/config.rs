use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub dotfiles_dir: PathBuf,
    pub xdg_config_home: PathBuf,
    pub language_manager: LanguageManager,
    pub symlink_method: SymlinkMethod,
    pub install_oh_my_zsh: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum LanguageManager {
    Asdf,
    Mise,
    Rtx,
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SymlinkMethod {
    Stow,
    Manual,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let toml = toml::to_string_pretty(self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_roundtrip() {
        let config = Config {
            dotfiles_dir: PathBuf::from("/home/user/dotfiles"),
            xdg_config_home: PathBuf::from("/home/user/.config"),
            language_manager: LanguageManager::Asdf,
            symlink_method: SymlinkMethod::Stow,
            install_oh_my_zsh: true,
        };

        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();
        assert_eq!(config.dotfiles_dir, parsed.dotfiles_dir);
    }
}
