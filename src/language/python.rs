use crate::language::LanguageInstaller;

pub struct PythonInstaller;

impl LanguageInstaller for PythonInstaller {
    fn language_name(&self) -> &str {
        "python"
    }

    fn default_version(&self) -> &str {
        "3.12.1"
    }

    fn display_name(&self) -> &str {
        "Python"
    }

    fn fallback_instructions(&self) -> String {
        format!(
            "Install {} manually:\n  \
            - macOS: brew install python@3.12\n  \
            - Linux: sudo apt install python3.12\n  \
            - Or use pyenv: https://github.com/pyenv/pyenv",
            self.display_name()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_installer() {
        let installer = PythonInstaller;
        assert_eq!(installer.language_name(), "python");
        assert_eq!(installer.display_name(), "Python");
        assert!(installer.fallback_instructions().contains("python"));
    }
}
