# Dotfiles Tool Implementation Plan

## Overview

Building a minimal, idempotent Rust-based setup tool that automates dotfiles installation with interactive configuration, self-installing dependencies, health validation (`doctor` command), and optional migration from existing setups with backup capability.

## Architecture Decisions

### Language: Rust

**Rationale:**
- **Learning opportunity**: Perfect project complexity for deepening Rust skills
- **Excellent ecosystem**: clap, dialoguer, serde provide all needed functionality
- **Single binary**: No runtime dependencies, easy distribution via GitHub releases
- **Type safety**: Compiler catches bugs, Result<T,E> forces proper error handling
- **Long-term maintainability**: Safe refactoring, clear error messages
- **Production quality**: Memory safe, fast execution, reliable tooling

**Trade-offs accepted:**
- Longer initial development (~35-50 hours vs ~26-35 for Go)
- Learning curve with borrow checker (but valuable skill development)
- Compilation step required (but cargo is excellent)

### Language Support

Support 5 languages with multiple version managers:
- **Languages**: Java, JavaScript (Node.js), Python, Rust, Go
- **Version managers**: ASDF, mise, rtx (auto-detect which is installed)
- Graceful degradation if no manager installed

### Configuration

- **XDG_CONFIG_HOME**: Configurable during setup (default: `~/.config`)
- **User preferences**: Stored in `~/.dotfiles.conf` (not in repo)
- **Migration scope**: Only configs currently in repo (zsh, nvim, tmux, git, etc.)

## Project Structure

```
dotfiles-tool/
├── Cargo.toml                   # Project manifest and dependencies
├── Cargo.lock                   # Locked dependency versions
├── PLAN.md                      # This file - implementation plan and progress
├── README.md                    # User-facing documentation
├── .gitignore                   # Git ignore rules
├── src/
│   ├── main.rs                  # CLI entry point (clap commands)
│   ├── lib.rs                   # Public library API
│   ├── error.rs                 # Custom error types (thiserror)
│   ├── commands/                # Command implementations
│   │   ├── mod.rs
│   │   ├── setup.rs             # Interactive setup command
│   │   ├── doctor.rs            # Health validation command
│   │   ├── migrate.rs           # Migration command
│   │   └── backup.rs            # Backup command
│   ├── core/                    # Core utilities
│   │   ├── mod.rs
│   │   ├── config.rs            # Configuration management (serde)
│   │   ├── logger.rs            # Colored logging utilities
│   │   └── prompt.rs            # Interactive prompts (dialoguer)
│   ├── detect/                  # Environment detection
│   │   ├── mod.rs
│   │   ├── os.rs                # OS detection
│   │   ├── tools.rs             # Tool availability checks
│   │   └── conflicts.rs         # Conflict detection
│   ├── install/                 # Dependency installation
│   │   ├── mod.rs
│   │   ├── homebrew.rs          # Homebrew installation
│   │   ├── version_manager.rs   # ASDF/mise/rtx detection & install
│   │   └── packages.rs          # Package management
│   ├── language/                # Language tooling
│   │   ├── mod.rs
│   │   ├── java.rs
│   │   ├── javascript.rs
│   │   ├── python.rs
│   │   ├── rust.rs
│   │   └── go.rs
│   ├── symlink/                 # Symlink management
│   │   ├── mod.rs
│   │   ├── stow.rs              # GNU Stow integration
│   │   └── manual.rs            # Manual symlink creation
│   └── validate/                # Health checks
│       ├── mod.rs
│       ├── dependencies.rs      # Dependency checks
│       ├── symlinks.rs          # Symlink validation
│       ├── configs.rs           # Config syntax validation
│       └── paths.rs             # Hardcoded path detection
├── tests/                       # Integration tests
│   ├── integration_test.rs
│   └── fixtures/                # Test data
└── .dotfiles.conf.example       # TOML configuration template
```

