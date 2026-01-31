use clap::{Parser, Subcommand};
use dotfiles::commands;
use dotfiles::Result;

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
    /// Bootstrap dotfiles on a fresh system
    Init,
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
        Commands::Init => commands::init(),
        Commands::Setup { dry_run } => commands::setup(dry_run),
        Commands::Doctor => commands::doctor(),
        Commands::Migrate => {
            println!("Migrate command (not yet implemented)");
            Ok(())
        }
        Commands::Backup => {
            println!("Backup command (not yet implemented)");
            Ok(())
        }
    }
}
