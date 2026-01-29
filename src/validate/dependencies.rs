use crate::validate::{CheckReport, CheckResult};

/// Validates that Homebrew is installed (macOS only)
pub fn check_homebrew() -> CheckResult {
    if !cfg!(target_os = "macos") {
        return CheckResult::pass("Homebrew", "Not required (not on macOS)");
    }

    if crate::install::homebrew::is_installed() {
        let path = crate::install::homebrew::get_brew_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        CheckResult::pass("Homebrew", format!("Installed at {}", path))
    } else {
        CheckResult::error(
            "Homebrew",
            "Not installed",
            Some("Install with: /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""),
        )
    }
}

/// Validates that a version manager is installed
pub fn check_version_manager() -> CheckResult {
    if let Some(vm) = crate::install::version_manager::detect() {
        let path = crate::install::version_manager::get_path(vm)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        CheckResult::pass(
            "Version Manager",
            format!("{} installed at {}", vm.display_name(), path),
        )
    } else {
        CheckResult::warn(
            "Version Manager",
            "No version manager detected (ASDF, mise, or rtx)",
            Some("Install mise with: brew install mise"),
        )
    }
}

/// Validates that a specific tool is installed
pub fn check_tool(tool: &str) -> CheckResult {
    if crate::detect::tools::is_installed(tool) {
        let path = crate::detect::tools::get_tool_path(tool)
            .unwrap_or_else(|| "unknown".to_string());
        CheckResult::pass(tool, format!("Installed at {}", path))
    } else {
        let suggestion = match tool {
            "stow" => "brew install stow",
            "git" => "brew install git",
            "fzf" => "brew install fzf",
            "bat" => "brew install bat",
            "fd" => "brew install fd",
            "tree" => "brew install tree",
            "nvim" => "brew install nvim",
            "tmux" => "brew install tmux",
            "ripgrep" => "brew install ripgrep",
            _ => "brew install <package>",
        };

        CheckResult::error(
            tool,
            "Not installed",
            Some(format!("Install with: {}", suggestion)),
        )
    }
}

/// Validates all dependencies
pub fn validate_all() -> CheckReport {
    let mut report = CheckReport::new();

    // Check Homebrew
    report.add(check_homebrew());

    // Check version manager
    report.add(check_version_manager());

    // Check essential tools
    for tool in crate::install::packages::ESSENTIAL_PACKAGES {
        report.add(check_tool(tool));
    }

    report
}

/// Validates only critical dependencies (Homebrew and stow)
pub fn validate_critical() -> CheckReport {
    let mut report = CheckReport::new();

    report.add(check_homebrew());
    report.add(check_tool("stow"));

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_homebrew() {
        let result = check_homebrew();

        #[cfg(target_os = "macos")]
        {
            // On macOS, should either pass or error (depending on installation)
            assert!(result.is_pass() || result.is_error());
            if result.is_error() {
                assert!(result.suggestion().is_some());
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // On non-macOS, should pass with "not required" message
            assert!(result.is_pass());
            assert!(result.message().contains("Not required"));
        }
    }

    #[test]
    fn test_check_version_manager() {
        let result = check_version_manager();

        // Should either pass (if installed) or warn (if not)
        assert!(result.is_pass() || result.is_warn());

        if result.is_warn() {
            assert!(result.suggestion().is_some());
            assert!(result
                .suggestion()
                .unwrap()
                .contains("mise") || result.suggestion().unwrap().contains("asdf"));
        }
    }

    #[test]
    fn test_check_tool_stow() {
        let result = check_tool("stow");

        // Should be either pass or error
        assert!(result.is_pass() || result.is_error());
        assert_eq!(result.name(), "stow");

        if result.is_error() {
            assert!(result.suggestion().is_some());
            assert!(result.suggestion().unwrap().contains("brew install"));
        }
    }

    #[test]
    fn test_check_tool_unknown() {
        let result = check_tool("definitely_not_installed_tool_12345");

        assert!(result.is_error());
        assert_eq!(result.name(), "definitely_not_installed_tool_12345");
        assert!(result.suggestion().is_some());
    }

    #[test]
    fn test_validate_critical() {
        let report = validate_critical();

        // Should have 2 checks: Homebrew and stow
        assert_eq!(report.total(), 2);

        // Check that we have the right checks
        let names: Vec<&str> = report.checks.iter().map(|c| c.name()).collect();
        assert!(names.contains(&"Homebrew"));
        assert!(names.contains(&"stow"));
    }

    #[test]
    fn test_validate_all() {
        let report = validate_all();

        // Should have Homebrew + Version Manager + all essential packages
        // That's 2 + ESSENTIAL_PACKAGES.len()
        let expected = 2 + crate::install::packages::ESSENTIAL_PACKAGES.len();
        assert_eq!(report.total(), expected);

        // Check that Homebrew is included
        assert!(report.checks.iter().any(|c| c.name() == "Homebrew"));

        // Check that version manager is included
        assert!(report
            .checks
            .iter()
            .any(|c| c.name() == "Version Manager"));

        // Check that stow is included
        assert!(report.checks.iter().any(|c| c.name() == "stow"));
    }

    #[test]
    fn test_check_tool_suggestions() {
        // Test that common tools have specific suggestions
        let tools = vec!["stow", "git", "fzf", "nvim"];

        for tool in tools {
            let result = check_tool(tool);
            if result.is_error() {
                assert!(result.suggestion().is_some());
                let suggestion = result.suggestion().unwrap();
                assert!(suggestion.contains("brew install"));
                assert!(suggestion.contains(tool));
            }
        }
    }
}
