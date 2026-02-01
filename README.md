# Convy

**Consistent commits, made easy.**

`convy` is a tool to enforce a consistent commit message format across a project. It serves two purposes:
1.  **Validator**: A `commit-msg` hook that rejects invalid commits.
2.  **Wizard**: An interactive CLI (`convy commit`) to help you construct perfect Conventional Commits every time.
3.  **Changelog**: Automated changelog generation based on your commit history.

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

### 3. Generate Changelog

Initialize a changelog:

```bash
convy changelog init
```

Preview changes since the last tag:

```bash
convy changelog generate
```

Write them to `CHANGELOG.md` under [Unreleased]:

```bash
convy changelog generate --write
```

Release a version (move [Unreleased] to [1.0.0]):

```bash
convy changelog release 1.0.0
```

## Commands

| Command | Description |
| :--- | :--- |
| `init` | Set up config and git hooks. |
| `commit` | Interactive commit wizard. Use `--run` to execute git commit. |
| `parse` | Validate a message string (used by hooks). |
| `changelog` | Initialize, generate, and release changelogs. |

## License

MIT
