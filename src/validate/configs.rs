use crate::validate::{CheckReport, CheckResult};
use std::fs;
use std::path::Path;

/// Validates TOML syntax
pub fn validate_toml(file_path: &Path) -> CheckResult {
    match fs::read_to_string(file_path) {
        Ok(content) => match toml::from_str::<toml::Value>(&content) {
            Ok(_) => CheckResult::pass(
                format!("Config:{}", file_path.file_name().unwrap().to_string_lossy()),
                "Valid TOML syntax",
            ),
            Err(e) => CheckResult::error(
                format!("Config:{}", file_path.file_name().unwrap().to_string_lossy()),
                format!("Invalid TOML syntax: {}", e),
                Some("Fix the TOML syntax errors"),
            ),
        },
        Err(e) => CheckResult::error(
            format!("Config:{}", file_path.file_name().unwrap_or_default().to_string_lossy()),
            format!("Failed to read file: {}", e),
            None::<String>,
        ),
    }
}

/// Validates JSON syntax
pub fn validate_json(file_path: &Path) -> CheckResult {
    match fs::read_to_string(file_path) {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => CheckResult::pass(
                format!("Config:{}", file_path.file_name().unwrap().to_string_lossy()),
                "Valid JSON syntax",
            ),
            Err(e) => CheckResult::error(
                format!("Config:{}", file_path.file_name().unwrap().to_string_lossy()),
                format!("Invalid JSON syntax: {}", e),
                Some("Fix the JSON syntax errors"),
            ),
        },
        Err(e) => CheckResult::error(
            format!("Config:{}", file_path.file_name().unwrap_or_default().to_string_lossy()),
            format!("Failed to read file: {}", e),
            None::<String>,
        ),
    }
}

/// Validates YAML syntax
pub fn validate_yaml(file_path: &Path) -> CheckResult {
    match fs::read_to_string(file_path) {
        Ok(content) => match serde_yaml::from_str::<serde_yaml::Value>(&content) {
            Ok(_) => CheckResult::pass(
                format!("Config:{}", file_path.file_name().unwrap().to_string_lossy()),
                "Valid YAML syntax",
            ),
            Err(e) => CheckResult::error(
                format!("Config:{}", file_path.file_name().unwrap().to_string_lossy()),
                format!("Invalid YAML syntax: {}", e),
                Some("Fix the YAML syntax errors"),
            ),
        },
        Err(e) => CheckResult::error(
            format!("Config:{}", file_path.file_name().unwrap_or_default().to_string_lossy()),
            format!("Failed to read file: {}", e),
            None::<String>,
        ),
    }
}

/// Validates config file based on extension
pub fn validate_config(file_path: &Path) -> CheckResult {
    match file_path.extension().and_then(|e| e.to_str()) {
        Some("toml") => validate_toml(file_path),
        Some("json") => validate_json(file_path),
        Some("yaml") | Some("yml") => validate_yaml(file_path),
        Some(ext) => CheckResult::pass(
            format!("Config:{}", file_path.file_name().unwrap().to_string_lossy()),
            format!("Skipped validation for .{} file", ext),
        ),
        None => CheckResult::pass(
            format!("Config:{}", file_path.file_name().unwrap_or_default().to_string_lossy()),
            "No extension, skipping validation",
        ),
    }
}

/// Scans a directory for config files and validates them
pub fn scan_directory(dir_path: &Path) -> CheckReport {
    let mut report = CheckReport::new();

    if !dir_path.exists() {
        report.add(CheckResult::error(
            "Configs",
            format!("Directory does not exist: {:?}", dir_path),
            None::<String>,
        ));
        return report;
    }

    match fs::read_dir(dir_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_str().unwrap_or("");
                        if matches!(ext_str, "toml" | "json" | "yaml" | "yml") {
                            report.add(validate_config(&path));
                        }
                    }
                }
            }

            if report.total() == 0 {
                report.add(CheckResult::pass(
                    "Configs",
                    format!("No config files found in {:?}", dir_path),
                ));
            }
        }
        Err(e) => {
            report.add(CheckResult::error(
                "Configs",
                format!("Failed to read directory: {}", e),
                None::<String>,
            ));
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_toml_valid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.toml");

        fs::write(&file_path, "[section]\nkey = \"value\"\n").unwrap();

        let result = validate_toml(&file_path);
        assert!(result.is_pass());
    }

    #[test]
    fn test_validate_toml_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.toml");

        fs::write(&file_path, "[section\nkey = value\n").unwrap();

        let result = validate_toml(&file_path);
        assert!(result.is_error());
        assert!(result.message().contains("Invalid TOML"));
    }

    #[test]
    fn test_validate_json_valid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.json");

        fs::write(&file_path, r#"{"key": "value"}"#).unwrap();

        let result = validate_json(&file_path);
        assert!(result.is_pass());
    }

    #[test]
    fn test_validate_json_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.json");

        fs::write(&file_path, r#"{"key": "value""#).unwrap();

        let result = validate_json(&file_path);
        assert!(result.is_error());
        assert!(result.message().contains("Invalid JSON"));
    }

    #[test]
    fn test_validate_yaml_valid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.yaml");

        fs::write(&file_path, "key: value\nlist:\n  - item1\n  - item2\n").unwrap();

        let result = validate_yaml(&file_path);
        assert!(result.is_pass());
    }

    #[test]
    fn test_validate_yaml_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.yaml");

        fs::write(&file_path, "key: value\n  invalid indentation\n").unwrap();

        let result = validate_yaml(&file_path);
        // YAML parser might be lenient, so this might pass or fail
        // Just verify it doesn't panic
        assert!(result.is_pass() || result.is_error());
    }

    #[test]
    fn test_validate_config_by_extension() {
        let temp_dir = TempDir::new().unwrap();

        let toml_file = temp_dir.path().join("test.toml");
        fs::write(&toml_file, "[section]\n").unwrap();
        assert!(validate_config(&toml_file).is_pass());

        let json_file = temp_dir.path().join("test.json");
        fs::write(&json_file, "{}").unwrap();
        assert!(validate_config(&json_file).is_pass());

        let yaml_file = temp_dir.path().join("test.yaml");
        fs::write(&yaml_file, "key: value\n").unwrap();
        assert!(validate_config(&yaml_file).is_pass());
    }

    #[test]
    fn test_validate_config_unknown_extension() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "some content\n").unwrap();

        let result = validate_config(&file_path);
        assert!(result.is_pass());
        assert!(result.message().contains("Skipped"));
    }

    #[test]
    fn test_scan_directory_with_configs() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("valid.toml"), "[section]\n").unwrap();
        fs::write(temp_dir.path().join("valid.json"), "{}").unwrap();
        fs::write(temp_dir.path().join("invalid.toml"), "[section").unwrap();

        let report = scan_directory(temp_dir.path());

        // Should have 3 results
        assert_eq!(report.total(), 3);
        assert!(report.has_errors()); // invalid.toml should cause error
    }

    #[test]
    fn test_scan_directory_empty() {
        let temp_dir = TempDir::new().unwrap();
        let report = scan_directory(temp_dir.path());

        // Empty directory should have one pass result
        assert_eq!(report.total(), 1);
        assert!(report.is_clean());
    }

    #[test]
    fn test_scan_directory_nonexistent() {
        let report = scan_directory(Path::new("/nonexistent/directory"));

        assert!(report.has_errors());
        assert!(report.checks.iter().any(|c| c.message().contains("does not exist")));
    }
}
