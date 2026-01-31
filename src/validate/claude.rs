use crate::validate::{CheckReport, CheckResult};
use std::path::Path;
use std::process::Command;

/// Validates .claude directory setup
pub fn validate_claude_directory(home_dir: &Path, dotfiles_dir: &Path) -> CheckReport {
    let mut report = CheckReport::new();

    let claude_dir = home_dir.join(".claude");

    // Check .claude directory exists
    report.add(check_claude_exists(&claude_dir));

    if claude_dir.exists() {
        // Check git repository
        report.add(check_claude_git_repo(&claude_dir));

        // Check git remote
        report.add(check_claude_remote(&claude_dir));

        // Check git status (uncommitted changes)
        report.add(check_claude_git_status(&claude_dir));

        // Check individual file symlinks
        let dotfiles_claude = dotfiles_dir.join(".claude");
        if dotfiles_claude.exists() {
            report.add(check_claude_individual_symlinks(
                &claude_dir,
                &dotfiles_claude,
            ));
        }
    }

    report
}

fn check_claude_exists(claude_dir: &Path) -> CheckResult {
    if claude_dir.exists() {
        CheckResult::pass("Claude Directory", "~/.claude directory exists")
    } else {
        CheckResult::error(
            "Claude Directory",
            "~/.claude directory not found",
            Some("Clone claudefiles repo: git clone <repo-url> ~/.claude (or run: dotfiles setup)"),
        )
    }
}

fn check_claude_git_repo(claude_dir: &Path) -> CheckResult {
    let git_dir = claude_dir.join(".git");
    if git_dir.exists() {
        CheckResult::pass("Claude Git", "~/.claude is a git repository")
    } else {
        CheckResult::error(
            "Claude Git",
            "~/.claude is not a git repository",
            Some("Initialize: cd ~/.claude && git init"),
        )
    }
}

fn check_claude_remote(claude_dir: &Path) -> CheckResult {
    let output = Command::new("git")
        .arg("-C")
        .arg(claude_dir)
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let remote = String::from_utf8_lossy(&output.stdout).trim().to_string();
            CheckResult::pass(
                "Claude Remote",
                format!("Git remote configured: {}", remote),
            )
        }
        _ => CheckResult::warn(
            "Claude Remote",
            "No git remote configured",
            Some("Add remote: git -C ~/.claude remote add origin <url>"),
        ),
    }
}

fn check_claude_git_status(claude_dir: &Path) -> CheckResult {
    let output = Command::new("git")
        .arg("-C")
        .arg(claude_dir)
        .arg("status")
        .arg("--porcelain")
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let status = String::from_utf8_lossy(&output.stdout);
            if status.trim().is_empty() {
                CheckResult::pass("Claude Git Status", "No uncommitted changes")
            } else {
                CheckResult::warn(
                    "Claude Git Status",
                    "Uncommitted changes in ~/.claude",
                    Some("Review and commit: cd ~/.claude && git status"),
                )
            }
        }
        _ => CheckResult::pass("Claude Git Status", "Unable to check git status"),
    }
}

fn check_claude_individual_symlinks(claude_dir: &Path, dotfiles_claude_dir: &Path) -> CheckResult {
    // Check for individual file symlinks like CLAUDE.md, settings.json
    let expected_symlinks = vec!["CLAUDE.md", "settings.json"];
    let mut missing = Vec::new();

    for file in expected_symlinks {
        let target = claude_dir.join(file);
        let source = dotfiles_claude_dir.join(file);

        if source.exists() && !target.exists() {
            missing.push(file);
        }
    }

    if missing.is_empty() {
        CheckResult::pass("Claude Symlinks", "Individual file symlinks configured")
    } else {
        CheckResult::warn(
            "Claude Symlinks",
            format!("Missing symlinks: {}", missing.join(", ")),
            Some("Run: dotfiles setup to create symlinks"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_check_claude_exists_when_directory_exists() {
        let temp = TempDir::new().unwrap();
        let result = check_claude_exists(temp.path());
        assert!(result.is_pass());
    }

    #[test]
    fn test_check_claude_exists_when_directory_missing() {
        let path = Path::new("/nonexistent/path/.claude");
        let result = check_claude_exists(path);
        assert!(result.is_error());
    }

    #[test]
    fn test_check_claude_git_repo_when_git_exists() {
        let temp = TempDir::new().unwrap();
        let git_dir = temp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        let result = check_claude_git_repo(temp.path());
        assert!(result.is_pass());
    }

    #[test]
    fn test_check_claude_git_repo_when_git_missing() {
        let temp = TempDir::new().unwrap();
        let result = check_claude_git_repo(temp.path());
        assert!(result.is_error());
    }

    #[test]
    fn test_check_claude_individual_symlinks_all_present() {
        let temp = TempDir::new().unwrap();
        let claude_dir = temp.path().join("claude");
        let dotfiles_claude = temp.path().join("dotfiles_claude");

        fs::create_dir(&claude_dir).unwrap();
        fs::create_dir(&dotfiles_claude).unwrap();

        // Create source files
        fs::write(dotfiles_claude.join("CLAUDE.md"), "test").unwrap();
        fs::write(dotfiles_claude.join("settings.json"), "{}").unwrap();

        // Create symlinks
        fs::write(claude_dir.join("CLAUDE.md"), "test").unwrap();
        fs::write(claude_dir.join("settings.json"), "{}").unwrap();

        let result = check_claude_individual_symlinks(&claude_dir, &dotfiles_claude);
        assert!(result.is_pass());
    }

    #[test]
    fn test_check_claude_individual_symlinks_missing() {
        let temp = TempDir::new().unwrap();
        let claude_dir = temp.path().join("claude");
        let dotfiles_claude = temp.path().join("dotfiles_claude");

        fs::create_dir(&claude_dir).unwrap();
        fs::create_dir(&dotfiles_claude).unwrap();

        // Create source files but not targets
        fs::write(dotfiles_claude.join("CLAUDE.md"), "test").unwrap();

        let result = check_claude_individual_symlinks(&claude_dir, &dotfiles_claude);
        assert!(result.is_warn());
        assert!(result.message().contains("CLAUDE.md"));
    }
}
