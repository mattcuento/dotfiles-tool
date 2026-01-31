use crate::core::prompt;
use crate::error::Result;
use crate::install;
use colored::Colorize;
use dialoguer::{Confirm, Input};

/// Default dotfiles repository URL (should be configured by user)
const DEFAULT_DOTFILES_REPO: &str = "https://github.com/YOUR_USERNAME/dotfiles.git";

/// Runs the init/bootstrap command for first-time setup
pub fn run() -> Result<()> {
    println!("{}", "🌟 Dotfiles Bootstrap".bold());
    println!();
    println!("This will set up your dotfiles on a fresh system.");
    println!();

    // Step 1: Prompt for dotfiles repository URL
    let repo_url: String = Input::new()
        .with_prompt("Dotfiles repository URL")
        .default(DEFAULT_DOTFILES_REPO.to_string())
        .interact_text()
        .map_err(|e| crate::error::DotfilesError::Config(format!("Prompt error: {}", e)))?;

    // Step 2: Prompt for target directory
    let target_dir = prompt::prompt_dotfiles_dir()?;

    // Step 3: Confirm
    println!();
    println!("{}", "📋 Bootstrap Summary".bold().underline());
    println!("  Repository: {}", repo_url.cyan());
    println!(
        "  Target directory: {}",
        target_dir.display().to_string().cyan()
    );
    println!();

    let confirmed = Confirm::new()
        .with_prompt("Clone dotfiles repository?")
        .default(true)
        .interact()
        .map_err(|e| crate::error::DotfilesError::Config(format!("Prompt error: {}", e)))?;

    if !confirmed {
        println!("{}", "Bootstrap cancelled".yellow());
        return Ok(());
    }

    // Step 4: Clone dotfiles repository
    println!();
    println!("{}", "📥 Cloning dotfiles repository...".bold());
    install::repos::clone_dotfiles_repo(&target_dir, &repo_url)?;

    println!();
    println!("{}", "✓ Bootstrap complete!".green().bold());
    println!();
    println!("Next steps:");
    println!(
        "  1. Review configuration files in {}",
        target_dir.display()
    );
    println!("  2. Run: {} to complete setup", "dotfiles setup".cyan());

    Ok(())
}
