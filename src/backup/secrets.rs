use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::Result;

/// Detected secret
#[derive(Debug, Clone, PartialEq)]
pub struct Secret {
    pub key: String,
    pub value: String,
    pub file: String,
    pub line_number: usize,
}

/// Secret patterns to detect
pub struct SecretPatterns {
    /// Matches environment variable assignments with secret-like names
    pub env_var: Regex,
    /// Matches API keys
    pub api_key: Regex,
    /// Matches tokens
    pub token: Regex,
    /// Matches passwords
    pub password: Regex,
}

impl SecretPatterns {
    pub fn new() -> Self {
        Self {
            // Matches: export TOKEN=value, API_KEY=value, TOKEN="value", TOKEN='value'
            env_var: Regex::new(r#"(?:export\s+)?([A-Z_]*(?:TOKEN|KEY|SECRET|PASSWORD|PASS|AUTH)[A-Z_]*)=(?:['"]?)([^'"\s]+)(?:['"]?)"#).unwrap(),
            // Matches: api_key: "value" or apiKey: "value"
            api_key: Regex::new(r#"(?:api[_-]?key|apiKey)[:\s=]+['"]?([^'"\s]+)['"]?"#).unwrap(),
            // Matches: token: "value"
            token: Regex::new(r#"(?:token|access[_-]?token)[:\s=]+['"]?([^'"\s]+)['"]?"#).unwrap(),
            // Matches: password: "value"
            password: Regex::new(r#"(?:password|passwd)[:\s=]+['"]?([^'"\s]+)['"]?"#).unwrap(),
        }
    }
}

impl Default for SecretPatterns {
    fn default() -> Self {
        Self::new()
    }
}

/// Scans a file for secrets
pub fn scan_file(file_path: &Path) -> Result<Vec<Secret>> {
    let patterns = SecretPatterns::new();
    let mut secrets = Vec::new();

    let content = fs::read_to_string(file_path)?;
    let file_name = file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    for (line_num, line) in content.lines().enumerate() {
        // Skip comments
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        // Check environment variable pattern
        if let Some(captures) = patterns.env_var.captures(line) {
            if let (Some(key), Some(value)) = (captures.get(1), captures.get(2)) {
                // Skip common non-secret variables
                let key_str = key.as_str();
                if !is_likely_secret(key_str) {
                    continue;
                }

                secrets.push(Secret {
                    key: key_str.to_string(),
                    value: value.as_str().to_string(),
                    file: file_name.clone(),
                    line_number: line_num + 1,
                });
            }
        }
    }

    Ok(secrets)
}

/// Determines if a key name is likely to be a secret
fn is_likely_secret(key: &str) -> bool {
    let key_upper = key.to_uppercase();

    // Must contain one of these keywords
    let secret_keywords = ["TOKEN", "KEY", "SECRET", "PASSWORD", "PASS", "AUTH"];
    let has_secret_keyword = secret_keywords
        .iter()
        .any(|keyword| key_upper.contains(keyword));

    if !has_secret_keyword {
        return false;
    }

    // Exclude common non-secrets
    let non_secrets = ["PUBLIC_KEY", "SSH_KEY_PATH", "KEY_FILE", "KEYMAP"];
    let is_non_secret = non_secrets
        .iter()
        .any(|non_secret| key_upper.contains(non_secret));

    has_secret_keyword && !is_non_secret
}

/// Scans a directory for secrets
pub fn scan_directory(dir_path: &Path) -> Result<Vec<Secret>> {
    let mut all_secrets = Vec::new();

    if !dir_path.exists() {
        return Ok(all_secrets);
    }

    // Config file extensions to scan
    let config_extensions = vec![
        "sh", "bash", "zsh", "fish", "rc", "conf", "config", "toml", "yaml", "yml", "json", "env",
    ];

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let should_scan = if let Some(ext) = path.extension() {
                config_extensions.contains(&ext.to_str().unwrap_or(""))
            } else {
                // Scan dotfiles without extension
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with('.'))
                    .unwrap_or(false)
            };

            if should_scan {
                if let Ok(secrets) = scan_file(&path) {
                    all_secrets.extend(secrets);
                }
            }
        }
    }

    Ok(all_secrets)
}

/// Extracts secrets to a .env file
pub fn extract_to_env(secrets: &[Secret], output_path: &Path) -> Result<()> {
    let mut env_content = String::new();
    env_content.push_str("# Extracted secrets - DO NOT COMMIT THIS FILE\n");
    env_content.push_str("# Add this file to .gitignore\n\n");

    // Deduplicate secrets by key (keep first occurrence)
    let mut seen_keys = std::collections::HashSet::new();

    for secret in secrets {
        if seen_keys.insert(&secret.key) {
            env_content.push_str(&format!(
                "{}={}\n",
                secret.key, secret.value
            ));
        }
    }

    fs::write(output_path, env_content)?;

    Ok(())
}

/// Generates a summary report of found secrets
pub fn summarize_secrets(secrets: &[Secret]) -> String {
    let mut by_file: HashMap<String, Vec<&Secret>> = HashMap::new();

    for secret in secrets {
        by_file
            .entry(secret.file.clone())
            .or_default()
            .push(secret);
    }

    let mut summary = String::new();
    summary.push_str(&format!("Found {} secret(s) across {} file(s):\n\n", secrets.len(), by_file.len()));

    for (file, file_secrets) in by_file.iter() {
        summary.push_str(&format!("{}:\n", file));
        for secret in file_secrets {
            summary.push_str(&format!(
                "  Line {}: {}\n",
                secret.line_number, secret.key
            ));
        }
        summary.push('\n');
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_secret_patterns_env_var() {
        let patterns = SecretPatterns::new();

        assert!(patterns.env_var.is_match("export API_TOKEN=abc123"));
        assert!(patterns.env_var.is_match("GITHUB_TOKEN=xyz789"));
        assert!(patterns.env_var.is_match("export SECRET_KEY=\"secret\""));
    }

    #[test]
    fn test_is_likely_secret() {
        assert!(is_likely_secret("API_TOKEN"));
        assert!(is_likely_secret("GITHUB_TOKEN"));
        assert!(is_likely_secret("SECRET_KEY"));
        assert!(is_likely_secret("DATABASE_PASSWORD"));

        assert!(!is_likely_secret("PUBLIC_KEY"));
        assert!(!is_likely_secret("SSH_KEY_PATH"));
        assert!(!is_likely_secret("HOME"));
        assert!(!is_likely_secret("PATH"));
    }

    #[test]
    fn test_scan_file_with_secrets() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.sh");

        fs::write(
            &file_path,
            "export API_TOKEN=abc123\nexport GITHUB_TOKEN=xyz789\nexport HOME=/home/user\n",
        )
        .unwrap();

        let secrets = scan_file(&file_path).unwrap();

        assert_eq!(secrets.len(), 2);
        assert_eq!(secrets[0].key, "API_TOKEN");
        assert_eq!(secrets[0].value, "abc123");
        assert_eq!(secrets[1].key, "GITHUB_TOKEN");
        assert_eq!(secrets[1].value, "xyz789");
    }

    #[test]
    fn test_scan_file_with_comments() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.sh");

        fs::write(
            &file_path,
            "# export API_TOKEN=abc123\nexport REAL_TOKEN=xyz789\n",
        )
        .unwrap();

        let secrets = scan_file(&file_path).unwrap();

        // Comment should be ignored
        assert_eq!(secrets.len(), 1);
        assert_eq!(secrets[0].key, "REAL_TOKEN");
    }

