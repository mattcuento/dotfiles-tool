use crate::error::Result;
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

    // Print formatted report
    println!("{}", overall_report.format_colored());

    // Exit with error code if there are errors
    if overall_report.has_errors() {
        std::process::exit(1);
    }

    Ok(())
}
