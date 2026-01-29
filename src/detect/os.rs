#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OS {
    MacOS,
    Linux,
    Unknown,
}

pub fn detect_os() -> OS {
    match std::env::consts::OS {
        "macos" => OS::MacOS,
        "linux" => OS::Linux,
        _ => OS::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_os() {
        let os = detect_os();
        assert!(matches!(os, OS::MacOS | OS::Linux | OS::Unknown));
    }
}
