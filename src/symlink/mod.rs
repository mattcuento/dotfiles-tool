pub mod manual;
pub mod stow;

use crate::error::Result;
use std::path::{Path, PathBuf};

/// Files and directories to exclude from symlinking
///
/// These are commonly non-portable or repository-specific files that
/// should not be symlinked to the home directory.
pub const EXCLUSIONS: &[&str] = &[".git", ".DS_Store", ".claude", "README.md", "LICENSE"];

/// Special directories that need individual file symlinks instead of directory symlinks
///
/// These directories contain both config files (that should be in version control and symlinked)
/// and runtime data (that should not be in version control)
pub const INDIVIDUAL_FILE_SYMLINK_DIRS: &[&str] = &[".claude"];

/// Result of a symlink operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymlinkStatus {
    /// Symlink was created successfully
    Created { source: PathBuf, target: PathBuf },
    /// Symlink already exists and points to correct location
    AlreadyExists { target: PathBuf },
    /// Conflict detected (file/dir exists at target location)
    Conflict { target: PathBuf, reason: String },
    /// Operation was skipped (e.g., dry-run mode)
    Skipped { target: PathBuf, reason: String },
}

impl SymlinkStatus {
    /// Returns true if the symlink operation was successful
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            SymlinkStatus::Created { .. } | SymlinkStatus::AlreadyExists { .. }
        )
    }

    /// Returns true if there was a conflict
    pub fn is_conflict(&self) -> bool {
        matches!(self, SymlinkStatus::Conflict { .. })
    }

    /// Returns the target path for this status
    pub fn target(&self) -> &Path {
        match self {
            SymlinkStatus::Created { target, .. } => target,
            SymlinkStatus::AlreadyExists { target } => target,
            SymlinkStatus::Conflict { target, .. } => target,
            SymlinkStatus::Skipped { target, .. } => target,
        }
    }
}

/// Report summarizing symlink operations
#[derive(Debug, Clone, Default)]
pub struct SymlinkReport {
    pub created: Vec<PathBuf>,
    pub already_exists: Vec<PathBuf>,
    pub conflicts: Vec<(PathBuf, String)>,
    pub skipped: Vec<(PathBuf, String)>,
}

impl SymlinkReport {
    /// Creates a new empty report
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a status to the report
    pub fn add(&mut self, status: SymlinkStatus) {
        match status {
            SymlinkStatus::Created { target, .. } => {
                self.created.push(target);
            }
            SymlinkStatus::AlreadyExists { target } => {
                self.already_exists.push(target);
            }
            SymlinkStatus::Conflict { target, reason } => {
                self.conflicts.push((target, reason));
            }
            SymlinkStatus::Skipped { target, reason } => {
                self.skipped.push((target, reason));
            }
        }
    }

    /// Returns true if all operations were successful
    pub fn is_success(&self) -> bool {
        self.conflicts.is_empty()
    }

    /// Returns the total number of operations
    pub fn total(&self) -> usize {
        self.created.len() + self.already_exists.len() + self.conflicts.len() + self.skipped.len()
    }

    /// Returns a summary string
    pub fn summary(&self) -> String {
        format!(
            "Created: {}, Already exists: {}, Conflicts: {}, Skipped: {}",
            self.created.len(),
            self.already_exists.len(),
            self.conflicts.len(),
            self.skipped.len()
        )
    }
}

/// Common interface for symlink creation methods
pub trait Symlinker {
    /// Creates symlinks from source directory to target directory
    fn symlink(&self, source: &Path, target: &Path) -> Result<SymlinkReport>;

    /// Checks if this symlinker is available on the system
    fn is_available(&self) -> bool;

    /// Returns the name of this symlinker
    fn name(&self) -> &str;

    /// Removes symlinks (if supported)
    fn remove(&self, _source: &Path, _target: &Path) -> Result<SymlinkReport> {
        // Default implementation: not supported
        Err(crate::error::DotfilesError::SymlinkFailed(format!(
            "{} does not support removal",
            self.name()
        )))
    }
}

/// Detects conflicts before creating symlinks
pub fn detect_conflicts(source: &Path, target: &Path) -> Vec<(PathBuf, String)> {
    let mut conflicts = Vec::new();

    if !source.exists() {
        return vec![(
            source.to_path_buf(),
            "Source directory does not exist".to_string(),
        )];
    }

    // Walk through source directory and check for conflicts
    if let Ok(entries) = std::fs::read_dir(source) {
        for entry in entries.flatten() {
            let source_path = entry.path();
            let file_name = source_path.file_name().unwrap();
            let target_path = target.join(file_name);

            if target_path.exists() {
                // Check if it's already a symlink pointing to the right place
                if target_path.is_symlink() {
                    if let Ok(link_target) = std::fs::read_link(&target_path) {
                        if link_target == source_path {
                            // Already correctly symlinked, no conflict
                            continue;
                        }
                    }
                    conflicts.push((
                        target_path,
                        "Symlink exists but points to wrong location".to_string(),
                    ));
                } else if target_path.is_dir() {
                    conflicts.push((target_path, "Directory already exists".to_string()));
                } else {
                    conflicts.push((target_path, "File already exists".to_string()));
                }
            }
        }
    }

    conflicts
}

