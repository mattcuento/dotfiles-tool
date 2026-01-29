use crate::language::LanguageInstaller;

pub struct RustInstaller;

impl LanguageInstaller for RustInstaller {
    fn language_name(&self) -> &str {
        "rust"
    }

    fn default_version(&self) -> &str {
        "1.83.0"
    }

    fn display_name(&self) -> &str {
        "Rust"
    }

    fn fallback_instructions(&self) -> String {
        format!(
            "Install {} manually:\n  \
            - All platforms: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\n  \
            - Or visit: https://rustup.rs",
            self.display_name()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_installer() {
        let installer = RustInstaller;
        assert_eq!(installer.language_name(), "rust");
        assert_eq!(installer.display_name(), "Rust");
        assert!(installer.fallback_instructions().contains("rustup"));
    }
}
