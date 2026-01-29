use crate::language::LanguageInstaller;

pub struct JavaInstaller;

impl LanguageInstaller for JavaInstaller {
    fn language_name(&self) -> &str {
        "java"
    }

    fn default_version(&self) -> &str {
        "openjdk-21"
    }

    fn display_name(&self) -> &str {
        "Java"
    }

    fn fallback_instructions(&self) -> String {
        format!(
            "Install {} manually:\n  \
            - macOS: brew install openjdk@21\n  \
            - Linux: sudo apt install openjdk-21-jdk",
            self.display_name()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_installer() {
        let installer = JavaInstaller;
        assert_eq!(installer.language_name(), "java");
        assert_eq!(installer.default_version(), "openjdk-21");
        assert_eq!(installer.display_name(), "Java");
        assert!(installer.fallback_instructions().contains("openjdk"));
    }
}
