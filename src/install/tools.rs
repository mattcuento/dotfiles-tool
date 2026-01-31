use crate::error::{DotfilesError, Result};
use colored::Colorize;
use std::path::Path;
use std::process::Command;

/// Installs TPM (Tmux Plugin Manager)
pub fn install_tpm(home_dir: &Path) -> Result<()> {
    let tpm_path = home_dir.join(".tmux/plugins/tpm");

    if tpm_path.exists() {
        println!(
            "{}",
            format!("  ✓ TPM already installed at {}", tpm_path.display()).green()
        );
        return Ok(());
    }

    println!("  Installing TPM (Tmux Plugin Manager)...");

    // Create parent directory
    if let Some(parent) = tpm_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Clone TPM repository
    let status = Command::new("git")
        .arg("clone")
        .arg("https://github.com/tmux-plugins/tpm")
        .arg(&tpm_path)
        .status()
        .map_err(|e| {
            DotfilesError::InstallationFailed(format!("Failed to execute git clone: {}", e))
        })?;

    if !status.success() {
        return Err(DotfilesError::InstallationFailed(
            "TPM installation failed".to_string(),
        ));
    }

    println!("{}", "  ✓ TPM installed successfully".green());
    println!("    Run 'tmux source ~/.tmux.conf' and press prefix + I to install plugins");
    Ok(())
}

/// Checks if TPM is installed
pub fn is_tpm_installed(home_dir: &Path) -> bool {
    home_dir.join(".tmux/plugins/tpm").exists()
}

/// Provides information about Mason (auto-installs with Neovim)
pub fn setup_mason_info() -> Result<()> {
    println!("  {}", "Mason LSP Manager:".bold());
    println!("    Mason will auto-install when you first launch Neovim");
    println!("    Inside nvim, run :Mason to manage language servers");
    println!("    Install LSPs with :MasonInstall <server-name>");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_tpm_installed_when_installed() {
        let temp = TempDir::new().unwrap();
        let tpm_path = temp.path().join(".tmux/plugins/tpm");

        fs::create_dir_all(&tpm_path).unwrap();

        assert!(is_tpm_installed(temp.path()));
    }

    #[test]
    fn test_is_tpm_installed_when_not_installed() {
        let temp = TempDir::new().unwrap();
        assert!(!is_tpm_installed(temp.path()));
    }

    #[test]
    fn test_install_tpm_when_already_installed() {
        let temp = TempDir::new().unwrap();
        let tpm_path = temp.path().join(".tmux/plugins/tpm");

        fs::create_dir_all(&tpm_path).unwrap();

        let result = install_tpm(temp.path());
        assert!(result.is_ok());
        assert!(tpm_path.exists());
    }

    #[test]
    fn test_setup_mason_info() {
        let result = setup_mason_info();
        assert!(result.is_ok());
    }
}
