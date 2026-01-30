use crate::core::{config::Config, prompt};
use crate::error::Result;
use crate::{install, language, symlink};
use colored::Colorize;
use dialoguer::{Confirm, MultiSelect};

/// Runs the interactive setup command
pub fn run(dry_run: bool) -> Result<()> {
    println!("{}", "🚀 Interactive Dotfiles Setup".bold());
    println!();

    if dry_run {
        println!("{}", "🔍 DRY-RUN MODE (no changes will be made)".yellow());
        println!();
    }

    // Step 1: Prompt for configuration
    println!("{}", "📝 Configuration".bold().underline());
    println!();

    let dotfiles_dir = prompt::prompt_dotfiles_dir()?;
    let xdg_config_home = prompt::prompt_xdg_config_home()?;
    let language_manager = prompt::prompt_language_manager()?;

    // Step 2: Language selection
    println!();
    println!("{}", "🔧 Language Selection".bold().underline());
    println!("Select languages to install (Space to select, Enter to continue):");
    println!();

    let available_languages = language::all_languages();
    let language_names: Vec<String> = available_languages
        .iter()
        .map(|l| format!("{} ({})", l.display_name(), l.default_version()))
        .collect();

    let selections = MultiSelect::new()
        .items(&language_names)
        .interact()
        .map_err(|e| crate::error::DotfilesError::Config(format!("Prompt error: {}", e)))?;

    let selected_languages: Vec<_> = selections
        .iter()
        .map(|&i| available_languages[i].language_name().to_string())
        .collect();

    // Step 3: Show summary and confirm
    println!();
    println!("{}", "📋 Setup Summary".bold().underline());
    println!("  Dotfiles directory: {}", dotfiles_dir.display().to_string().cyan());
    println!("  XDG config home: {}", xdg_config_home.display().to_string().cyan());
    println!("  Language manager: {}", format!("{:?}", language_manager).cyan());

    if selected_languages.is_empty() {
        println!("  Languages: {}", "None selected".yellow());
    } else {
        println!("  Languages:");
        for lang in &selected_languages {
            println!("    - {}", lang.cyan());
        }
    }

    println!();

    if !dry_run {
        let confirmed = Confirm::new()
            .with_prompt("Proceed with setup?")
            .default(true)
            .interact()
            .map_err(|e| crate::error::DotfilesError::Config(format!("Prompt error: {}", e)))?;

        if !confirmed {
            println!("{}", "Setup cancelled".yellow());
            return Ok(());
        }
    }

    // Step 4: Execute setup
    println!();
    println!("{}", "🔨 Starting setup...".bold());
    println!();

    // 4a. Install Homebrew (macOS only)
    if cfg!(target_os = "macos") {
        println!("{}", "Checking Homebrew...".bold());
        if !install::homebrew::is_installed() {
            if dry_run {
                println!("{}", "  Would install Homebrew".yellow());
            } else {
                install::homebrew::install()?;
            }
        } else {
            println!("{}", "  ✓ Homebrew already installed".green());
        }
        println!();
    }

    // 4b. Install version manager
    println!("{}", "Checking version manager...".bold());
    if install::version_manager::detect().is_none() {
        if dry_run {
            println!("{}", "  Would install version manager".yellow());
        } else {
            install::version_manager::install_preferred()?;
        }
    } else {
        let vm = install::version_manager::detect().unwrap();
        println!("{}", format!("  ✓ {} already installed", vm.display_name()).green());
    }
    println!();

    // 4c. Install essential packages
    println!("{}", "Installing essential packages...".bold());
    if dry_run {
        println!("{}", "  Would install packages: stow, fzf, bat, fd, tree, nvim, tmux".yellow());
    } else {
        let status = install::packages::package_status();
        if !status.is_complete() {
            install::packages::install_essential_packages()?;
        } else {
            println!("{}", "  ✓ All essential packages already installed".green());
        }
    }
    println!();

    // 4d. Install selected languages
    if !selected_languages.is_empty() {
        println!("{}", "Installing languages...".bold());

        if dry_run {
            for lang in &selected_languages {
                println!("{}", format!("  Would install {}", lang).yellow());
            }
        } else {
            if let Some(vm) = install::version_manager::detect() {
                for lang_name in &selected_languages {
                    if let Some(installer) = language::get_installer(lang_name) {
                        println!("  Installing {}...", installer.display_name());
                        match installer.install(vm, None) {
                            Ok(()) => println!("{}", format!("    ✓ {} installed", installer.display_name()).green()),
                            Err(e) => println!("{}", format!("    ✗ Failed: {}", e).red()),
                        }
                    }
                }
            } else {
                println!("{}", "  ⚠ No version manager available, skipping language installation".yellow());
            }
        }
        println!();
    }

    // 4e. Create symlinks
    println!("{}", "Creating symlinks...".bold());
    if dry_run {
        println!("{}", "  Would create symlinks from dotfiles to home".yellow());
    } else {
        // Determine which symlinker to use
        let status = install::packages::package_status();
        let has_stow = status.installed_essential.iter().any(|p| p == "stow");

        let symlinker: Box<dyn symlink::Symlinker> = if has_stow {
            println!("  Using GNU Stow");
            Box::new(symlink::stow::StowSymlinker::new())
        } else {
            println!("  Using manual symlinks");
            Box::new(symlink::manual::ManualSymlinker::new())
        };

        let home = dirs::home_dir().unwrap();
        match symlinker.symlink(&dotfiles_dir, &home) {
            Ok(report) => {
                println!("{}", format!("  ✓ {}", report.summary()).green());
            }
            Err(e) => {
                println!("{}", format!("  ✗ Error creating symlinks: {}", e).red());
            }
        }
    }
    println!();

    // Step 5: Save configuration
    if !dry_run {
        println!("{}", "Saving configuration...".bold());
        let config = Config {
            dotfiles_dir,
            xdg_config_home,
            language_manager,
            symlink_method: crate::core::config::SymlinkMethod::Stow,
            install_oh_my_zsh: false,
        };

        let config_path = dirs::home_dir().unwrap().join(".dotfiles.conf");
        config.save(&config_path)?;
        println!("{}", format!("  ✓ Configuration saved to {}", config_path.display()).green());
        println!();
    }

    // Step 6: Post-install instructions
    println!();
    println!("{}", "✅ Setup Complete!".bold().green());
    println!();
    println!("{}", "📝 Next Steps:".bold());
    println!("  1. Restart your shell or run: source ~/.zshrc");
    println!("  2. Verify installation: dotfiles doctor");
    println!("  3. Configure additional tools manually:");
    println!("     - iTerm2 preferences");
    println!("     - GitHub CLI: gh auth login");
    println!();

    Ok(())
}