/// Validates that symlinks point to the correct locations
pub fn validate_symlinks(source: &Path, target: &Path) -> Result<Vec<(PathBuf, String)>> {
    let mut issues = Vec::new();

    if !source.exists() {
        return Ok(vec![(
            source.to_path_buf(),
            "Source directory does not exist".to_string(),
        )]);
    }

    if let Ok(entries) = std::fs::read_dir(source) {
        for entry in entries.flatten() {
            let source_path = entry.path();
            let file_name = source_path.file_name().unwrap();
            let target_path = target.join(file_name);

            if !target_path.exists() {
                issues.push((target_path.clone(), "Symlink does not exist".to_string()));
            } else if !target_path.is_symlink() {
                issues.push((target_path.clone(), "Not a symlink".to_string()));
            } else if let Ok(link_target) = std::fs::read_link(&target_path) {
                if link_target != source_path {
                    issues.push((
                        target_path.clone(),
                        format!("Points to {:?} instead of {:?}", link_target, source_path),
                    ));
                }
            } else {
                issues.push((target_path.clone(), "Failed to read symlink".to_string()));
            }
        }
    }

    Ok(issues)
}

/// Symlinks individual files from special directories that need file-level symlinks
///
/// This is used for directories like .claude where config files should be symlinked
/// but runtime data should remain as regular files.
pub fn symlink_individual_files(
    symlinker: &dyn Symlinker,
    dotfiles_dir: &Path,
    home_dir: &Path,
) -> Result<SymlinkReport> {
    let mut report = SymlinkReport::new();

    for special_dir in INDIVIDUAL_FILE_SYMLINK_DIRS {
        let source_special = dotfiles_dir.join(special_dir);
        let target_special = home_dir.join(special_dir);

        // Skip if source directory doesn't exist
        if !source_special.exists() {
            continue;
        }

        // Ensure target directory exists
        if !target_special.exists() {
            std::fs::create_dir_all(&target_special)?;
        }

        // Symlink individual files from the special directory
        let special_report = symlinker.symlink(&source_special, &target_special)?;

        // Merge reports
        for path in special_report.created {
            report.created.push(path);
        }
        for path in special_report.already_exists {
            report.already_exists.push(path);
        }
        for (path, reason) in special_report.conflicts {
            report.conflicts.push((path, reason));
        }
        for (path, reason) in special_report.skipped {
            report.skipped.push((path, reason));
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symlink_status_is_success() {
        let created = SymlinkStatus::Created {
            source: PathBuf::from("/src"),
            target: PathBuf::from("/target"),
        };
        assert!(created.is_success());

        let exists = SymlinkStatus::AlreadyExists {
            target: PathBuf::from("/target"),
        };
        assert!(exists.is_success());

        let conflict = SymlinkStatus::Conflict {
            target: PathBuf::from("/target"),
            reason: "exists".to_string(),
        };
        assert!(!conflict.is_success());
    }

    #[test]
    fn test_symlink_status_is_conflict() {
        let conflict = SymlinkStatus::Conflict {
            target: PathBuf::from("/target"),
            reason: "exists".to_string(),
        };
        assert!(conflict.is_conflict());

        let created = SymlinkStatus::Created {
            source: PathBuf::from("/src"),
            target: PathBuf::from("/target"),
        };
        assert!(!created.is_conflict());
    }

    #[test]
    fn test_symlink_status_target() {
        let created = SymlinkStatus::Created {
            source: PathBuf::from("/src"),
            target: PathBuf::from("/target"),
        };
        assert_eq!(created.target(), Path::new("/target"));
    }

    #[test]
    fn test_symlink_report_new() {
        let report = SymlinkReport::new();
        assert_eq!(report.total(), 0);
        assert!(report.is_success());
    }

    #[test]
    fn test_symlink_report_add() {
        let mut report = SymlinkReport::new();

        report.add(SymlinkStatus::Created {
            source: PathBuf::from("/src/file1"),
            target: PathBuf::from("/target/file1"),
        });
        assert_eq!(report.created.len(), 1);
        assert_eq!(report.total(), 1);

        report.add(SymlinkStatus::Conflict {
            target: PathBuf::from("/target/file2"),
            reason: "exists".to_string(),
        });
        assert_eq!(report.conflicts.len(), 1);
        assert_eq!(report.total(), 2);
        assert!(!report.is_success());
    }

    #[test]
    fn test_symlink_report_summary() {
        let mut report = SymlinkReport::new();
        report.add(SymlinkStatus::Created {
            source: PathBuf::from("/src/file1"),
            target: PathBuf::from("/target/file1"),
        });
        report.add(SymlinkStatus::AlreadyExists {
            target: PathBuf::from("/target/file2"),
        });

        let summary = report.summary();
        assert!(summary.contains("Created: 1"));
        assert!(summary.contains("Already exists: 1"));
    }

    #[test]
    fn test_detect_conflicts_nonexistent_source() {
        let conflicts = detect_conflicts(Path::new("/nonexistent/source"), Path::new("/target"));
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].1.contains("does not exist"));
    }

    #[test]
    fn test_validate_symlinks_nonexistent_source() {
        let issues =
            validate_symlinks(Path::new("/nonexistent/source"), Path::new("/target")).unwrap();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].1.contains("does not exist"));
    }
}