    #[test]
    fn test_scan_file_with_quoted_values() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.sh");

        fs::write(
            &file_path,
            "export API_TOKEN=\"abc123\"\nexport GITHUB_TOKEN='xyz789'\n",
        )
        .unwrap();

        let secrets = scan_file(&file_path).unwrap();

        assert_eq!(secrets.len(), 2);
        assert_eq!(secrets[0].value, "abc123");
        assert_eq!(secrets[1].value, "xyz789");
    }

    #[test]
    fn test_scan_directory() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("config.sh"),
            "export API_TOKEN=abc123\n",
        )
        .unwrap();

        fs::write(
            temp_dir.path().join(".zshrc"),
            "export GITHUB_TOKEN=xyz789\n",
        )
        .unwrap();

        fs::write(temp_dir.path().join("readme.txt"), "Not a config file\n").unwrap();

        let secrets = scan_directory(temp_dir.path()).unwrap();

        assert_eq!(secrets.len(), 2);
    }

    #[test]
    fn test_extract_to_env() {
        let temp_dir = TempDir::new().unwrap();
        let env_path = temp_dir.path().join(".env");

        let secrets = vec![
            Secret {
                key: "API_TOKEN".to_string(),
                value: "abc123".to_string(),
                file: "config.sh".to_string(),
                line_number: 1,
            },
            Secret {
                key: "GITHUB_TOKEN".to_string(),
                value: "xyz789".to_string(),
                file: "config.sh".to_string(),
                line_number: 2,
            },
        ];

        extract_to_env(&secrets, &env_path).unwrap();

        let content = fs::read_to_string(&env_path).unwrap();
        assert!(content.contains("API_TOKEN=abc123"));
        assert!(content.contains("GITHUB_TOKEN=xyz789"));
        assert!(content.contains("DO NOT COMMIT"));
    }

    #[test]
    fn test_extract_to_env_deduplicates() {
        let temp_dir = TempDir::new().unwrap();
        let env_path = temp_dir.path().join(".env");

        let secrets = vec![
            Secret {
                key: "API_TOKEN".to_string(),
                value: "abc123".to_string(),
                file: "config1.sh".to_string(),
                line_number: 1,
            },
            Secret {
                key: "API_TOKEN".to_string(),
                value: "different".to_string(),
                file: "config2.sh".to_string(),
                line_number: 1,
            },
        ];

        extract_to_env(&secrets, &env_path).unwrap();

        let content = fs::read_to_string(&env_path).unwrap();
        // Should only have one API_TOKEN (first occurrence)
        assert_eq!(content.matches("API_TOKEN").count(), 1);
        assert!(content.contains("API_TOKEN=abc123"));
    }

    #[test]
    fn test_summarize_secrets() {
        let secrets = vec![
            Secret {
                key: "API_TOKEN".to_string(),
                value: "abc123".to_string(),
                file: "config.sh".to_string(),
                line_number: 5,
            },
            Secret {
                key: "GITHUB_TOKEN".to_string(),
                value: "xyz789".to_string(),
                file: "config.sh".to_string(),
                line_number: 10,
            },
        ];

        let summary = summarize_secrets(&secrets);

        assert!(summary.contains("Found 2 secret(s)"));
        assert!(summary.contains("config.sh"));
        assert!(summary.contains("Line 5: API_TOKEN"));
        assert!(summary.contains("Line 10: GITHUB_TOKEN"));
    }
}
