pub mod migrate;
pub mod secrets;

use crate::error::{DotfilesError, Result};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

/// Backup metadata
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub timestamp: String,
    pub source: PathBuf,
}

impl BackupInfo {
    /// Parses a backup directory name to extract timestamp
    pub fn from_path(path: PathBuf, source: PathBuf) -> Option<Self> {
        let dir_name = path.file_name()?.to_str()?;

        if dir_name.starts_with(".dotfiles-backup-") {
            let timestamp = dir_name.strip_prefix(".dotfiles-backup-")?.to_string();
            Some(Self {
                path,
                timestamp,
                source,
            })
        } else {
            None
        }
    }
}

/// Creates a timestamped backup of a directory
pub fn create_backup(source: &Path, backup_dir: Option<&Path>) -> Result<PathBuf> {
    if !source.exists() {
        return Err(DotfilesError::Config(format!(
            "Source directory does not exist: {:?}",
            source
        )));
    }

    // Generate timestamp
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();

    // Determine backup location
    let backup_parent = if let Some(dir) = backup_dir {
        dir.to_path_buf()
    } else {
        dirs::home_dir().ok_or_else(|| {
            DotfilesError::Config("Could not determine home directory".to_string())
        })?
    };

    let backup_name = format!(".dotfiles-backup-{}", timestamp);
    let backup_path = backup_parent.join(backup_name);

    // Create backup directory
    fs::create_dir_all(&backup_path)?;

    // Copy contents
    copy_dir_recursive(source, &backup_path)?;

    println!("✓ Created backup at {:?}", backup_path);

    Ok(backup_path)
}

/// Copies a directory recursively
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Lists all backups in a directory
pub fn list_backups(backup_dir: Option<&Path>) -> Result<Vec<BackupInfo>> {
    let search_dir = if let Some(dir) = backup_dir {
        dir.to_path_buf()
    } else {
        dirs::home_dir().ok_or_else(|| {
            DotfilesError::Config("Could not determine home directory".to_string())
        })?
    };

    if !search_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();

    for entry in fs::read_dir(search_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(backup) = BackupInfo::from_path(path, PathBuf::new()) {
                backups.push(backup);
            }
        }
    }

    // Sort by timestamp (newest first)
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(backups)
}

/// Gets the most recent backup
pub fn get_latest_backup(backup_dir: Option<&Path>) -> Result<Option<BackupInfo>> {
    let backups = list_backups(backup_dir)?;
    Ok(backups.into_iter().next())
}

/// Restores from a backup
pub fn restore_backup(backup: &BackupInfo, target: &Path) -> Result<()> {
    if !backup.path.exists() {
        return Err(DotfilesError::Config(format!(
            "Backup does not exist: {:?}",
            backup.path
        )));
    }

    if target.exists() {
        // Create a backup of the current state before restoring
        create_backup(target, None)?;
    }

    // Clear target directory
    if target.exists() {
        fs::remove_dir_all(target)?;
    }

    // Restore from backup
    copy_dir_recursive(&backup.path, target)?;

    println!("✓ Restored from backup: {}", backup.timestamp);

    Ok(())
}

/// Verifies that a backup is valid
pub fn verify_backup(backup_path: &Path) -> Result<bool> {
    if !backup_path.exists() {
        return Ok(false);
    }

    if !backup_path.is_dir() {
        return Ok(false);
    }

    // Check if backup has any contents
    let entries = fs::read_dir(backup_path)?;
    let has_contents = entries.count() > 0;

    Ok(has_contents)
}

