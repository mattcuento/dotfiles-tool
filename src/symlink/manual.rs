use crate::error::{DotfilesError, Result};
use crate::symlink::{SymlinkReport, SymlinkStatus, Symlinker};
use std::path::{Path, PathBuf};

/// Manual symlink creator (fallback when GNU Stow is not available)
pub struct ManualSymlinker {
    /// Whether to run in dry-run mode (no actual changes)
    pub dry_run: bool,
    /// Whether to force overwrite existing symlinks
    pub force: bool,
}

impl ManualSymlinker {
    /// Creates a new ManualSymlinker with default settings
    pub fn new() -> Self {
        Self {
            dry_run: false,
            force: false,
        }
    }

    /// Creates a new ManualSymlinker with dry-run mode enabled
    pub fn dry_run() -> Self {
        Self {
            dry_run: true,
            force: false,
        }
    }

    /// Creates a symlink from source to target
    fn create_symlink(&self, source: &Path, target: &Path) -> Result<SymlinkStatus> {
        // Check if target already exists
        if target.exists() {
            if target.is_symlink() {
                // Check if it points to the right place
                if let Ok(link_target) = std::fs::read_link(target) {
                    if link_target == source {
                        return Ok(SymlinkStatus::AlreadyExists {
                            target: target.to_path_buf(),
                        });
                    }
                }

                // Symlink exists but points elsewhere
                if self.force {
                    if !self.dry_run {
                        std::fs::remove_file(target)?;
                    }
                } else {
                    return Ok(SymlinkStatus::Conflict {
                        target: target.to_path_buf(),
                        reason: format!(
                            "Symlink exists and points to {:?}",
                            std::fs::read_link(target).unwrap()
                        ),
                    });
                }
            } else {
                // File or directory exists
                return Ok(SymlinkStatus::Conflict {
                    target: target.to_path_buf(),
                    reason: if target.is_dir() {
                        "Directory exists".to_string()
                    } else {
                        "File exists".to_string()
                    },
                });
            }
        }

        // Create parent directory if needed
        if let Some(parent) = target.parent() {
            if !parent.exists() && !self.dry_run {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Create the symlink
        if !self.dry_run {
            #[cfg(unix)]
            std::os::unix::fs::symlink(source, target)?;

            #[cfg(not(unix))]
            return Err(DotfilesError::SymlinkFailed(
                "Manual symlinks only supported on Unix systems".to_string(),
            ));
        }

        Ok(SymlinkStatus::Created {
            source: source.to_path_buf(),
            target: target.to_path_buf(),
        })
    }

    /// Removes a symlink if it exists
    fn remove_symlink(&self, target: &Path) -> Result<SymlinkStatus> {
        if !target.exists() {
            return Ok(SymlinkStatus::Skipped {
                target: target.to_path_buf(),
                reason: "Symlink does not exist".to_string(),
            });
        }

        if !target.is_symlink() {
            return Ok(SymlinkStatus::Conflict {
                target: target.to_path_buf(),
                reason: "Not a symlink, will not remove".to_string(),
            });
        }

        if !self.dry_run {
            std::fs::remove_file(target)?;
        }

        Ok(SymlinkStatus::Created {
            source: PathBuf::new(),
            target: target.to_path_buf(),
        })
    }
}

impl Default for ManualSymlinker {
    fn default() -> Self {
        Self::new()
    }
}

impl Symlinker for ManualSymlinker {
    fn symlink(&self, source: &Path, target: &Path) -> Result<SymlinkReport> {
        let mut report = SymlinkReport::new();

        if !source.exists() {
            return Err(DotfilesError::SymlinkFailed(format!(
                "Source directory does not exist: {:?}",
                source
            )));
        }

        // Walk through source directory
        if source.is_dir() {
            let entries = std::fs::read_dir(source)?;

            for entry in entries {
                let entry = entry?;
                let source_path = entry.path();
                let file_name = source_path
                    .file_name()
                    .ok_or_else(|| DotfilesError::SymlinkFailed("Invalid filename".to_string()))?;
                let target_path = target.join(file_name);

                let status = self.create_symlink(&source_path, &target_path)?;
                report.add(status);
            }
        } else {
            // Source is a file, create a single symlink
            let file_name = source
                .file_name()
                .ok_or_else(|| DotfilesError::SymlinkFailed("Invalid filename".to_string()))?;
            let target_path = target.join(file_name);

            let status = self.create_symlink(source, &target_path)?;
            report.add(status);
        }

        Ok(report)
    }

    fn is_available(&self) -> bool {
        // Manual symlinking is always available on Unix systems
        cfg!(unix)
    }

    fn name(&self) -> &str {
        "Manual Symlinks"
    }

    fn remove(&self, source: &Path, target: &Path) -> Result<SymlinkReport> {
        let mut report = SymlinkReport::new();

        if !source.exists() {
            return Err(DotfilesError::SymlinkFailed(format!(
                "Source directory does not exist: {:?}",
                source
            )));
        }

        // Walk through source directory and remove corresponding symlinks
        if source.is_dir() {
            let entries = std::fs::read_dir(source)?;

            for entry in entries {
                let entry = entry?;
                let source_path = entry.path();
                let file_name = source_path
                    .file_name()
                    .ok_or_else(|| DotfilesError::SymlinkFailed("Invalid filename".to_string()))?;
                let target_path = target.join(file_name);

                let status = self.remove_symlink(&target_path)?;
                report.add(status);
            }
        } else {
            let file_name = source
                .file_name()
                .ok_or_else(|| DotfilesError::SymlinkFailed("Invalid filename".to_string()))?;
            let target_path = target.join(file_name);

            let status = self.remove_symlink(&target_path)?;
            report.add(status);
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_manual_symlinker_new() {
        let manual = ManualSymlinker::new();
        assert!(!manual.dry_run);
        assert!(!manual.force);
    }

    #[test]
    fn test_manual_symlinker_dry_run() {
        let manual = ManualSymlinker::dry_run();
        assert!(manual.dry_run);
    }

    #[test]
    fn test_manual_symlinker_default() {
        let manual = ManualSymlinker::default();
        assert!(!manual.dry_run);
        assert!(!manual.force);
    }

    #[test]
    fn test_manual_symlinker_is_available() {
        let manual = ManualSymlinker::new();
        // On Unix systems, manual symlinking should always be available
        #[cfg(unix)]
        assert!(manual.is_available());

        #[cfg(not(unix))]
        assert!(!manual.is_available());
    }

    #[test]
    fn test_manual_symlinker_name() {
        let manual = ManualSymlinker::new();
        assert_eq!(manual.name(), "Manual Symlinks");
    }

    #[test]
    #[cfg(unix)]
    fn test_create_symlink_new() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        let target_file = temp_dir.path().join("target.txt");

        // Create source file
        fs::write(&source_file, "test content").unwrap();

        let manual = ManualSymlinker::new();
        let status = manual.create_symlink(&source_file, &target_file).unwrap();

        assert!(matches!(status, SymlinkStatus::Created { .. }));
        assert!(target_file.is_symlink());

        let link_target = fs::read_link(&target_file).unwrap();
        assert_eq!(link_target, source_file);
    }

    #[test]
    #[cfg(unix)]
    fn test_create_symlink_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        let target_file = temp_dir.path().join("target.txt");

        // Create source file
        fs::write(&source_file, "test content").unwrap();

        // Create symlink manually
        std::os::unix::fs::symlink(&source_file, &target_file).unwrap();

        let manual = ManualSymlinker::new();
        let status = manual.create_symlink(&source_file, &target_file).unwrap();

        assert!(matches!(status, SymlinkStatus::AlreadyExists { .. }));
    }

    #[test]
    #[cfg(unix)]
    fn test_create_symlink_conflict_file() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        let target_file = temp_dir.path().join("target.txt");

        // Create both files
        fs::write(&source_file, "source content").unwrap();
        fs::write(&target_file, "target content").unwrap();

        let manual = ManualSymlinker::new();
        let status = manual.create_symlink(&source_file, &target_file).unwrap();

        assert!(matches!(status, SymlinkStatus::Conflict { .. }));
    }

    #[test]
    #[cfg(unix)]
    fn test_create_symlink_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        let target_file = temp_dir.path().join("target.txt");

        // Create source file
        fs::write(&source_file, "test content").unwrap();

        let manual = ManualSymlinker::dry_run();
        let status = manual.create_symlink(&source_file, &target_file).unwrap();

        assert!(matches!(status, SymlinkStatus::Created { .. }));
        // In dry-run mode, symlink should not actually be created
        assert!(!target_file.exists());
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_directory() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target");

        // Create source directory with files
        fs::create_dir(&source_dir).unwrap();
        fs::write(source_dir.join("file1.txt"), "content1").unwrap();
        fs::write(source_dir.join("file2.txt"), "content2").unwrap();

        // Create target directory
        fs::create_dir(&target_dir).unwrap();

        let manual = ManualSymlinker::new();
        let report = manual.symlink(&source_dir, &target_dir).unwrap();

        assert_eq!(report.created.len(), 2);
        assert!(target_dir.join("file1.txt").is_symlink());
        assert!(target_dir.join("file2.txt").is_symlink());
    }

    #[test]
    #[cfg(unix)]
    fn test_remove_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        let target_file = temp_dir.path().join("target.txt");

        // Create source file and symlink
        fs::write(&source_file, "test content").unwrap();
        std::os::unix::fs::symlink(&source_file, &target_file).unwrap();

        let manual = ManualSymlinker::new();
        let status = manual.remove_symlink(&target_file).unwrap();

        assert!(matches!(status, SymlinkStatus::Created { .. }));
        assert!(!target_file.exists());
    }
}
