# Convy

**Consistent commits, made easy.**

`convy` is a tool to enforce a consistent commit message format across a project. It serves two purposes:
1.  **Validator**: A `commit-msg` hook that rejects invalid commits.
2.  **Wizard**: An interactive CLI (`convy commit`) to help you construct perfect Conventional Commits every time.

## Installation

```bash
cargo install --path .
```

## Quick Start

### 1. Initialize

Run this in your git repository:

```bash
convy init
```

This creates a `.convy.toml` config and installs the git hook.

### 2. Commit Interactively (Recommended)

Instead of typing `git commit -m ...`, use:

```bash
convy commit --run
```

This launches an interactive wizard that asks for:
- Type (feat, fix, etc.)
- Scope (optional, configurable)
- Description
- Breaking changes
- **Custom Footers** (Co-authored-by, References, etc.)

It then runs `git commit` for you.

### 3. Manual Commits

If you prefer `git commit`, the installed hook will ensure you follow the rules.

```bash
git commit -m "feat(parser): add support for emoji"
```

## Configuration

Customize behavior in `.convy.toml`:

```toml
# Add custom types (e.g. for specific workflows)
additional_types = ["wip", "security"]

# Restrict scopes (optional). If set, only these scopes are allowed.
scopes = ["core", "cli", "api", "docs"]

# Enforce BREAKING CHANGE footer for '!' commits
require_breaking_change_footer = true

# Enable Gitmoji (prepends ✨, 🐛, etc. to description)
emoji = true
```

## Commands

| Command | Description |
| :--- | :--- |
| `init` | Set up config and git hooks. |
| `commit` | Interactive commit wizard. Use `--run` to execute git commit. |
| `parse` | Validate a message string (used by hooks). |
| `changelog` | Changelog management tools. |

## License

MIT
