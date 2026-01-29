use crate::error::{DotfilesError, Result};
use std::path::PathBuf;
use std::process::Command;

/// Supported version managers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionManager {
    Asdf,
    Mise,
    Rtx, // Older name for mise
}

impl VersionManager {
    /// Returns the command name for this version manager
    pub fn command(&self) -> &str {
        match self {
            VersionManager::Asdf => "asdf",
            VersionManager::Mise => "mise",
            VersionManager::Rtx => "rtx",
        }
    }

    /// Returns the display name for this version manager
    pub fn display_name(&self) -> &str {
        match self {
            VersionManager::Asdf => "ASDF",
            VersionManager::Mise => "mise",
            VersionManager::Rtx => "rtx",
        }
    }

    /// Returns the Homebrew package name
    pub fn homebrew_package(&self) -> &str {
        match self {
            VersionManager::Asdf => "asdf",
            VersionManager::Mise => "mise",
            VersionManager::Rtx => "rtx",
        }
    }
}

/// Detects which version manager is installed
pub fn detect() -> Option<VersionManager> {
    // Check in order of preference: mise, asdf, rtx
    [
        VersionManager::Mise,
        VersionManager::Asdf,
        VersionManager::Rtx,
    ]
    .into_iter()
    .find(|&vm| is_installed(vm))
}

/// Checks if a specific version manager is installed
pub fn is_installed(vm: VersionManager) -> bool {
    Command::new("which")
        .arg(vm.command())
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Gets the path to the version manager executable
pub fn get_path(vm: VersionManager) -> Option<PathBuf> {
    let output = Command::new("which").arg(vm.command()).output().ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout);
        Some(PathBuf::from(path.trim()))
    } else {
        None
    }
}

/// Installs a version manager using Homebrew
pub fn install(vm: VersionManager) -> Result<()> {
    if is_installed(vm) {
        println!("{} is already installed", vm.display_name());
        return Ok(());
    }

    println!("Installing {}...", vm.display_name());
    crate::install::homebrew::install_package(vm.homebrew_package())?;

    println!("{} installed successfully!", vm.display_name());
    Ok(())
}

/// Installs the preferred version manager (mise) if none is installed
pub fn install_preferred() -> Result<VersionManager> {
    if let Some(vm) = detect() {
        println!("{} is already installed", vm.display_name());
        return Ok(vm);
    }

    let preferred = VersionManager::Mise;
    install(preferred)?;
    Ok(preferred)
}

/// Installs a language runtime using the specified version manager
pub fn install_language(vm: VersionManager, language: &str, version: &str) -> Result<()> {
    let vm_path = get_path(vm)
        .ok_or_else(|| DotfilesError::DependencyMissing(vm.display_name().to_string()))?;

    println!(
        "Installing {} {} using {}...",
        language,
        version,
        vm.display_name()
    );

    // Add plugin first (for asdf)
    if vm == VersionManager::Asdf {
        let _ = Command::new(&vm_path)
            .arg("plugin")
            .arg("add")
            .arg(language)
            .output();
    }

    // Install the language version
    let status = Command::new(&vm_path)
        .arg("install")
        .arg(language)
        .arg(version)
        .status()?;

    if !status.success() {
        return Err(DotfilesError::InstallationFailed(format!(
            "Failed to install {} {}",
            language, version
        )));
    }

    // Set as global version
    let status = Command::new(&vm_path)
        .arg("global")
        .arg(language)
        .arg(version)
        .status()?;

    if !status.success() {
        return Err(DotfilesError::InstallationFailed(format!(
            "Failed to set {} {} as global",
            language, version
        )));
    }

    println!("{} {} installed and set as global!", language, version);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_manager_command() {
        assert_eq!(VersionManager::Asdf.command(), "asdf");
        assert_eq!(VersionManager::Mise.command(), "mise");
        assert_eq!(VersionManager::Rtx.command(), "rtx");
    }

    #[test]
    fn test_version_manager_display_name() {
        assert_eq!(VersionManager::Asdf.display_name(), "ASDF");
        assert_eq!(VersionManager::Mise.display_name(), "mise");
        assert_eq!(VersionManager::Rtx.display_name(), "rtx");
    }

    #[test]
    fn test_version_manager_homebrew_package() {
        assert_eq!(VersionManager::Asdf.homebrew_package(), "asdf");
        assert_eq!(VersionManager::Mise.homebrew_package(), "mise");
        assert_eq!(VersionManager::Rtx.homebrew_package(), "rtx");
    }

    #[test]
    fn test_detect() {
        // This test will return Some(vm) if any version manager is installed
        // or None if none are installed. Both are valid outcomes.
        let result = detect();

        if let Some(vm) = result {
            // If a version manager is detected, verify it's actually installed
            assert!(
                is_installed(vm),
                "{} should be installed",
                vm.display_name()
            );
        }
    }

    #[test]
    fn test_is_installed_consistency() {
        // For each version manager, is_installed should match whether detect() returns it
        let detected = detect();

        for vm in [
            VersionManager::Asdf,
            VersionManager::Mise,
            VersionManager::Rtx,
        ] {
            let installed = is_installed(vm);

            // If this VM is detected, it should report as installed
            if Some(vm) == detected {
                assert!(
                    installed,
                    "{} is detected but not reported as installed",
                    vm.display_name()
                );
            }
        }
    }

    #[test]
    fn test_get_path_consistency() {
        // If a version manager is installed, get_path should return Some
        for vm in [
            VersionManager::Asdf,
            VersionManager::Mise,
            VersionManager::Rtx,
        ] {
            let installed = is_installed(vm);
            let path = get_path(vm);

            if installed {
                assert!(
                    path.is_some(),
                    "{} is installed but get_path returned None",
                    vm.display_name()
                );
            } else {
                assert!(
                    path.is_none(),
                    "{} is not installed but get_path returned Some",
                    vm.display_name()
                );
            }
        }
    }
}
