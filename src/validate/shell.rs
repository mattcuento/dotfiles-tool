use crate::validate::{CheckReport, CheckResult};
use std::fs;
use std::path::Path;

/// Validates shell integration (scripts sourced in .zshrc)
pub fn validate_shell_integration(home_dir: &Path, dotfiles_dir: &Path) -> CheckReport {
    let mut report = CheckReport::new();

    let zshrc = home_dir.join(".zshrc");
    let script = dotfiles_dir.join("scripts/check-claude-changes.sh");

    if zshrc.exists() {
        report.add(check_script_sourced(
            &zshrc,
            &script,
            "check-claude-changes.sh",
        ));
    } else {
        report.add(CheckResult::warn(
            "Shell RC",
            "~/.zshrc not found",
            Some("Create .zshrc or use different shell"),
        ));
    }

    report
}

fn check_script_sourced(shell_rc: &Path, script_path: &Path, script_name: &str) -> CheckResult {
    if !script_path.exists() {
        return CheckResult::warn(
            "Sync Script",
            format!("{} not found in dotfiles", script_name),
            Some("Ensure script exists in dotfiles/scripts/"),
        );
    }

    match fs::read_to_string(shell_rc) {
        Ok(content) => {
            let script_str = script_path.to_str().unwrap_or("");

            // Check for source or . commands
            // Also check for just the script name in case of relative paths
            if content.contains(&format!("source {}", script_str))
                || content.contains(&format!(". {}", script_str))
                || content.contains(script_name)
            {
                CheckResult::pass("Sync Script", format!("{} is sourced", script_name))
            } else {
                CheckResult::error(
                    "Sync Script",
                    format!("{} not sourced in .zshrc", script_name),
                    Some("Run: dotfiles setup to add source line"),
                )
            }
        }
        Err(e) => CheckResult::error(
            "Shell RC",
            format!("Failed to read .zshrc: {}", e),
            None::<String>,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_check_script_sourced_with_source_command() {
        let temp = TempDir::new().unwrap();
        let zshrc = temp.path().join(".zshrc");
        let script = temp.path().join("scripts/check-claude-changes.sh");

        // Create script directory and file
        fs::create_dir_all(script.parent().unwrap()).unwrap();
        fs::write(&script, "#!/bin/bash\necho test").unwrap();

        // Create .zshrc with source command
        fs::write(&zshrc, format!("source {}", script.to_str().unwrap())).unwrap();

        let result = check_script_sourced(&zshrc, &script, "check-claude-changes.sh");
        assert!(result.is_pass());
    }

    #[test]
    fn test_check_script_sourced_with_dot_command() {
        let temp = TempDir::new().unwrap();
        let zshrc = temp.path().join(".zshrc");
        let script = temp.path().join("scripts/check-claude-changes.sh");

        fs::create_dir_all(script.parent().unwrap()).unwrap();
        fs::write(&script, "#!/bin/bash\necho test").unwrap();

        // Create .zshrc with . command
        fs::write(&zshrc, format!(". {}", script.to_str().unwrap())).unwrap();

        let result = check_script_sourced(&zshrc, &script, "check-claude-changes.sh");
        assert!(result.is_pass());
    }

    #[test]
    fn test_check_script_sourced_with_script_name_only() {
        let temp = TempDir::new().unwrap();
        let zshrc = temp.path().join(".zshrc");
        let script = temp.path().join("scripts/check-claude-changes.sh");

        fs::create_dir_all(script.parent().unwrap()).unwrap();
        fs::write(&script, "#!/bin/bash\necho test").unwrap();

        // Create .zshrc with just script name (relative path)
        fs::write(&zshrc, "source ~/dotfiles/scripts/check-claude-changes.sh").unwrap();

        let result = check_script_sourced(&zshrc, &script, "check-claude-changes.sh");
        assert!(result.is_pass());
    }

    #[test]
    fn test_check_script_not_sourced() {
        let temp = TempDir::new().unwrap();
        let zshrc = temp.path().join(".zshrc");
        let script = temp.path().join("scripts/check-claude-changes.sh");

        fs::create_dir_all(script.parent().unwrap()).unwrap();
        fs::write(&script, "#!/bin/bash\necho test").unwrap();

        // Create .zshrc without sourcing the script
        fs::write(&zshrc, "# Some other config").unwrap();

        let result = check_script_sourced(&zshrc, &script, "check-claude-changes.sh");
        assert!(result.is_error());
    }

    #[test]
    fn test_check_script_sourced_script_missing() {
        let temp = TempDir::new().unwrap();
        let zshrc = temp.path().join(".zshrc");
        let script = temp.path().join("scripts/check-claude-changes.sh");

        fs::write(&zshrc, "# Some config").unwrap();

        let result = check_script_sourced(&zshrc, &script, "check-claude-changes.sh");
        assert!(result.is_warn());
        assert!(result.message().contains("not found"));
    }

    #[test]
    fn test_check_script_sourced_zshrc_unreadable() {
        let temp = TempDir::new().unwrap();
        let zshrc = temp.path().join(".zshrc");
        let script = temp.path().join("scripts/check-claude-changes.sh");

        fs::create_dir_all(script.parent().unwrap()).unwrap();
        fs::write(&script, "#!/bin/bash\necho test").unwrap();

        // Don't create .zshrc, making it unreadable
        let result = check_script_sourced(&zshrc, &script, "check-claude-changes.sh");
        assert!(result.is_error());
        assert!(result.message().contains("Failed to read"));
    }

    #[test]
    fn test_validate_shell_integration_zshrc_missing() {
        let temp = TempDir::new().unwrap();
        let home = temp.path().join("home");
        let dotfiles = temp.path().join("dotfiles");

        fs::create_dir(&home).unwrap();
        fs::create_dir(&dotfiles).unwrap();

        let report = validate_shell_integration(&home, &dotfiles);
        assert_eq!(report.checks.len(), 1);
        assert!(report.checks[0].is_warn());
        assert!(report.checks[0].message().contains(".zshrc not found"));
    }
}
