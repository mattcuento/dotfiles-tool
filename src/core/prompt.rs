use crate::core::config::LanguageManager;
use crate::error::Result;
use dialoguer::{Confirm, Input, Select};
use std::path::PathBuf;

pub fn prompt_dotfiles_dir() -> Result<PathBuf> {
    let default = dirs::home_dir()
        .unwrap()
        .join("Development")
        .join("dotfiles");

    let path: String = Input::new()
        .with_prompt("Dotfiles directory location")
        .default(default.to_string_lossy().to_string())
        .interact_text()
        .map_err(|e| crate::error::DotfilesError::Config(e.to_string()))?;

    Ok(PathBuf::from(path))
}

pub fn prompt_xdg_config_home() -> Result<PathBuf> {
    let default = dirs::home_dir().unwrap().join(".config");

    let path: String = Input::new()
        .with_prompt("XDG_CONFIG_HOME location")
        .default(default.to_string_lossy().to_string())
        .interact_text()
        .map_err(|e| crate::error::DotfilesError::Config(e.to_string()))?;

    Ok(PathBuf::from(path))
}

pub fn prompt_language_manager() -> Result<LanguageManager> {
    let options = vec!["asdf", "mise", "rtx", "none"];
    let selection = Select::new()
        .with_prompt("Language manager")
        .items(&options)
        .default(0)
        .interact()
        .map_err(|e| crate::error::DotfilesError::Config(e.to_string()))?;

    Ok(match selection {
        0 => LanguageManager::Asdf,
        1 => LanguageManager::Mise,
        2 => LanguageManager::Rtx,
        _ => LanguageManager::None,
    })
}

pub fn confirm_install_deps() -> Result<bool> {
    Confirm::new()
        .with_prompt("Install missing dependencies?")
        .default(true)
        .interact()
        .map_err(|e| crate::error::DotfilesError::Config(e.to_string()))
}
