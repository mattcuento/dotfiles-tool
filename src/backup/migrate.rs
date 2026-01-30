use crate::backup::{self, secrets};
use crate::error::{DotfilesError, Result};
use crate::symlink::{self, SymlinkReport, Symlinker};
use std::path::{Path, PathBuf};

/// Migration options
#[derive(Debug, Clone)]
pub struct MigrationOptions {
    /// Source dotfiles directory (existing setup)
    pub source: PathBuf,
    /// Target dotfiles directory (new setup)
    pub target: PathBuf,
    /// Whether to extract secrets
    pub extract_secrets: bool,
    /// Whether to create backup before migration
    pub create_backup: bool,
    /// Dry run mode (no actual changes)
    pub dry_run: bool,
}

impl MigrationOptions {
    pub fn new(source: PathBuf, target: PathBuf) -> Self {
        Self {
            source,
            target,
            extract_secrets: true,
            create_backup: true,
            dry_run: false,
        }
    }
}

/// Result of a migration operation
#[derive(Debug)]
pub struct MigrationResult {
    pub backup_path: Option<PathBuf>,
    pub secrets_extracted: usize,
    pub symlink_report: Option<SymlinkReport>,
    pub conflicts: Vec<(PathBuf, String)>,
}

/// Migrates dotfiles from old setup to new setup
pub fn migrate(options: &MigrationOptions) -> Result<MigrationResult> {
    let mut result = MigrationResult {
        backup_path: None,
        secrets_extracted: 0,
        symlink_report: None,
        conflicts: Vec::new(),
    };

    // Step 1: Validate source exists
    if !options.source.exists() {
        return Err(DotfilesError::Config(format!(
            "Source directory does not exist: {:?}",
            options.source
        )));
    }

    // Step 2: Create backup if requested
    if options.create_backup && !options.dry_run {
        println!("Creating backup before migration...");
        let backup_path = backup::create_backup(&options.source, None)?;
        result.backup_path = Some(backup_path);
    }

    // Step 3: Extract secrets if requested
    if options.extract_secrets {
        println!("Scanning for secrets...");
        let found_secrets = secrets::scan_directory(&options.source)?;

        if !found_secrets.is_empty() {
            println!("{}", secrets::summarize_secrets(&found_secrets));

            if !options.dry_run {
                let env_path = options.target.join(".env");
                secrets::extract_to_env(&found_secrets, &env_path)?;
                println!(
                    "✓ Extracted {} secrets to {:?}",
                    found_secrets.len(),
                    env_path
                );
            }

            result.secrets_extracted = found_secrets.len();
        } else {
            println!("No secrets found");
        }
    }

    // Step 4: Detect conflicts
    println!("Checking for conflicts...");
    let conflicts = symlink::detect_conflicts(&options.source, &options.target);

    if !conflicts.is_empty() {
        println!("⚠ Found {} conflict(s):", conflicts.len());
        for (path, reason) in &conflicts {
            println!("  - {:?}: {}", path, reason);
        }
        result.conflicts = conflicts;
    }

    // Step 5: Create symlinks (if no conflicts or dry run)
    if result.conflicts.is_empty() || options.dry_run {
        println!("Creating symlinks...");

        // Use manual symlinker for migration (more control)
        let symlinker = symlink::manual::ManualSymlinker {
            dry_run: options.dry_run,
            force: false,
        };

        let report = symlinker.symlink(&options.source, &options.target)?;

        if options.dry_run {
            println!("Dry run - no changes made");
        }

        println!("✓ Symlink operation: {}", report.summary());
        result.symlink_report = Some(report);
    } else {
        println!("⚠ Migration aborted due to conflicts");
        println!("  Resolve conflicts manually or use --force flag");
    }

    Ok(result)
}

/// Rolls back a migration by restoring from the most recent backup
pub fn rollback(target: &Path) -> Result<()> {
    println!("Rolling back migration...");

    // Find the most recent backup
    let backup = backup::get_latest_backup(None)?
        .ok_or_else(|| DotfilesError::Config("No backup found to rollback from".to_string()))?;

    println!("Restoring from backup: {}", backup.timestamp);

    // Restore the backup
    backup::restore_backup(&backup, target)?;

    println!("✓ Rollback complete");

    Ok(())
}

/// Verifies migration was successful
pub fn verify_migration(source: &Path, target: &Path) -> Result<Vec<(PathBuf, String)>> {
    println!("Verifying migration...");

    let issues = symlink::validate_symlinks(source, target)?;

    if issues.is_empty() {
        println!("✓ All symlinks are valid");
    } else {
        println!("⚠ Found {} issue(s):", issues.len());
        for (path, issue) in &issues {
            println!("  - {:?}: {}", path, issue);
        }
    }

    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_migration_options_new() {
        let source = PathBuf::from("/source");
        let target = PathBuf::from("/target");
        let options = MigrationOptions::new(source.clone(), target.clone());

        assert_eq!(options.source, source);
        assert_eq!(options.target, target);
        assert!(options.extract_secrets);
        assert!(options.create_backup);
        assert!(!options.dry_run);
    }

    #[test]
    fn test_migrate_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source");
        let target = temp_dir.path().join("target");

        // Create source with a file
        fs::create_dir(&source).unwrap();
        fs::write(source.join("config.txt"), "test").unwrap();

        fs::create_dir(&target).unwrap();

        let mut options = MigrationOptions::new(source, target.clone());
        options.dry_run = true;
        options.create_backup = false;

        let result = migrate(&options).unwrap();

        assert_eq!(result.secrets_extracted, 0);
        assert!(result.symlink_report.is_some());

        // In dry run, no symlinks should actually be created
        assert!(!target.join("config.txt").exists());
    }

    #[test]
    fn test_migrate_with_secrets() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source");
        let target = temp_dir.path().join("target");

        // Create source with a config containing secrets
        fs::create_dir(&source).unwrap();
        fs::write(source.join("config.sh"), "export API_TOKEN=secret123\n").unwrap();

        fs::create_dir(&target).unwrap();

        let mut options = MigrationOptions::new(source, target.clone());
        options.dry_run = true;
        options.create_backup = false;

        let result = migrate(&options).unwrap();

        assert_eq!(result.secrets_extracted, 1);
    }

    #[test]
    fn test_migrate_nonexistent_source() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("nonexistent");
        let target = temp_dir.path().join("target");

        let options = MigrationOptions::new(source, target);
        let result = migrate(&options);

        assert!(result.is_err());
    }

    #[test]
    fn test_verify_migration() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source");
        let target = temp_dir.path().join("target");

        // Create source and target
        fs::create_dir(&source).unwrap();
        fs::write(source.join("file.txt"), "content").unwrap();

        fs::create_dir(&target).unwrap();

        // No symlinks exist yet, so verification should find issues
        let issues = verify_migration(&source, &target).unwrap();

        assert!(!issues.is_empty());
    }

    #[test]
    fn test_rollback_no_backup() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target");

        fs::create_dir(&target).unwrap();

        // Should fail because no backup exists
        let result = rollback(&target);
        assert!(result.is_err());
    }
}
