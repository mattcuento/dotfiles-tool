use crate::language::LanguageInstaller;

pub struct GoInstaller;

impl LanguageInstaller for GoInstaller {
    fn language_name(&self) -> &str {
        "golang"
    }

    fn default_version(&self) -> &str {
        "1.23.4"
    }

    fn display_name(&self) -> &str {
        "Go"
    }

    fn fallback_instructions(&self) -> String {
        format!(
            "Install {} manually:\n  \
            - macOS: brew install go\n  \
            - Linux: sudo apt install golang\n  \
            - Or visit: https://go.dev/doc/install",
            self.display_name()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_installer() {
        let installer = GoInstaller;
        assert_eq!(installer.language_name(), "golang");
        assert_eq!(installer.display_name(), "Go");
        assert!(installer.fallback_instructions().contains("go"));
    }
}
