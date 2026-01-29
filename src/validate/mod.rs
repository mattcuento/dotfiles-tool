pub mod configs;
pub mod dependencies;
pub mod paths;
pub mod symlinks;

use colored::Colorize;

/// Result of a validation check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// Check passed successfully
    Pass {
        name: String,
        message: String,
    },
    /// Check passed with warnings
    Warn {
        name: String,
        message: String,
        suggestion: Option<String>,
    },
    /// Check failed with errors
    Error {
        name: String,
        message: String,
        suggestion: Option<String>,
    },
}

impl CheckResult {
    /// Creates a passing check result
    pub fn pass(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Pass {
            name: name.into(),
            message: message.into(),
        }
    }

    /// Creates a warning check result
    pub fn warn(
        name: impl Into<String>,
        message: impl Into<String>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::Warn {
            name: name.into(),
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates an error check result
    pub fn error(
        name: impl Into<String>,
        message: impl Into<String>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::Error {
            name: name.into(),
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Returns true if this is a passing check
    pub fn is_pass(&self) -> bool {
        matches!(self, CheckResult::Pass { .. })
    }

    /// Returns true if this is a warning
    pub fn is_warn(&self) -> bool {
        matches!(self, CheckResult::Warn { .. })
    }

    /// Returns true if this is an error
    pub fn is_error(&self) -> bool {
        matches!(self, CheckResult::Error { .. })
    }

    /// Returns the check name
    pub fn name(&self) -> &str {
        match self {
            CheckResult::Pass { name, .. } => name,
            CheckResult::Warn { name, .. } => name,
            CheckResult::Error { name, .. } => name,
        }
    }

    /// Returns the message
    pub fn message(&self) -> &str {
        match self {
            CheckResult::Pass { message, .. } => message,
            CheckResult::Warn { message, .. } => message,
            CheckResult::Error { message, .. } => message,
        }
    }

    /// Returns the suggestion if available
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            CheckResult::Pass { .. } => None,
            CheckResult::Warn { suggestion, .. } => suggestion.as_deref(),
            CheckResult::Error { suggestion, .. } => suggestion.as_deref(),
        }
    }

    /// Formats this check result with colors
    pub fn format_colored(&self) -> String {
        match self {
            CheckResult::Pass { name, message } => {
                format!("  {} {} - {}", "✓".green(), name.bold(), message)
            }
            CheckResult::Warn {
                name,
                message,
                suggestion,
            } => {
                let mut output = format!("  {} {} - {}", "⚠".yellow(), name.bold(), message);
                if let Some(fix) = suggestion {
                    output.push_str(&format!("\n    {}: {}", "Fix".bold(), fix.dimmed()));
                }
                output
            }
            CheckResult::Error {
                name,
                message,
                suggestion,
            } => {
                let mut output = format!("  {} {} - {}", "✗".red(), name.bold(), message);
                if let Some(fix) = suggestion {
                    output.push_str(&format!("\n    {}: {}", "Fix".bold(), fix.dimmed()));
                }
                output
            }
        }
    }
}

/// Report containing multiple check results
#[derive(Debug, Clone, Default)]
pub struct CheckReport {
    pub checks: Vec<CheckResult>,
}

impl CheckReport {
    /// Creates a new empty report
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a check result to the report
    pub fn add(&mut self, result: CheckResult) {
        self.checks.push(result);
    }

    /// Returns the number of passing checks
    pub fn pass_count(&self) -> usize {
        self.checks.iter().filter(|c| c.is_pass()).count()
    }

    /// Returns the number of warnings
    pub fn warn_count(&self) -> usize {
        self.checks.iter().filter(|c| c.is_warn()).count()
    }

    /// Returns the number of errors
    pub fn error_count(&self) -> usize {
        self.checks.iter().filter(|c| c.is_error()).count()
    }

    /// Returns true if all checks passed (no warnings or errors)
    pub fn is_clean(&self) -> bool {
        self.warn_count() == 0 && self.error_count() == 0
    }

    /// Returns true if there are any errors
    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }

    /// Returns the total number of checks
    pub fn total(&self) -> usize {
        self.checks.len()
    }

    /// Formats the report with colors
    pub fn format_colored(&self) -> String {
        let mut output = String::new();

        // Group checks by category (based on name prefix)
        let mut categories: std::collections::HashMap<String, Vec<&CheckResult>> =
            std::collections::HashMap::new();

        for check in &self.checks {
            let category = check
                .name()
                .split(':')
                .next()
                .unwrap_or("General")
                .to_string();
            categories.entry(category).or_default().push(check);
        }

        // Sort categories for consistent output
        let mut category_names: Vec<_> = categories.keys().collect();
        category_names.sort();

        // Print each category
        for category in category_names {
            output.push_str(&format!("\n{}\n", category.bold().underline()));
            for check in &categories[category] {
                output.push_str(&check.format_colored());
                output.push('\n');
            }
        }

        // Summary
        output.push_str(&format!("\n{}\n", "Summary".bold().underline()));
        output.push_str(&format!(
            "  {} {} passed\n",
            "✓".green(),
            self.pass_count()
        ));

        if self.warn_count() > 0 {
            output.push_str(&format!(
                "  {} {} warnings\n",
                "⚠".yellow(),
                self.warn_count()
            ));
        }

        if self.error_count() > 0 {
            output.push_str(&format!("  {} {} errors\n", "✗".red(), self.error_count()));
        }

        output.push_str(&format!("  Total: {} checks\n", self.total()));

        output
    }

    /// Returns a simple summary string
    pub fn summary(&self) -> String {
        format!(
            "Passed: {}, Warnings: {}, Errors: {}",
            self.pass_count(),
            self.warn_count(),
            self.error_count()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_pass() {
        let result = CheckResult::pass("Test", "Everything is good");
        assert!(result.is_pass());
        assert!(!result.is_warn());
        assert!(!result.is_error());
        assert_eq!(result.name(), "Test");
        assert_eq!(result.message(), "Everything is good");
        assert_eq!(result.suggestion(), None);
    }

    #[test]
    fn test_check_result_warn() {
        let result = CheckResult::warn(
            "Test",
            "Something might be wrong",
            Some("Try fixing this"),
        );
        assert!(!result.is_pass());
        assert!(result.is_warn());
        assert!(!result.is_error());
        assert_eq!(result.name(), "Test");
        assert_eq!(result.message(), "Something might be wrong");
        assert_eq!(result.suggestion(), Some("Try fixing this"));
    }

    #[test]
    fn test_check_result_error() {
        let result = CheckResult::error("Test", "Something is broken", Some("Run this command"));
        assert!(!result.is_pass());
        assert!(!result.is_warn());
        assert!(result.is_error());
        assert_eq!(result.name(), "Test");
        assert_eq!(result.message(), "Something is broken");
        assert_eq!(result.suggestion(), Some("Run this command"));
    }

    #[test]
    fn test_check_result_error_no_suggestion() {
        let result: CheckResult = CheckResult::error("Test", "Something is broken", None::<String>);
        assert!(result.is_error());
        assert_eq!(result.suggestion(), None);
    }

    #[test]
    fn test_check_report_new() {
        let report = CheckReport::new();
        assert_eq!(report.total(), 0);
        assert_eq!(report.pass_count(), 0);
        assert_eq!(report.warn_count(), 0);
        assert_eq!(report.error_count(), 0);
        assert!(report.is_clean());
        assert!(!report.has_errors());
    }

    #[test]
    fn test_check_report_add() {
        let mut report = CheckReport::new();

        report.add(CheckResult::pass("Test1", "Good"));
        assert_eq!(report.pass_count(), 1);
        assert_eq!(report.total(), 1);

        report.add(CheckResult::warn("Test2", "Warning", Some("Fix it")));
        assert_eq!(report.warn_count(), 1);
        assert_eq!(report.total(), 2);

        report.add(CheckResult::error("Test3", "Error", Some("Fix this")));
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.total(), 3);
    }

    #[test]
    fn test_check_report_is_clean() {
        let mut report = CheckReport::new();
        report.add(CheckResult::pass("Test", "Good"));
        assert!(report.is_clean());

        report.add(CheckResult::warn("Test", "Warning", None::<String>));
        assert!(!report.is_clean());
    }

    #[test]
    fn test_check_report_has_errors() {
        let mut report = CheckReport::new();
        report.add(CheckResult::pass("Test", "Good"));
        assert!(!report.has_errors());

        report.add(CheckResult::error("Test", "Error", None::<String>));
        assert!(report.has_errors());
    }

    #[test]
    fn test_check_report_summary() {
        let mut report = CheckReport::new();
        report.add(CheckResult::pass("Test1", "Good"));
        report.add(CheckResult::warn("Test2", "Warning", None::<String>));
        report.add(CheckResult::error("Test3", "Error", None::<String>));

        let summary = report.summary();
        assert!(summary.contains("Passed: 1"));
        assert!(summary.contains("Warnings: 1"));
        assert!(summary.contains("Errors: 1"));
    }

    #[test]
    fn test_check_result_format_colored() {
        let pass = CheckResult::pass("Test", "Good");
        let formatted = pass.format_colored();
        assert!(formatted.contains("Test"));
        assert!(formatted.contains("Good"));

        let warn = CheckResult::warn("Test", "Warning", Some("Fix"));
        let formatted = warn.format_colored();
        assert!(formatted.contains("Test"));
        assert!(formatted.contains("Warning"));
        assert!(formatted.contains("Fix"));
    }
}
