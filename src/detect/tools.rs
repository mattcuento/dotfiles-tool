use std::process::Command;

pub fn is_installed(tool: &str) -> bool {
    Command::new("which")
        .arg(tool)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn get_tool_path(tool: &str) -> Option<String> {
    Command::new("which")
        .arg(tool)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_installed() {
        // Should always have 'ls' on Unix systems
        assert!(is_installed("ls"));
        assert!(!is_installed("nonexistent-tool-xyz"));
    }

    #[test]
    fn test_get_tool_path() {
        let path = get_tool_path("ls");
        assert!(path.is_some());
        assert!(path.unwrap().contains("ls"));
    }
}
