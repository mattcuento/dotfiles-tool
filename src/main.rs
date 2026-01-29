use anyhow::Result;
use clap::{Parser, Subcommand};

mod core;
mod detect;
mod error;

#[derive(Parser)]
#[command(name = "dotfiles")]
#[command(about = "Interactive dotfiles setup and management")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run interactive setup
    Setup {
        #[arg(long)]
        dry_run: bool,
    },
    /// Validate all configurations
    Doctor,
    /// Migrate existing configs
    Migrate,
    /// Create backup
    Backup,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup { dry_run } => {
            if dry_run {
                core::logger::log_info("Running in DRY-RUN mode (no changes will be made)");
            }
            core::logger::log_info("Setup command (placeholder)");
            Ok(())
        }
        Commands::Doctor => {
            core::logger::log_info("Doctor command (placeholder)");
            Ok(())
        }
        Commands::Migrate => {
            core::logger::log_info("Migrate command (placeholder)");
            Ok(())
        }
        Commands::Backup => {
            core::logger::log_info("Backup command (placeholder)");
            Ok(())
        }
    }
}
