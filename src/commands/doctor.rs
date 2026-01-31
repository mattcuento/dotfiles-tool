use crate::error::Result;
use crate::install;
use crate::validate;
use colored::Colorize;

/// Runs the doctor command to validate the dotfiles setup
pub fn run() -> Result<()> {
    println!("{}", "🏥 Dotfiles Health Check".bold());
    println!();

    // Collect all validation results
    let mut overall_report = validate::CheckReport::new();

    // 1. Validate dependencies
    println!("{}", "Checking dependencies...".bold());
    let dep_report = validate::dependencies::validate_all();
    for check in dep_report.checks {
        overall_report.add(check);
    }
    println!();

    // 1b. Validate brew packages (categorized)
    println!("{}", "Checking brew packages...".bold());
    let pkg_status = install::packages::package_status();

    // Essential packages (errors if missing)
    for pkg in &pkg_status.missing_essential {
        overall_report.add(validate::CheckResult::error(
            "Essential Package",
            format!("Missing essential package: {}", pkg),
            Some(format!("Run: brew install {}", pkg)),
        ));
    }

    // Development packages (warnings if missing)
    let missing_dev = install::packages::check_development_packages();
    if !missing_dev.is_empty() {
        overall_report.add(validate::CheckResult::warn(
            "Development Tools",
            format!(
                "Missing {} development tools: {}",
                missing_dev.len(),
                missing_dev.join(", ")
            ),
            Some("Run: dotfiles setup (or manually install)"),
        ));
    } else if !install::packages::DEVELOPMENT_PACKAGES.is_empty() {
        overall_report.add(validate::CheckResult::pass(
            "Development Tools",
            "All development tools installed",
        ));
    }

    // Cloud packages (warnings if missing)
    let missing_cloud = install::packages::check_cloud_packages();
    if !missing_cloud.is_empty() {
        overall_report.add(validate::CheckResult::warn(
            "Cloud Tools",
            format!(
                "Missing {} cloud tools: {}",
                missing_cloud.len(),
                missing_cloud.join(", ")
            ),
            Some("Run: brew install awscli opentofu"),
        ));
    } else if !install::packages::CLOUD_PACKAGES.is_empty() {
        overall_report.add(validate::CheckResult::pass(
            "Cloud Tools",
            "All cloud tools installed",
        ));
    }

    // Productivity packages (info only)
    let missing_productivity = install::packages::check_productivity_packages();
    if !missing_productivity.is_empty() {
        overall_report.add(validate::CheckResult::pass(
            "Productivity Tools",
            format!(
                "Optional: {} productivity tools available for install ({})",
                missing_productivity.len(),
                missing_productivity.join(", ")
            ),
        ));
    } else if !install::packages::PRODUCTIVITY_PACKAGES.is_empty() {
        overall_report.add(validate::CheckResult::pass(
            "Productivity Tools",
            "All productivity tools installed",
        ));
    }

    // Editor packages (info only)
    let missing_editors = install::packages::check_editor_packages();
    if !missing_editors.is_empty() {
        overall_report.add(validate::CheckResult::pass(
            "Editor Tools",
            format!(
                "Optional: {} editor tools available for install ({})",
                missing_editors.len(),
                missing_editors.join(", ")
            ),
        ));
    } else if !install::packages::EDITOR_PACKAGES.is_empty() {
        overall_report.add(validate::CheckResult::pass(
            "Editor Tools",
            "All editor tools installed",
        ));
    }

    println!();

    // 2. Validate symlinks (if dotfiles dir exists)
    if let Some(home) = dirs::home_dir() {
        let dotfiles_dir = home.join("dotfiles");
        if dotfiles_dir.exists() {
            println!("{}", "Checking symlinks...".bold());
            let symlink_report = validate::symlinks::validate_symlinks(&dotfiles_dir, &home);
            for check in symlink_report.checks {
                overall_report.add(check);
            }
            println!();
        }
    }

    // 3. Check for hardcoded paths
    if let Some(home) = dirs::home_dir() {
        let config_dir = home.join(".config");
        if config_dir.exists() {
            println!("{}", "Scanning for hardcoded paths...".bold());
            let paths_report = validate::paths::scan_directory(&config_dir);
            for check in paths_report.checks {
                overall_report.add(check);
            }
            println!();
        }
    }

    // 4. Validate config file syntax
    if let Some(home) = dirs::home_dir() {
        let config_dir = home.join(".config");
        if config_dir.exists() {
            println!("{}", "Validating config files...".bold());
            let config_report = validate::configs::scan_directory(&config_dir);
            for check in config_report.checks {
                overall_report.add(check);
            }
            println!();
        }
    }

    // 5. Validate critical symlinks
    if let Some(home) = dirs::home_dir() {
        let dotfiles_dir = home.join("dotfiles");
        if dotfiles_dir.exists() {
            println!("{}", "Checking critical symlinks...".bold());
            let critical_symlinks_report =
                validate::symlinks::validate_critical_symlinks(&home, &dotfiles_dir);
            for check in critical_symlinks_report.checks {
                overall_report.add(check);
            }
            println!();
        }
    }

    // 6. Validate .claude directory
    if let Some(home) = dirs::home_dir() {
        let dotfiles_dir = home.join("dotfiles");
        if dotfiles_dir.exists() {
            println!("{}", "Checking .claude configuration...".bold());
            let claude_report = validate::claude::validate_claude_directory(&home, &dotfiles_dir);
            for check in claude_report.checks {
                overall_report.add(check);
            }
            println!();
        }
    }

    // 7. Validate shell integration
    if let Some(home) = dirs::home_dir() {
        let dotfiles_dir = home.join("dotfiles");
        if dotfiles_dir.exists() {
            println!("{}", "Checking shell integration...".bold());
            let shell_report = validate::shell::validate_shell_integration(&home, &dotfiles_dir);
            for check in shell_report.checks {
                overall_report.add(check);
            }
            println!();
        }
    }

    // 8. Validate iTerm2 configuration (macOS only)
    #[cfg(target_os = "macos")]
    if let Some(home) = dirs::home_dir() {
        let dotfiles_dir = home.join("dotfiles");
        if dotfiles_dir.exists() {
            println!("{}", "Checking iTerm2 configuration...".bold());
            let iterm_report = validate::iterm::validate_iterm_config(&dotfiles_dir);
            for check in iterm_report.checks {
                overall_report.add(check);
            }
            println!();
        }
    }

    // Print formatted report
    println!("{}", overall_report.format_colored());

    // Exit with error code if there are errors
    if overall_report.has_errors() {
        std::process::exit(1);
    }

    Ok(())
}
