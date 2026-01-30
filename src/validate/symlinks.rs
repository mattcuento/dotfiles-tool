use crate::validate::{CheckReport, CheckResult};
use std::path::Path;

/// Validates symlinks in a directory
pub fn validate_symlinks(source: &Path, target: &Path) -> CheckReport {
    let mut report = CheckReport::new();

    // Use the existing validation function from symlink module
    match crate::symlink::validate_symlinks(source, target) {
        Ok(issues) => {
            if issues.is_empty() {
                report.add(CheckResult::pass(
                    "Symlinks",
                    format!("All symlinks from {:?} to {:?} are valid", source, target),
                ));
            } else {
                for (path, issue) in issues {
                    report.add(CheckResult::error(
                        format!("Symlink:{}", path.file_name().unwrap().to_string_lossy()),
                        issue.clone(),
                        Some(format!("Fix symlink at {:?}", path)),
                    ));
                }
            }
        }
        Err(e) => {
            report.add(CheckResult::error(
                "Symlinks",
                format!("Failed to validate symlinks: {}", e),
                None::<String>,
            ));
        }
    }

    report
}

/// Checks if a specific symlink points to the correct location
pub fn check_symlink(target: &Path, expected_source: &Path) -> CheckResult {
    if !target.exists() {
        return CheckResult::error(
            format!(
                "Symlink:{}",
                target.file_name().unwrap_or_default().to_string_lossy()
            ),
            "Symlink does not exist",
            Some(format!(
                "Create symlink: ln -s {:?} {:?}",
                expected_source, target
            )),
        );
    }

    if !target.is_symlink() {
        return CheckResult::error(
            format!(
                "Symlink:{}",
                target.file_name().unwrap_or_default().to_string_lossy()
            ),
            "Path exists but is not a symlink",
            Some(format!(
                "Remove file and create symlink: rm {:?} && ln -s {:?} {:?}",
                target, expected_source, target
            )),
        );
    }

    match std::fs::read_link(target) {
        Ok(actual_source) => {
            if actual_source == expected_source {
                CheckResult::pass(
                    format!(
                        "Symlink:{}",
                        target.file_name().unwrap_or_default().to_string_lossy()
                    ),
                    format!("Points to {:?}", actual_source),
                )
            } else {
                CheckResult::error(
                    format!(
                        "Symlink:{}",
                        target.file_name().unwrap_or_default().to_string_lossy()
                    ),
                    format!(
                        "Points to {:?} instead of {:?}",
                        actual_source, expected_source
                    ),
                    Some(format!(
                        "Fix symlink: ln -sf {:?} {:?}",
                        expected_source, target
                    )),
                )
            }
        }
        Err(e) => CheckResult::error(
            format!(
                "Symlink:{}",
                target.file_name().unwrap_or_default().to_string_lossy()
            ),
            format!("Failed to read symlink: {}", e),
            None::<String>,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_symlinks_nonexistent_source() {
        let report = validate_symlinks(Path::new("/nonexistent/source"), Path::new("/target"));

        // Should have one error for nonexistent source
        assert!(report.has_errors());
        assert!(report
            .checks
            .iter()
            .any(|c| c.message().contains("does not exist")));
    }

    #[test]
    fn test_check_symlink_nonexistent() {
        let result = check_symlink(Path::new("/nonexistent/target"), Path::new("/some/source"));

        assert!(result.is_error());
        assert!(result.message().contains("does not exist"));
        assert!(result.suggestion().is_some());
    }

    #[test]
    #[cfg(unix)]
    fn test_check_symlink_valid() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let target = temp_dir.path().join("target.txt");

        // Create source and symlink
        fs::write(&source, "test").unwrap();
        std::os::unix::fs::symlink(&source, &target).unwrap();

        let result = check_symlink(&target, &source);
        assert!(result.is_pass());
    }

    #[test]
    #[cfg(unix)]
    fn test_check_symlink_wrong_target() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let source1 = temp_dir.path().join("source1.txt");
        let source2 = temp_dir.path().join("source2.txt");
        let target = temp_dir.path().join("target.txt");

        // Create sources and symlink to wrong one
        fs::write(&source1, "test1").unwrap();
        fs::write(&source2, "test2").unwrap();
        std::os::unix::fs::symlink(&source1, &target).unwrap();

        let result = check_symlink(&target, &source2);
        assert!(result.is_error());
        assert!(result.message().contains("instead of"));
    }

    #[test]
    #[cfg(unix)]
    fn test_check_symlink_not_a_symlink() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let target = temp_dir.path().join("target.txt");

        // Create both as regular files
        fs::write(&source, "source").unwrap();
        fs::write(&target, "target").unwrap();

        let result = check_symlink(&target, &source);
        assert!(result.is_error());
        assert!(result.message().contains("not a symlink"));
    }
}
