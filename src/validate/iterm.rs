use crate::validate::{CheckReport, CheckResult};
use std::path::Path;
use std::process::Command;

/// Validates iTerm2 configuration (macOS only)
pub fn validate_iterm_config(dotfiles_dir: &Path) -> CheckReport {
    let mut report = CheckReport::new();

    // Check if iTerm2 plist exists in dotfiles
    report.add(check_iterm_plist_in_dotfiles(dotfiles_dir));

    // Check if custom preferences path is configured
    report.add(check_iterm_custom_prefs());

    report
}

fn check_iterm_plist_in_dotfiles(dotfiles_dir: &Path) -> CheckResult {
    let possible_locations = vec![
        dotfiles_dir.join(".config/iterm2/com.googlecode.iterm2.plist"),
        dotfiles_dir.join("iterm/com.googlecode.iterm2.plist"),
        dotfiles_dir.join("iterm2/com.googlecode.iterm2.plist"),
    ];

    for path in possible_locations {
        if path.exists() {
            return CheckResult::pass(
                "iTerm Plist",
                format!("iTerm config found: {}", path.display()),
            );
        }
    }

    CheckResult::warn(
        "iTerm Plist",
        "iTerm configuration not found in dotfiles",
        Some("Export iTerm preferences to dotfiles directory"),
    )
}

fn check_iterm_custom_prefs() -> CheckResult {
    let output = Command::new("defaults")
        .arg("read")
        .arg("com.googlecode.iterm2")
        .arg("PrefsCustomFolder")
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            CheckResult::pass(
                "iTerm Custom Folder",
                format!("Custom preferences folder: {}", path),
            )
        }
        _ => CheckResult::warn(
            "iTerm Custom Folder",
            "iTerm not using custom preferences folder",
            Some("Configure in iTerm2: Preferences → General → Preferences → Load preferences from folder"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_check_iterm_plist_in_config_iterm2() {
        let temp = TempDir::new().unwrap();
        let plist_path = temp
            .path()
            .join(".config/iterm2/com.googlecode.iterm2.plist");

        fs::create_dir_all(plist_path.parent().unwrap()).unwrap();
        fs::write(&plist_path, "test plist content").unwrap();

        let result = check_iterm_plist_in_dotfiles(temp.path());
        assert!(result.is_pass());
        assert!(result.message().contains(".config/iterm2"));
    }

    #[test]
    fn test_check_iterm_plist_in_iterm_dir() {
        let temp = TempDir::new().unwrap();
        let plist_path = temp.path().join("iterm/com.googlecode.iterm2.plist");

        fs::create_dir_all(plist_path.parent().unwrap()).unwrap();
        fs::write(&plist_path, "test plist content").unwrap();

        let result = check_iterm_plist_in_dotfiles(temp.path());
        assert!(result.is_pass());
        assert!(result.message().contains("iterm"));
    }

    #[test]
    fn test_check_iterm_plist_in_iterm2_dir() {
        let temp = TempDir::new().unwrap();
        let plist_path = temp.path().join("iterm2/com.googlecode.iterm2.plist");

        fs::create_dir_all(plist_path.parent().unwrap()).unwrap();
        fs::write(&plist_path, "test plist content").unwrap();

        let result = check_iterm_plist_in_dotfiles(temp.path());
        assert!(result.is_pass());
        assert!(result.message().contains("iterm2"));
    }

    #[test]
    fn test_check_iterm_plist_not_found() {
        let temp = TempDir::new().unwrap();
        let result = check_iterm_plist_in_dotfiles(temp.path());
        assert!(result.is_warn());
        assert!(result.message().contains("not found"));
    }

    #[test]
    fn test_validate_iterm_config() {
        let temp = TempDir::new().unwrap();
        let report = validate_iterm_config(temp.path());

        // Should have 2 checks: plist and custom folder
        assert_eq!(report.checks.len(), 2);
    }
}
