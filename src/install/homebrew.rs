use crate::error::{DotfilesError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Possible Homebrew installation paths
const HOMEBREW_PATHS: &[&str] = &[
    "/opt/homebrew/bin/brew", // ARM Mac (M1/M2/M3)
    "/usr/local/bin/brew",    // Intel Mac
];

/// Official Homebrew installation script URL
const HOMEBREW_INSTALL_URL: &str =
    "https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh";

/// Detects if Homebrew is installed and returns its path
pub fn detect_homebrew() -> Option<PathBuf> {
    HOMEBREW_PATHS
        .iter()
        .map(Path::new)
        .find(|path| path.exists())
        .map(|path| path.to_path_buf())
}

/// Checks if Homebrew is installed
pub fn is_installed() -> bool {
    detect_homebrew().is_some()
}

/// Gets the path to the brew executable
pub fn get_brew_path() -> Option<PathBuf> {
    detect_homebrew()
}

/// Installs Homebrew using the official installation script
pub fn install() -> Result<()> {
    if is_installed() {
        return Ok(());
    }

    println!("Installing Homebrew...");

    let status = Command::new("bash")
        .arg("-c")
        .arg(format!(
            r#"/bin/bash -c "$(curl -fsSL {})""#,
            HOMEBREW_INSTALL_URL
        ))
        .status()?;

    if !status.success() {
        return Err(DotfilesError::InstallationFailed(
            "Homebrew installation failed".to_string(),
        ));
    }

    println!("Homebrew installed successfully!");
    Ok(())
}

/// Installs a package using Homebrew
pub fn install_package(package: &str) -> Result<()> {
    let brew_path =
        get_brew_path().ok_or_else(|| DotfilesError::DependencyMissing("Homebrew".to_string()))?;

    println!("Installing {}...", package);

    let status = Command::new(brew_path)
        .arg("install")
        .arg(package)
        .status()?;

    if !status.success() {
        return Err(DotfilesError::InstallationFailed(format!(
            "Failed to install {}",
            package
        )));
    }

    Ok(())
}

/// Checks if a package is installed via Homebrew
pub fn is_package_installed(package: &str) -> bool {
    if let Some(brew_path) = get_brew_path() {
        let output = Command::new(brew_path).arg("list").arg(package).output();

        if let Ok(output) = output {
            return output.status.success();
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homebrew_paths_constant() {
        assert_eq!(HOMEBREW_PATHS.len(), 2);
        assert!(HOMEBREW_PATHS.contains(&"/opt/homebrew/bin/brew"));
        assert!(HOMEBREW_PATHS.contains(&"/usr/local/bin/brew"));
    }

    #[test]
    fn test_detect_homebrew() {
        // This test will pass if homebrew is installed on the system
        // Otherwise it will return None, which is also valid
        let result = detect_homebrew();

        // If homebrew is found, it should be one of the known paths
        if let Some(path) = result {
            let path_str = path.to_str().unwrap();
            assert!(
                path_str.contains("/opt/homebrew/bin/brew")
                    || path_str.contains("/usr/local/bin/brew"),
                "Unexpected homebrew path: {}",
                path_str
            );
        }
    }

    #[test]
    fn test_is_installed_consistency() {
        // is_installed() should match whether detect_homebrew() returns Some
        assert_eq!(is_installed(), detect_homebrew().is_some());
    }

    #[test]
    fn test_get_brew_path_consistency() {
        // get_brew_path() should match detect_homebrew()
        assert_eq!(get_brew_path(), detect_homebrew());
    }

    #[test]
    fn test_is_package_installed_with_common_package() {
        // Test with a commonly installed package if brew exists
        if is_installed() {
            // git is typically installed on most systems
            // This is a smoke test - we're just checking it doesn't panic
            let _ = is_package_installed("git");
        }
    }
}
