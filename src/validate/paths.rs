use crate::validate::{CheckReport, CheckResult};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Patterns to detect hardcoded paths
pub struct PathPatterns {
    pub home_path: Regex,
    pub users_path: Regex,
    pub absolute_path: Regex,
}

impl PathPatterns {
    /// Creates default path patterns
    pub fn new() -> Self {
        Self {
            // Matches /Users/username or /home/username
            home_path: Regex::new(r"/(?:Users|home)/[a-zA-Z0-9_-]+").unwrap(),
            // Matches /Users specifically
            users_path: Regex::new(r"/Users/[a-zA-Z0-9_-]+").unwrap(),
            // Matches absolute paths (starting with /)
            absolute_path: Regex::new(r"^/[a-zA-Z0-9_/-]+").unwrap(),
        }
    }
}

impl Default for PathPatterns {
    fn default() -> Self {
        Self::new()
    }
}

/// Scans a file for hardcoded paths
pub fn scan_file(file_path: &Path) -> CheckResult {
    let patterns = PathPatterns::new();

    match fs::read_to_string(file_path) {
        Ok(content) => {
            let mut issues = Vec::new();

            for (line_num, line) in content.lines().enumerate() {
                // Skip comments
                if line.trim_start().starts_with('#') {
                    continue;
                }

                // Check for hardcoded home paths
                if patterns.home_path.is_match(line) {
                    issues.push(format!("Line {}: Found hardcoded home path", line_num + 1));
                }
            }

            if issues.is_empty() {
                CheckResult::pass(
                    format!("Paths:{}", file_path.file_name().unwrap().to_string_lossy()),
                    "No hardcoded paths found",
                )
            } else {
                CheckResult::warn(
                    format!("Paths:{}", file_path.file_name().unwrap().to_string_lossy()),
                    format!("Found {} hardcoded path(s)", issues.len()),
                    Some("Use $HOME or ~ instead of absolute paths"),
                )
            }
        }
        Err(e) => CheckResult::error(
            format!(
                "Paths:{}",
                file_path.file_name().unwrap_or_default().to_string_lossy()
            ),
            format!("Failed to read file: {}", e),
            None::<String>,
        ),
    }
}

/// Scans a directory for hardcoded paths in config files
pub fn scan_directory(dir_path: &Path) -> CheckReport {
    let mut report = CheckReport::new();

    if !dir_path.exists() {
        report.add(CheckResult::error(
            "Paths",
            format!("Directory does not exist: {:?}", dir_path),
            None::<String>,
        ));
        return report;
    }

    // Common config file extensions
    let config_extensions = vec![
        "sh", "bash", "zsh", "fish", "rc", "conf", "config", "toml", "yaml", "yml",
    ];

    match fs::read_dir(dir_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if config_extensions.contains(&ext.to_str().unwrap_or("")) {
                            report.add(scan_file(&path));
                        }
                    } else if path.file_name().is_some() {
                        // Check for dotfiles without extension
                        let name = path.file_name().unwrap().to_str().unwrap_or("");
                        if name.starts_with('.') {
                            report.add(scan_file(&path));
                        }
                    }
                }
            }

            if report.total() == 0 {
                report.add(CheckResult::pass(
                    "Paths",
                    format!("No config files found in {:?}", dir_path),
                ));
            }
        }
        Err(e) => {
            report.add(CheckResult::error(
                "Paths",
                format!("Failed to read directory: {}", e),
                None::<String>,
            ));
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_path_patterns_home_path() {
        let patterns = PathPatterns::new();

        assert!(patterns.home_path.is_match("/Users/john"));
        assert!(patterns.home_path.is_match("/home/jane"));
        assert!(!patterns.home_path.is_match("/etc/config"));
    }

    #[test]
    fn test_path_patterns_users_path() {
        let patterns = PathPatterns::new();

        assert!(patterns.users_path.is_match("/Users/john"));
        assert!(!patterns.users_path.is_match("/home/jane"));
    }

    #[test]
    fn test_scan_file_clean() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.sh");

        fs::write(&file_path, "echo $HOME\nexport PATH=$PATH:$HOME/bin\n").unwrap();

        let result = scan_file(&file_path);
        assert!(result.is_pass());
    }

    #[test]
    fn test_scan_file_with_hardcoded_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.sh");

        fs::write(&file_path, "export PATH=/Users/john/bin:$PATH\n").unwrap();

        let result = scan_file(&file_path);
        assert!(result.is_warn());
        assert!(result.message().contains("hardcoded path"));
    }

    #[test]
    fn test_scan_file_with_comment() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.sh");

        // Hardcoded path in comment should be ignored
        fs::write(
            &file_path,
            "# This is a comment with /Users/john\necho $HOME\n",
        )
        .unwrap();

        let result = scan_file(&file_path);
        assert!(result.is_pass());
    }

    #[test]
    fn test_scan_file_nonexistent() {
        let result = scan_file(Path::new("/nonexistent/file.sh"));
        assert!(result.is_error());
        assert!(result.message().contains("Failed to read"));
    }

    #[test]
    fn test_scan_directory_empty() {
        let temp_dir = TempDir::new().unwrap();
        let report = scan_directory(temp_dir.path());

        // Empty directory should have one pass result
        assert_eq!(report.total(), 1);
        assert!(report.is_clean());
    }

    #[test]
    fn test_scan_directory_with_config() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test.sh");
        let file2 = temp_dir.path().join(".zshrc");

        fs::write(&file1, "echo $HOME\n").unwrap();
        fs::write(&file2, "export PATH=/Users/john/bin:$PATH\n").unwrap();

        let report = scan_directory(temp_dir.path());

        // Should have 2 results (one for each file)
        assert_eq!(report.total(), 2);
        assert!(report.warn_count() > 0); // file2 should trigger warning
    }

    #[test]
    fn test_scan_directory_nonexistent() {
        let report = scan_directory(Path::new("/nonexistent/directory"));

        assert!(report.has_errors());
        assert!(report
            .checks
            .iter()
            .any(|c| c.message().contains("does not exist")));
    }
}
