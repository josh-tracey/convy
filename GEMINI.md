# Convy - Project Context

`convy` is a Rust-based CLI tool designed to enforce the [Conventional Commits](https://conventionalcommits.org) specification throughout the development lifecycle. It provides three primary functionalities:
1.  **Validation**: A `commit-msg` git hook that rejects non-compliant commit messages.
2.  **Interactive Wizard**: A guided CLI (`convy commit --run`) to help developers construct perfectly formatted commits.
3.  **Changelog Management**: Automated generation and maintenance of `CHANGELOG.md` based on commit history.

## Project Overview

- **Core Technology**: Rust (2021 edition)
- **CLI Framework**: `clap` (v4)
- **Parsing Engine**: `logos` for lexing commit messages according to the Conventional Commits spec.
- **Interactivity**: `ratatui` for terminal-based prompts.
- **Configuration**: TOML-based config via `.convy.toml`.

## Architecture

- `src/main.rs`: Entry point. Handles subcommand dispatching and high-level logic for `init`, `parse`, `commit`, and `changelog`.
- `src/cli.rs`: Definitions for the CLI structure and arguments.
- `src/lexer.rs`: The core parser for Conventional Commits. It handles types, scopes, breaking changes (exclamations and footers), and body/footer extraction.
- `src/changelog.rs`: Logic for reading git logs, categorizing commits (Features, Bug Fixes, etc.), and updating `CHANGELOG.md` using regex-based section replacement.
- `src/lib.rs`: Exposes internal modules for testing and organization.
- `src/commit_msg`: A template script used to install the `commit-msg` git hook.

## Building and Running

### Development Commands
- **Build**: `cargo build`
- **Run**: `cargo run -- <COMMAND>` (e.g., `cargo run -- parse "feat: message"`)
- **Test**: `cargo test` (includes unit tests in `src/lexer.rs` and integration tests in `tests/`)
- **Install**: `cargo install --path .`

### Key CLI Commands
- `convy init`: Initializes `.convy.toml` and installs the git hook in `.git/hooks/commit-msg`.
- `convy commit --run`: Launches the interactive wizard and executes `git commit`.
- `convy parse "<MESSAGE>"`: Validates a raw commit message string.
- `convy changelog init`: Creates a new `CHANGELOG.md`.
- `convy changelog generate [--write]`: Previews or updates the [Unreleased] section of the changelog.
- `convy changelog release <VERSION>`: Tags the current [Unreleased] changes with a version and date.

## Development Conventions

- **Commit Messages**: The project enforces Conventional Commits on itself.
- **Configuration**: `.convy.toml` allows customizing allowed scopes, additional types, and whether to require breaking change footers or use emojis.
- **Testing**:
    - **Unit Tests**: Lexing and parsing logic are heavily tested within `src/lexer.rs`.
    - **Integration Tests**: Command-level behavior is tested in `tests/changelog_integration_tests.rs`.
- **Formatting**: Adheres to standard Rust formatting (`cargo fmt`).

## Configuration (`.convy.toml`)

The tool looks for a `.convy.toml` in the repository root:
```toml
additional_types = ["perf", "build"]
scopes = ["parser", "cli", "changelog"]
require_breaking_change_footer = true
emoji = false
```
