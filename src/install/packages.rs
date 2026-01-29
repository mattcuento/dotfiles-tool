use crate::error::Result;

/// Essential packages to install for dotfiles management
pub const ESSENTIAL_PACKAGES: &[&str] = &[
    "stow", // GNU Stow for symlink management
    "fzf",  // Fuzzy finder
    "bat",  // Better cat
    "fd",   // Better find
    "tree", // Directory tree viewer
    "nvim", // Neovim editor
    "tmux", // Terminal multiplexer
];

/// Optional but recommended packages
pub const OPTIONAL_PACKAGES: &[&str] = &[
    "ripgrep", // Better grep
    "git",     // Version control
    "curl",    // HTTP client
    "wget",    // File downloader
];

/// Installs a single package via Homebrew (idempotent)
pub fn install_package(package: &str) -> Result<()> {
    if crate::install::homebrew::is_package_installed(package) {
        println!("✓ {} is already installed", package);
        return Ok(());
    }

    crate::install::homebrew::install_package(package)
}

/// Installs all essential packages
pub fn install_essential_packages() -> Result<Vec<String>> {
    let mut installed = Vec::new();

    println!("Installing essential packages...");

    for package in ESSENTIAL_PACKAGES {
        match install_package(package) {
            Ok(()) => {
                installed.push(package.to_string());
            }
            Err(e) => {
                eprintln!("Warning: Failed to install {}: {}", package, e);
                // Continue with other packages even if one fails
            }
        }
    }

    if !installed.is_empty() {
        println!("✓ Installed {} essential packages", installed.len());
    }

    Ok(installed)
}

/// Installs optional packages
pub fn install_optional_packages() -> Result<Vec<String>> {
    let mut installed = Vec::new();

    println!("Installing optional packages...");

    for package in OPTIONAL_PACKAGES {
        match install_package(package) {
            Ok(()) => {
                installed.push(package.to_string());
            }
            Err(e) => {
                eprintln!("Warning: Failed to install {}: {}", package, e);
                // Continue with other packages even if one fails
            }
        }
    }

    if !installed.is_empty() {
        println!("✓ Installed {} optional packages", installed.len());
    }

    Ok(installed)
}

/// Checks if all essential packages are installed
pub fn check_essential_packages() -> Vec<String> {
    ESSENTIAL_PACKAGES
        .iter()
        .filter(|pkg| !crate::install::homebrew::is_package_installed(pkg))
        .map(|pkg| pkg.to_string())
        .collect()
}

/// Returns a summary of package installation status
pub fn package_status() -> PackageStatus {
    let missing_essential: Vec<String> = ESSENTIAL_PACKAGES
        .iter()
        .filter(|pkg| !crate::install::homebrew::is_package_installed(pkg))
        .map(|pkg| pkg.to_string())
        .collect();

    let installed_essential: Vec<String> = ESSENTIAL_PACKAGES
        .iter()
        .filter(|pkg| crate::install::homebrew::is_package_installed(pkg))
        .map(|pkg| pkg.to_string())
        .collect();

    let installed_optional: Vec<String> = OPTIONAL_PACKAGES
        .iter()
        .filter(|pkg| crate::install::homebrew::is_package_installed(pkg))
        .map(|pkg| pkg.to_string())
        .collect();

    PackageStatus {
        missing_essential,
        installed_essential,
        installed_optional,
    }
}

/// Package installation status summary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageStatus {
    pub missing_essential: Vec<String>,
    pub installed_essential: Vec<String>,
    pub installed_optional: Vec<String>,
}

impl PackageStatus {
    /// Returns true if all essential packages are installed
    pub fn is_complete(&self) -> bool {
        self.missing_essential.is_empty()
    }

    /// Returns the total number of installed packages
    pub fn total_installed(&self) -> usize {
        self.installed_essential.len() + self.installed_optional.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_essential_packages_list() {
        assert_eq!(ESSENTIAL_PACKAGES.len(), 7);
        assert!(ESSENTIAL_PACKAGES.contains(&"stow"));
        assert!(ESSENTIAL_PACKAGES.contains(&"fzf"));
        assert!(ESSENTIAL_PACKAGES.contains(&"bat"));
        assert!(ESSENTIAL_PACKAGES.contains(&"fd"));
        assert!(ESSENTIAL_PACKAGES.contains(&"tree"));
        assert!(ESSENTIAL_PACKAGES.contains(&"nvim"));
        assert!(ESSENTIAL_PACKAGES.contains(&"tmux"));
    }

    #[test]
    fn test_optional_packages_list() {
        assert_eq!(OPTIONAL_PACKAGES.len(), 4);
        assert!(OPTIONAL_PACKAGES.contains(&"ripgrep"));
        assert!(OPTIONAL_PACKAGES.contains(&"git"));
        assert!(OPTIONAL_PACKAGES.contains(&"curl"));
        assert!(OPTIONAL_PACKAGES.contains(&"wget"));
    }

    #[test]
    fn test_check_essential_packages() {
        // This test checks that the function runs without panicking
        // The actual result depends on what's installed on the system
        let missing = check_essential_packages();

        // Missing packages should all be from the essential list
        for pkg in &missing {
            assert!(
                ESSENTIAL_PACKAGES.contains(&pkg.as_str()),
                "Package {} is not in ESSENTIAL_PACKAGES",
                pkg
            );
        }
    }

    #[test]
    fn test_package_status() {
        // Test that package_status runs without panicking
        let status = package_status();

        // All missing packages should be essential packages
        for pkg in &status.missing_essential {
            assert!(
                ESSENTIAL_PACKAGES.contains(&pkg.as_str()),
                "Package {} is not in ESSENTIAL_PACKAGES",
                pkg
            );
        }

        // All installed essential packages should be essential packages
        for pkg in &status.installed_essential {
            assert!(
                ESSENTIAL_PACKAGES.contains(&pkg.as_str()),
                "Package {} is not in ESSENTIAL_PACKAGES",
                pkg
            );
        }

        // All installed optional packages should be optional packages
        for pkg in &status.installed_optional {
            assert!(
                OPTIONAL_PACKAGES.contains(&pkg.as_str()),
                "Package {} is not in OPTIONAL_PACKAGES",
                pkg
            );
        }
    }

    #[test]
    fn test_package_status_completeness() {
        let status = package_status();

        // If no essential packages are missing, is_complete should be true
        if status.missing_essential.is_empty() {
            assert!(status.is_complete());
        } else {
            assert!(!status.is_complete());
        }
    }

    #[test]
    fn test_package_status_total() {
        let status = package_status();
        let total = status.total_installed();

        assert_eq!(
            total,
            status.installed_essential.len() + status.installed_optional.len()
        );
    }
}
