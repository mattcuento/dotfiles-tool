use crate::error::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;

/// Ensures a script is sourced in shell RC file
pub fn ensure_script_sourced(shell_rc: &Path, script_path: &Path, script_name: &str) -> Result<()> {
    // Read existing content
    let content = if shell_rc.exists() {
        fs::read_to_string(shell_rc)?
    } else {
        String::new()
    };

    // Check if already sourced
    if is_script_sourced(&content, script_path) {
        println!(
            "{}",
            format!(
                "  ✓ {} already sourced in {}",
                script_name,
                shell_rc.display()
            )
            .green()
        );
        return Ok(());
    }

    // Append source line
    let script_str = script_path
        .to_str()
        .ok_or_else(|| crate::error::DotfilesError::Config("Invalid script path".to_string()))?;

    let source_line = format!(
        "\n# Source {} (added by dotfiles-tool)\nsource {}\n",
        script_name, script_str
    );

    let new_content = content + &source_line;
    fs::write(shell_rc, new_content)?;

    println!(
        "{}",
        format!("  ✓ Added {} to {}", script_name, shell_rc.display()).green()
    );
    Ok(())
}

/// Checks if a script is already sourced in content
fn is_script_sourced(content: &str, script_path: &Path) -> bool {
    let script_str = script_path.to_str().unwrap_or("");

    content.contains(&format!("source {}", script_str))
        || content.contains(&format!(". {}", script_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_script_sourced_with_source() {
        let content = "source /path/to/script.sh\nother content";
        assert!(is_script_sourced(content, Path::new("/path/to/script.sh")));
    }

    #[test]
    fn test_is_script_sourced_with_dot() {
        let content = ". /path/to/script.sh\nother content";
        assert!(is_script_sourced(content, Path::new("/path/to/script.sh")));
    }

    #[test]
    fn test_is_script_not_sourced() {
        let content = "# some config\nalias ls='ls -la'";
        assert!(!is_script_sourced(content, Path::new("/path/to/script.sh")));
    }

    #[test]
    fn test_ensure_script_sourced_new_file() {
        let temp = TempDir::new().unwrap();
        let zshrc = temp.path().join(".zshrc");
        let script = temp.path().join("script.sh");

        fs::write(&script, "#!/bin/bash\necho test").unwrap();

        let result = ensure_script_sourced(&zshrc, &script, "script.sh");
        assert!(result.is_ok());

        let content = fs::read_to_string(&zshrc).unwrap();
        assert!(content.contains("source"));
        assert!(content.contains("script.sh"));
    }

    #[test]
    fn test_ensure_script_sourced_existing_file() {
        let temp = TempDir::new().unwrap();
        let zshrc = temp.path().join(".zshrc");
        let script = temp.path().join("script.sh");

        fs::write(&script, "#!/bin/bash\necho test").unwrap();
        fs::write(&zshrc, "# existing content\n").unwrap();

        let result = ensure_script_sourced(&zshrc, &script, "script.sh");
        assert!(result.is_ok());

        let content = fs::read_to_string(&zshrc).unwrap();
        assert!(content.contains("existing content"));
        assert!(content.contains("source"));
        assert!(content.contains("script.sh"));
    }

    #[test]
    fn test_ensure_script_sourced_already_sourced() {
        let temp = TempDir::new().unwrap();
        let zshrc = temp.path().join(".zshrc");
        let script = temp.path().join("script.sh");

        fs::write(&script, "#!/bin/bash\necho test").unwrap();

        let script_str = script.to_str().unwrap();
        let initial_content = format!("source {}\n", script_str);
        fs::write(&zshrc, &initial_content).unwrap();

        let result = ensure_script_sourced(&zshrc, &script, "script.sh");
        assert!(result.is_ok());

        // Content should be unchanged
        let content = fs::read_to_string(&zshrc).unwrap();
        assert_eq!(content, initial_content);
    }

    #[test]
    fn test_ensure_script_sourced_preserves_existing_content() {
        let temp = TempDir::new().unwrap();
        let zshrc = temp.path().join(".zshrc");
        let script = temp.path().join("script.sh");

        fs::write(&script, "#!/bin/bash\necho test").unwrap();

        let existing = "export PATH=/usr/local/bin:$PATH\nalias ll='ls -la'\n";
        fs::write(&zshrc, existing).unwrap();

        let result = ensure_script_sourced(&zshrc, &script, "script.sh");
        assert!(result.is_ok());

        let content = fs::read_to_string(&zshrc).unwrap();
        assert!(content.contains(existing));
        assert!(content.contains("source"));
    }
}
