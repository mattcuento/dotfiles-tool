pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;

use crate::error::Result;
use crate::install::version_manager::VersionManager;

/// Common interface for language installers
pub trait LanguageInstaller {
    /// Returns the language name (e.g., "java", "nodejs", "python")
    fn language_name(&self) -> &str;

    /// Returns the default version to install
    fn default_version(&self) -> &str;

    /// Returns a human-readable display name
    fn display_name(&self) -> &str;

    /// Installs the language using the specified version manager
    fn install(&self, vm: VersionManager, version: Option<&str>) -> Result<()> {
        let version = version.unwrap_or_else(|| self.default_version());
        crate::install::version_manager::install_language(vm, self.language_name(), version)
    }

    /// Provides fallback installation instructions if no version manager is available
    fn fallback_instructions(&self) -> String;
}

/// Returns all available language installers
pub fn all_languages() -> Vec<Box<dyn LanguageInstaller>> {
    vec![
        Box::new(java::JavaInstaller),
        Box::new(javascript::JavaScriptInstaller),
        Box::new(python::PythonInstaller),
        Box::new(rust::RustInstaller),
        Box::new(go::GoInstaller),
    ]
}

/// Gets a language installer by name
pub fn get_installer(name: &str) -> Option<Box<dyn LanguageInstaller>> {
    all_languages().into_iter().find(|installer| {
        installer.language_name() == name
            || installer.display_name().to_lowercase() == name.to_lowercase()
    })
}
