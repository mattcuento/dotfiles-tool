use crate::error::Result;
use std::path::{Path, PathBuf};

pub fn detect_conflicts(home: &Path) -> Result<Vec<PathBuf>> {
    let mut conflicts = Vec::new();
    let files = vec![".zshrc", ".tmux.conf", ".config/nvim", ".gitconfig"];

    for file in files {
        let path = home.join(file);
        if path.exists() && !path.is_symlink() {
            conflicts.push(path);
        }
    }

    Ok(conflicts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_conflicts() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        // Create a regular file (conflict)
        fs::write(home.join(".zshrc"), "# test").unwrap();

        // Create a symlink (not a conflict)
        let target = temp.path().join("dotfiles").join(".tmux.conf");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, "# tmux").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, home.join(".tmux.conf")).unwrap();

        let conflicts = detect_conflicts(home).unwrap();
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].ends_with(".zshrc"));
    }
}
