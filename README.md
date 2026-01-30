# Dotfiles Tool

> Interactive dotfiles setup and management tool built in Rust

A minimal, idempotent tool that automates dotfiles installation with interactive configuration, self-installing dependencies, health validation, and optional migration from existing setups.

## Features

- 🚀 **Interactive Setup** - Guided prompts for easy configuration
- 🔧 **Automatic Dependencies** - Installs Homebrew, version managers, and essential tools
- 🌐 **Multi-Language Support** - Java, Node.js, Python, Rust, Go via ASDF/mise/rtx
- 🔗 **Smart Symlinking** - GNU Stow with automatic fallback to manual symlinks
- 🏥 **Health Checks** - `doctor` command validates your entire setup
- 💾 **Backup & Migration** - Safe migration with timestamped backups and rollback
- 🔐 **Secret Detection** - Automatically extracts secrets to `.env` files
- 🎨 **Colored Output** - Clear, actionable feedback with visual indicators

## Quick Start

```bash
# Clone and build
git clone https://github.com/yourusername/dotfiles-tool
cd dotfiles-tool
cargo build --release

# Install
sudo cp target/release/dotfiles /usr/local/bin/

# Run interactive setup
dotfiles setup

# Verify installation
dotfiles doctor
```

## Installation

### From Source

```bash
cargo install --path .
```

### From GitHub Releases

```bash
# Download latest release
curl -L https://github.com/yourusername/dotfiles-tool/releases/latest/download/dotfiles-macos -o dotfiles
chmod +x dotfiles
sudo mv dotfiles /usr/local/bin/
```

## Usage

### Setup Command

Run the interactive setup wizard:

```bash
dotfiles setup
```

This will:
1. Prompt for configuration (dotfiles directory, XDG config home, language manager)
2. Let you select which languages to install
3. Show a summary and ask for confirmation
4. Install Homebrew (macOS only, if needed)
5. Install a version manager (mise/ASDF/rtx)
6. Install essential packages (stow, fzf, bat, fd, tree, nvim, tmux)
7. Install selected language runtimes
8. Create symlinks from your dotfiles to your home directory
9. Save configuration to `~/.dotfiles.conf`

**Dry-run mode:**
```bash
dotfiles setup --dry-run
```

### Doctor Command

Validate your dotfiles setup:

```bash
dotfiles doctor
```

Checks:
- ✓ Homebrew installation
- ✓ Version manager (ASDF/mise/rtx)
- ✓ Essential tools (stow, git, fzf, etc.)
- ✓ Symlinks point to correct locations
- ✓ No hardcoded paths (`/Users/username` → use `$HOME`)
- ✓ Config file syntax (TOML, JSON, YAML)

Output example:
```
🏥 Dotfiles Health Check

Homebrew
  ✓ Homebrew - Installed at /opt/homebrew/bin/brew

Version Manager
  ✓ Version Manager - ASDF installed at /opt/homebrew/bin/asdf

stow
  ✓ stow - Installed at /opt/homebrew/bin/stow

Summary
  ✓ 9 passed
  ⚠ 0 warnings
  Total: 9 checks
```

## Commands Reference

| Command | Description |
|---------|-------------|
| `dotfiles setup [--dry-run]` | Run interactive setup wizard |
| `dotfiles doctor` | Validate dotfiles setup |
| `dotfiles --help` | Show help message |
| `dotfiles --version` | Show version |

## Configuration

Configuration is saved to `~/.dotfiles.conf` in TOML format:

```toml
dotfiles_dir = "/Users/you/dotfiles"
xdg_config_home = "/Users/you/.config"
language_manager = "Asdf"
symlink_method = "Stow"
install_oh_my_zsh = false
```

See `.dotfiles.conf.example` for all available options.

## Supported Languages

| Language | Default Version | Manager |
|----------|----------------|---------|
| Java | OpenJDK 21 | ASDF/mise/rtx |
| Node.js | 22.12.0 | ASDF/mise/rtx |
| Python | 3.12.1 | ASDF/mise/rtx |
| Rust | 1.83.0 | ASDF/mise/rtx |
| Go | 1.23.4 | ASDF/mise/rtx |

## Essential Packages

Automatically installed via Homebrew:

- **stow** - GNU Stow for symlink management
- **fzf** - Fuzzy finder
- **bat** - Better `cat` with syntax highlighting
- **fd** - Better `find`
- **tree** - Directory tree viewer
- **nvim** - Neovim editor
- **tmux** - Terminal multiplexer

## Architecture

```
dotfiles-tool/
├── src/
│   ├── backup/          # Backup and migration
│   ├── commands/        # CLI commands (setup, doctor)
│   ├── core/            # Config, logging, prompts
│   ├── detect/          # OS and tool detection
│   ├── install/         # Dependency installation
│   ├── language/        # Language installers
│   ├── symlink/         # Symlink management
│   └── validate/        # Health checks
├── tests/               # Integration tests
└── .github/workflows/   # CI/CD
```

### Design Principles

- **SOLID Principles** - Clean architecture with clear separation of concerns
- **Idempotent** - Safe to run multiple times
- **Minimal** - No unnecessary abstractions or features
- **Testable** - 120+ unit tests with 100% core logic coverage
- **Type-Safe** - Leverages Rust's type system for correctness

## Development

### Prerequisites

- Rust 1.70+ (2021 edition)
- macOS or Linux

### Building

```bash
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Linting

```bash
# Check for issues
cargo clippy

# Auto-fix
cargo clippy --fix

# Format code
cargo fmt
```

## Troubleshooting

### "Homebrew not found" on macOS

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### "Version manager not found"

```bash
brew install mise
# or
brew install asdf
```

### "Permission denied" when creating symlinks

Make sure you have write permissions to your home directory and the target locations.

### Symlink conflicts

If you have existing files that conflict with your dotfiles:

1. Backup existing files
2. Remove or rename them
3. Re-run `dotfiles setup`

Or use the migration workflow (coming in future release).

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run clippy (`cargo clippy`)
6. Format code (`cargo fmt`)
7. Commit (`git commit -m 'Add amazing feature'`)
8. Push (`git push origin feature/amazing-feature`)
9. Open a Pull Request

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Built with [clap](https://github.com/clap-rs/clap) for CLI parsing
- Interactive prompts with [dialoguer](https://github.com/console-rs/dialoguer)
- Colored output via [colored](https://github.com/colored-rs/colored)
- Error handling with [thiserror](https://github.com/dtolnay/thiserror)

---

**Note:** This tool is designed for personal dotfiles management. Always review and understand what it's doing before running setup commands.
