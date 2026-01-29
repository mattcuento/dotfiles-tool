use crate::language::LanguageInstaller;

pub struct JavaScriptInstaller;

impl LanguageInstaller for JavaScriptInstaller {
    fn language_name(&self) -> &str {
        "nodejs"
    }

    fn default_version(&self) -> &str {
        "22.12.0"
    }

    fn display_name(&self) -> &str {
        "Node.js"
    }

    fn fallback_instructions(&self) -> String {
        format!(
            "Install {} manually:\n  \
            - macOS: brew install node\n  \
            - Linux: curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash - && sudo apt install nodejs\n  \
            - Or use nvm: https://github.com/nvm-sh/nvm",
            self.display_name()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_javascript_installer() {
        let installer = JavaScriptInstaller;
        assert_eq!(installer.language_name(), "nodejs");
        assert_eq!(installer.display_name(), "Node.js");
        assert!(installer.fallback_instructions().contains("node"));
    }
}