## Key Dependencies

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }  # CLI framework
dialoguer = "0.11"                                  # Interactive prompts
colored = "2.1"                                     # Colored terminal output
serde = { version = "1.0", features = ["derive"] }  # Serialization
toml = "0.8"                                        # TOML parsing
anyhow = "1.0"                                      # Error propagation
thiserror = "1.0"                                   # Custom error types
walkdir = "2.5"                                     # Directory traversal
glob = "0.3"                                        # Pattern matching
dirs = "5.0"                                        # Standard directories
chrono = "0.4"                                      # Timestamps
regex = "1.10"                                      # Pattern matching

[dev-dependencies]
assert_cmd = "2.0"                                  # CLI testing
predicates = "3.1"                                  # Test assertions
tempfile = "3.10"                                   # Temporary directories
```

## SOLID Principles Application

**Single Responsibility:**
- Each module (detect, install, validate, etc.) has one clear purpose
- Functions focused on single tasks

**Open/Closed:**
- Easy to add new tools without modifying core logic
- Language installation abstracted (add new language without changing structure)
- Version manager support extensible

**Liskov Substitution:**
- Symlink methods (stow vs manual) interchangeable
- Version managers (ASDF, mise, rtx) swappable via trait

**Interface Segregation:**
- Doctor command has separate checks for different concerns
- Not one monolithic validation function

**Dependency Inversion:**
- Configuration-driven (no hardcoded paths)
- Abstract installation behind traits/functions
- Easy to mock for testing

---

## Implementation Phases

### ✅ Phase 1: Core Infrastructure & Project Setup (COMPLETE)

**Goal**: Set up Rust project, core types, and basic infrastructure

**Status**: ✅ Complete (Committed: f82a9db)

**Completed Tasks:**
- [x] Initialize Cargo project with all dependencies
- [x] Create module structure (src/commands/, src/core/, src/detect/, etc.)
- [x] Define core error types with thiserror
- [x] Implement logging utilities with colored output
- [x] Create configuration types (Config, LanguageManager, SymlinkMethod)
- [x] Add interactive prompt functions (dialoguer)
- [x] Implement OS detection (macOS, Linux, Unknown)
- [x] Add tool detection (is_installed, get_tool_path)
- [x] Create conflict detection for existing dotfiles
- [x] Build CLI skeleton with clap (setup, doctor, migrate, backup commands)
- [x] Write unit tests (5 tests passing)

**Files Created:**
- `Cargo.toml`, `.gitignore`
- `src/main.rs`, `src/lib.rs`, `src/error.rs`
- `src/core/{mod.rs, logger.rs, config.rs, prompt.rs}`
- `src/detect/{mod.rs, os.rs, tools.rs, conflicts.rs}`

**Testing:**
- ✅ All unit tests passing (5 tests)
- ✅ Config roundtrip test
- ✅ Tool detection tests
- ✅ Conflict detection test
- ✅ OS detection test

---

### Phase 2: Installation Modules (TODO)

**Goal**: Install Homebrew, version managers, packages, and languages

**Estimated Time**: 8-12 hours

**Tasks:**
- [ ] Create `src/install/` module structure
- [ ] Implement Homebrew installation and detection
- [ ] Implement version manager detection (ASDF/mise/rtx)
- [ ] Implement version manager installation
- [ ] Create LanguageInstaller trait
- [ ] Implement 5 language installers (Java, JavaScript, Python, Rust, Go)
- [ ] Implement package installation (stow, fzf, bat, fd, tree, nvim, tmux)
- [ ] Write comprehensive tests with mocking

**Files to Create:**
- `src/install/mod.rs`
- `src/install/homebrew.rs`
- `src/install/version_manager.rs`
- `src/install/packages.rs`
- `src/language/mod.rs`
- `src/language/{java,javascript,python,rust,go}.rs`

**Key Features:**
- Idempotent installation (check before installing)
- Graceful fallback if version manager not available
- Support for multiple version managers

---

### Phase 3: Symlink Management (TODO)

**Goal**: Create symlinks and handle conflicts

**Estimated Time**: 4-6 hours

**Tasks:**
- [ ] Create `src/symlink/` module structure
- [ ] Define Symlinker trait
- [ ] Implement GNU Stow integration
- [ ] Implement manual symlink creation
- [ ] Add conflict detection and reporting
- [ ] Handle existing symlinks gracefully
- [ ] Write comprehensive symlink tests

**Files to Create:**
- `src/symlink/mod.rs`
- `src/symlink/stow.rs`
- `src/symlink/manual.rs`

**Key Features:**
- SymlinkStatus enum (Created, AlreadyExists, Conflict)
- Automatic parent directory creation
- Symlink verification

---

### Phase 4: Validation (Doctor Command) (TODO)

**Goal**: Comprehensive health checks and validation

**Estimated Time**: 6-8 hours

**Tasks:**
- [ ] Create `src/validate/` module structure
- [ ] Implement CheckResult framework (Pass/Warn/Error)
- [ ] Add dependency validation (homebrew, version manager, tools)
- [ ] Add symlink validation
- [ ] Implement hardcoded path detection with regex
- [ ] Add config syntax validation
- [ ] Implement colored output for results
- [ ] Write tests for all validation types
- [ ] Implement `doctor` command in `src/commands/doctor.rs`

**Files to Create:**
- `src/validate/mod.rs`
- `src/validate/dependencies.rs`
- `src/validate/symlinks.rs`
- `src/validate/configs.rs`
- `src/validate/paths.rs`
- `src/commands/doctor.rs`

**Key Features:**
- Colored output (green/yellow/red)
- Actionable suggestions for failures
- Summary statistics
- Exit code 1 on errors

---

### Phase 5: Backup and Migration (TODO)

**Goal**: Safe backup and migration with rollback capability

**Estimated Time**: 5-7 hours

**Tasks:**
- [ ] Create backup functionality with timestamps
- [ ] Implement secret extraction (TOKEN, KEY, PASSWORD patterns)
- [ ] Create migration workflow
- [ ] Implement rollback functionality
- [ ] Add conflict resolution
- [ ] Write backup/restore tests
- [ ] Implement `backup` and `migrate` commands

**Files to Create:**
- `src/commands/backup.rs`
- `src/commands/migrate.rs`

**Key Features:**
- Timestamped backups (`~/.dotfiles-backup-YYYYMMDD-HHMMSS`)
- Secret detection and extraction
- Interactive conflict resolution
- Rollback to last backup

---

### Phase 6: Interactive Setup Command (TODO)

**Goal**: Bring all pieces together in interactive setup workflow

**Estimated Time**: 6-8 hours

**Tasks:**
- [ ] Wire all modules together in setup command
- [ ] Implement interactive workflow
- [ ] Add progress indicators
- [ ] Create post-install instructions
- [ ] Implement dry-run mode
- [ ] Add language selection with MultiSelect
- [ ] Save configuration to `~/.dotfiles.conf`
- [ ] Write integration tests

**Files to Create:**
- `src/commands/mod.rs`
- `src/commands/setup.rs`

**Key Features:**
- Interactive prompts for all configuration
- Installation preview/confirmation
- Dry-run mode (no changes)
- Manual post-install instructions (iTerm2, gh auth, etc.)

---

### Phase 7: Testing and Polish (TODO)

**Goal**: Comprehensive test coverage and final polish

**Estimated Time**: 4-6 hours

**Tasks:**
- [ ] Create integration test suite
- [ ] Add CLI testing with assert_cmd
- [ ] Create test fixtures
- [ ] Write end-to-end tests
- [ ] Set up GitHub Actions CI/CD
- [ ] Write comprehensive README
- [ ] Create `.dotfiles.conf.example`
- [ ] Add inline documentation
- [ ] Run clippy and fix warnings
- [ ] Final bug fixes

**Files to Create:**
- `tests/integration_test.rs`
- `tests/common/mod.rs`
- `tests/fixtures/`
- `.github/workflows/test.yml`
- `README.md`
- `.dotfiles.conf.example`

**CI/CD:**
```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --verbose
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```

---

## Timeline Estimate

**Total Estimated Time**: ~44-62 hours

| Phase | Description | Estimated Time | Status |
|-------|-------------|----------------|--------|
| Phase 1 | Core Infrastructure & Project Setup | 6-8 hours | ✅ Complete |
| Phase 2 | Installation Modules | 8-12 hours | ⏳ TODO |
| Phase 3 | Symlink Management | 4-6 hours | ⏳ TODO |
| Phase 4 | Validation (Doctor Command) | 6-8 hours | ⏳ TODO |
| Phase 5 | Backup and Migration | 5-7 hours | ⏳ TODO |
| Phase 6 | Interactive Setup Command | 6-8 hours | ⏳ TODO |
| Phase 7 | Testing and Polish | 4-6 hours | ⏳ TODO |

**Breakdown by experience level:**
- **Rust beginner**: ~55-62 hours (more time fighting borrow checker)
- **Rust intermediate** (current level): ~44-50 hours
- **Rust expert**: ~35-40 hours

---

## Commands Overview

```bash
# Run interactive setup wizard
dotfiles setup [--dry-run]

