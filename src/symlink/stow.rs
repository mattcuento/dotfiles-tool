use crate::error::{DotfilesError, Result};
use crate::symlink::{SymlinkReport, SymlinkStatus, Symlinker};
use std::path::Path;
use std::process::Command;

/// GNU Stow symlink manager
pub struct StowSymlinker {
    /// Whether to run in dry-run mode (no actual changes)
    pub dry_run: bool,
    /// Whether to show verbose output
    pub verbose: bool,
}

impl StowSymlinker {
    /// Creates a new StowSymlinker with default settings
    pub fn new() -> Self {
        Self {
            dry_run: false,
            verbose: false,
        }
    }

    /// Creates a new StowSymlinker with dry-run mode enabled
    pub fn dry_run() -> Self {
        Self {
            dry_run: true,
            verbose: false,
        }
    }

    /// Gets the path to the stow executable
    fn stow_path(&self) -> Option<std::path::PathBuf> {
        crate::detect::tools::get_tool_path("stow").map(std::path::PathBuf::from)
    }

    /// Runs a stow command with the given arguments
    fn run_stow(&self, args: &[&str]) -> Result<std::process::Output> {
        let stow = self
            .stow_path()
            .ok_or_else(|| DotfilesError::DependencyMissing("GNU Stow".to_string()))?;

        let output = Command::new(stow).args(args).output()?;

        Ok(output)
    }

    /// Parses stow output to determine what happened
    fn parse_stow_output(
        &self,
        source: &Path,
        target: &Path,
        output: &std::process::Output,
    ) -> SymlinkReport {
        let mut report = SymlinkReport::new();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Stow reports conflicts in stderr
            if stderr.contains("existing target") || stderr.contains("conflict") {
                // Parse conflicts from stow output
                for line in stderr.lines() {
                    if line.contains("existing target") || line.contains("conflict") {
                        let target_path = target.to_path_buf();
                        report.add(SymlinkStatus::Conflict {
                            target: target_path,
                            reason: line.to_string(),
                        });
                    }
                }
            } else {
                // Generic error
                report.add(SymlinkStatus::Conflict {
                    target: target.to_path_buf(),
                    reason: stderr.to_string(),
                });
            }
        } else {
            // Success - assume symlinks were created
            // Note: Stow doesn't give us detailed output by default,
            // so we report a generic success
            report.add(SymlinkStatus::Created {
                source: source.to_path_buf(),
                target: target.to_path_buf(),
            });
        }

        report
    }
}

impl Default for StowSymlinker {
    fn default() -> Self {
        Self::new()
    }
}

impl Symlinker for StowSymlinker {
    fn symlink(&self, source: &Path, target: &Path) -> Result<SymlinkReport> {
        if !source.exists() {
            return Err(DotfilesError::SymlinkFailed(format!(
                "Source directory does not exist: {:?}",
                source
            )));
        }

        // Get the package name (last component of source path)
        let package = source
            .file_name()
            .ok_or_else(|| DotfilesError::SymlinkFailed("Invalid source path".to_string()))?
            .to_str()
            .ok_or_else(|| DotfilesError::SymlinkFailed("Invalid UTF-8 in path".to_string()))?;

        // Get the stow directory (parent of source)
        let stow_dir = source.parent().ok_or_else(|| {
            DotfilesError::SymlinkFailed("Source has no parent directory".to_string())
        })?;

        // Build stow command arguments
        let mut args = vec![
            "-d",
            stow_dir.to_str().unwrap(),
            "-t",
            target.to_str().unwrap(),
        ];

        // Add exclusion patterns
        for pattern in crate::symlink::EXCLUSIONS {
            args.push("--ignore");
            args.push(pattern);
        }

        if self.dry_run {
            args.push("-n"); // no-op/dry-run
        }

        if self.verbose {
            args.push("-v"); // verbose
        }

        args.push(package);

        // Run stow command
        let output = self.run_stow(&args)?;

        // Parse output and return report
        Ok(self.parse_stow_output(source, target, &output))
    }

    fn is_available(&self) -> bool {
        crate::detect::tools::is_installed("stow")
    }

    fn name(&self) -> &str {
        "GNU Stow"
    }

    fn remove(&self, source: &Path, target: &Path) -> Result<SymlinkReport> {
        if !source.exists() {
            return Err(DotfilesError::SymlinkFailed(format!(
                "Source directory does not exist: {:?}",
                source
            )));
        }

        let package = source
            .file_name()
            .ok_or_else(|| DotfilesError::SymlinkFailed("Invalid source path".to_string()))?
            .to_str()
            .ok_or_else(|| DotfilesError::SymlinkFailed("Invalid UTF-8 in path".to_string()))?;

        let stow_dir = source.parent().ok_or_else(|| {
            DotfilesError::SymlinkFailed("Source has no parent directory".to_string())
        })?;

        // Build stow command with -D (delete/unstow)
        let mut args = vec![
            "-d",
            stow_dir.to_str().unwrap(),
            "-t",
            target.to_str().unwrap(),
            "-D", // Delete/unstow
        ];

        if self.dry_run {
            args.push("-n");
        }

        if self.verbose {
            args.push("-v");
        }

        args.push(package);

        let output = self.run_stow(&args)?;
        Ok(self.parse_stow_output(source, target, &output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stow_symlinker_new() {
        let stow = StowSymlinker::new();
        assert!(!stow.dry_run);
        assert!(!stow.verbose);
    }

    #[test]
    fn test_stow_symlinker_dry_run() {
        let stow = StowSymlinker::dry_run();
        assert!(stow.dry_run);
    }

    #[test]
    fn test_stow_symlinker_default() {
        let stow = StowSymlinker::default();
        assert!(!stow.dry_run);
        assert!(!stow.verbose);
    }

    #[test]
    fn test_stow_symlinker_is_available() {
        let stow = StowSymlinker::new();
        // This test checks if stow is installed on the system
        // It will pass regardless of the result
        let _ = stow.is_available();
    }

    #[test]
    fn test_stow_symlinker_name() {
        let stow = StowSymlinker::new();
        assert_eq!(stow.name(), "GNU Stow");
    }

    #[test]
    fn test_stow_path() {
        let stow = StowSymlinker::new();
        let path = stow.stow_path();

        // If stow is installed, path should be Some and end with "stow"
        if let Some(p) = path {
            assert!(p.to_str().unwrap().contains("stow"));
        }
    }

    #[test]
    fn test_stow_exclusions_constant() {
        use crate::symlink::EXCLUSIONS;
        assert!(EXCLUSIONS.contains(&".git"));
        assert!(EXCLUSIONS.contains(&".DS_Store"));
        assert!(EXCLUSIONS.contains(&".claude"));
        assert!(EXCLUSIONS.contains(&"README.md"));
        assert!(EXCLUSIONS.contains(&"LICENSE"));
        assert_eq!(EXCLUSIONS.len(), 5);
    }
}