/// Deletes old backups, keeping only the N most recent
pub fn cleanup_old_backups(keep: usize, backup_dir: Option<&Path>) -> Result<Vec<PathBuf>> {
    let backups = list_backups(backup_dir)?;
    let mut deleted = Vec::new();

    for backup in backups.iter().skip(keep) {
        if backup.path.exists() {
            fs::remove_dir_all(&backup.path)?;
            deleted.push(backup.path.clone());
            println!("✓ Deleted old backup: {}", backup.timestamp);
        }
    }

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_backup_info_from_path() {
        let path = PathBuf::from("/home/user/.dotfiles-backup-20260129-143022");
        let backup = BackupInfo::from_path(path.clone(), PathBuf::new());

        assert!(backup.is_some());
        let backup = backup.unwrap();
        assert_eq!(backup.timestamp, "20260129-143022");
        assert_eq!(backup.path, path);
    }

    #[test]
    fn test_backup_info_from_invalid_path() {
        let path = PathBuf::from("/home/user/not-a-backup");
        let backup = BackupInfo::from_path(path, PathBuf::new());

        assert!(backup.is_none());
    }

    #[test]
    fn test_create_backup() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let backup_parent = temp_dir.path().join("backups");

        // Create source directory with files
        fs::create_dir(&source_dir).unwrap();
        fs::write(source_dir.join("file1.txt"), "content1").unwrap();
        fs::write(source_dir.join("file2.txt"), "content2").unwrap();

        fs::create_dir(&backup_parent).unwrap();

        // Create backup
        let backup_path = create_backup(&source_dir, Some(&backup_parent)).unwrap();

        // Verify backup exists
        assert!(backup_path.exists());
        assert!(backup_path.is_dir());

        // Verify contents
        assert!(backup_path.join("file1.txt").exists());
        assert!(backup_path.join("file2.txt").exists());

        let content1 = fs::read_to_string(backup_path.join("file1.txt")).unwrap();
        assert_eq!(content1, "content1");
    }

    #[test]
    fn test_create_backup_nonexistent_source() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("nonexistent");
        let backup_parent = temp_dir.path();

        let result = create_backup(&source_dir, Some(backup_parent));
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_dir_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");

        // Create nested structure
        fs::create_dir(&source_dir).unwrap();
        fs::write(source_dir.join("file1.txt"), "content1").unwrap();

        let subdir = source_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("file2.txt"), "content2").unwrap();

        // Copy
        copy_dir_recursive(&source_dir, &dest_dir).unwrap();

        // Verify
        assert!(dest_dir.join("file1.txt").exists());
        assert!(dest_dir.join("subdir").exists());
        assert!(dest_dir.join("subdir/file2.txt").exists());
    }

    #[test]
    fn test_list_backups() {
        let temp_dir = TempDir::new().unwrap();

        // Create some backup directories
        fs::create_dir(temp_dir.path().join(".dotfiles-backup-20260129-120000")).unwrap();
        fs::create_dir(temp_dir.path().join(".dotfiles-backup-20260129-130000")).unwrap();
        fs::create_dir(temp_dir.path().join("not-a-backup")).unwrap();

        let backups = list_backups(Some(temp_dir.path())).unwrap();

        assert_eq!(backups.len(), 2);
        // Should be sorted newest first
        assert_eq!(backups[0].timestamp, "20260129-130000");
        assert_eq!(backups[1].timestamp, "20260129-120000");
    }

    #[test]
    fn test_get_latest_backup() {
        let temp_dir = TempDir::new().unwrap();

        // Create backups
        fs::create_dir(temp_dir.path().join(".dotfiles-backup-20260129-120000")).unwrap();
        fs::create_dir(temp_dir.path().join(".dotfiles-backup-20260129-130000")).unwrap();

        let latest = get_latest_backup(Some(temp_dir.path())).unwrap();

        assert!(latest.is_some());
        assert_eq!(latest.unwrap().timestamp, "20260129-130000");
    }

    #[test]
    fn test_get_latest_backup_none() {
        let temp_dir = TempDir::new().unwrap();

        let latest = get_latest_backup(Some(temp_dir.path())).unwrap();
        assert!(latest.is_none());
    }

    #[test]
    fn test_verify_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backup");

        // Empty directory - invalid
        fs::create_dir(&backup_dir).unwrap();
        assert!(!verify_backup(&backup_dir).unwrap());

        // Directory with contents - valid
        fs::write(backup_dir.join("file.txt"), "content").unwrap();
        assert!(verify_backup(&backup_dir).unwrap());

        // Nonexistent - invalid
        let nonexistent = temp_dir.path().join("nonexistent");
        assert!(!verify_backup(&nonexistent).unwrap());
    }

    #[test]
    fn test_cleanup_old_backups() {
        let temp_dir = TempDir::new().unwrap();

        // Create 5 backups
        for i in 1..=5 {
            let name = format!(".dotfiles-backup-2026012{}-120000", i);
            fs::create_dir(temp_dir.path().join(&name)).unwrap();
        }

        // Keep only 2 most recent
        let deleted = cleanup_old_backups(2, Some(temp_dir.path())).unwrap();

        assert_eq!(deleted.len(), 3);

        // Verify only 2 remain
        let remaining = list_backups(Some(temp_dir.path())).unwrap();
        assert_eq!(remaining.len(), 2);
        assert_eq!(remaining[0].timestamp, "20260125-120000");
        assert_eq!(remaining[1].timestamp, "20260124-120000");
    }

    #[test]
    fn test_restore_backup() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target");
        let backup_parent = temp_dir.path();

        // Create source and backup
        fs::create_dir(&source_dir).unwrap();
        fs::write(source_dir.join("file.txt"), "original").unwrap();

        let backup_path = create_backup(&source_dir, Some(backup_parent)).unwrap();
        let backup = BackupInfo::from_path(backup_path.clone(), source_dir.clone()).unwrap();

        // Modify source
        fs::write(source_dir.join("file.txt"), "modified").unwrap();

        // Restore to target
        restore_backup(&backup, &target_dir).unwrap();

        // Verify restoration
        assert!(target_dir.join("file.txt").exists());
        let content = fs::read_to_string(target_dir.join("file.txt")).unwrap();
        assert_eq!(content, "original");
    }
}