# Validate all configurations
dotfiles doctor

# Migrate existing configs with backup
dotfiles migrate

# Create timestamped backup
dotfiles backup

# Get help
dotfiles --help
```

---

## Distribution Strategy

**Building the tool:**
```bash
cd ~/Development/dotfiles-tool
cargo build --release
```

Binary location: `target/release/dotfiles`

**Installation options:**

1. **Copy to PATH:**
```bash
sudo cp target/release/dotfiles /usr/local/bin/
```

2. **Create alias in dotfiles repo:**
```bash
ln -s ~/Development/dotfiles-tool/target/release/dotfiles ~/Development/dotfiles/bin/dotfiles
```

3. **GitHub Releases** (future):
   - Use GitHub Actions to build binaries for macOS (ARM + Intel)
   - Attach to releases
   - Users download and install

---

## Success Criteria

**Core Functionality:**
- [ ] Setup tool runs without errors
- [ ] Interactive questions work with sensible defaults
- [ ] All dependencies install correctly
- [ ] Symlinks created (stow or manual)
- [ ] Configuration file (`~/.dotfiles.conf`) saved

**Language Support:**
- [ ] Java, Node, Python, Rust, Go all installable
- [ ] Multiple version managers supported (ASDF, mise, rtx)
- [ ] Graceful fallback if no version manager

**Doctor Command:**
- [ ] Detects all issues (hardcoded paths, missing tools, broken symlinks)
- [ ] Color-coded output (green/yellow/red)
- [ ] Actionable suggestions for failures

**Migration:**
- [ ] Creates timestamped backups
- [ ] Extracts secrets to separate file
- [ ] Handles conflicts interactively
- [ ] Rollback works correctly

**Testing:**
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Manual end-to-end testing successful

**Documentation:**
- [ ] README updated with Quick Start
- [ ] All CLI commands documented
- [ ] Troubleshooting section added
- [ ] Language tooling explained

---

## Development Workflow

**Daily Development:**
```bash
# Start of work session
cd ~/Development/dotfiles-tool

# Implement feature
# ... edit files ...

# Run tests frequently
cargo test

# Check for common issues
cargo clippy

# Format code
cargo fmt

# Build and test binary
cargo build --release
./target/release/dotfiles --help

# End of session: commit working code
git add .
git commit -m "Phase X: [description]"
```

**Checkpoint Strategy:**

After each phase:
- [ ] All unit tests pass (`cargo test`)
- [ ] Clippy has no warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Manual testing of new functionality
- [ ] Git commit with clear message
- [ ] Move to next phase

---

## Current Status

**Phase 1**: ✅ Complete
- All core infrastructure in place
- 5 unit tests passing
- CLI skeleton functional
- Ready to begin Phase 2

**Next Steps:**
- Begin Phase 2: Installation Modules
- Implement Homebrew installation
- Add version manager detection and installation
- Create language installer trait and implementations

---

## Notes

- This is a learning project - embrace compiler errors as teaching moments
- Focus on working code over perfect code initially
- Refactor as patterns emerge
- Test early and often
- Keep commits small and focused
