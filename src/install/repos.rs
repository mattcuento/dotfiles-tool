use crate::error::{DotfilesError, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Repository configuration for cloning
pub struct RepoConfig {
    pub url: String,
    pub target_path: PathBuf,
    pub name: String,
}

/// Clones a git repository if it doesn't exist
pub fn clone_repo(config: &RepoConfig) -> Result<()> {
    if config.target_path.exists() {
        println!(
            "{}",
            format!(
                "  ✓ {} repository already exists at {}",
                config.name,
                config.target_path.display()
            )
            .green()
        );
        return Ok(());
    }

    println!("  Cloning {} repository...", config.name);
    println!("    From: {}", config.url.cyan());
    println!(
        "    To: {}",
        config.target_path.display().to_string().cyan()
    );

    // Create parent directory if needed
    if let Some(parent) = config.target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let status = Command::new("git")
        .arg("clone")
        .arg(&config.url)
        .arg(&config.target_path)
        .status()
        .map_err(|e| {
            DotfilesError::InstallationFailed(format!("Failed to execute git clone: {}", e))
        })?;

    if !status.success() {
        return Err(DotfilesError::InstallationFailed(format!(
            "Failed to clone {} repository",
            config.name
        )));
    }

    println!(
        "{}",
        format!("  ✓ {} repository cloned successfully", config.name).green()
    );
    Ok(())
}

/// Clones the dotfiles repository
pub fn clone_dotfiles_repo(target_dir: &Path, repo_url: &str) -> Result<()> {
    let config = RepoConfig {
        url: repo_url.to_string(),
        target_path: target_dir.to_path_buf(),
        name: "dotfiles".to_string(),
    };

    clone_repo(&config)
}

/// Clones the claude repository
pub fn clone_claude_repo(repo_url: &str) -> Result<()> {
    let home = dirs::home_dir()
        .ok_or_else(|| DotfilesError::Config("Could not determine home directory".to_string()))?;

    let config = RepoConfig {
        url: repo_url.to_string(),
        target_path: home.join(".claude"),
        name: "claude".to_string(),
    };

    clone_repo(&config)
}

/// Checks if a directory is a git repository
pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_git_repo_returns_true_for_git_repo() {
        let temp = TempDir::new().unwrap();
        let git_dir = temp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        assert!(is_git_repo(temp.path()));
    }

    #[test]
    fn test_is_git_repo_returns_false_for_non_git_repo() {
        let temp = TempDir::new().unwrap();
        assert!(!is_git_repo(temp.path()));
    }

    #[test]
    fn test_is_git_repo_returns_false_for_nonexistent_path() {
        let path = Path::new("/nonexistent/path");
        assert!(!is_git_repo(path));
    }

    #[test]
    fn test_clone_repo_succeeds_when_already_exists() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join("existing-repo");
        fs::create_dir(&repo_path).unwrap();

        let config = RepoConfig {
            url: "https://example.com/repo.git".to_string(),
            target_path: repo_path.clone(),
            name: "test".to_string(),
        };

        let result = clone_repo(&config);
        assert!(result.is_ok());
        assert!(repo_path.exists());
    }
}
