use clap::{clap_derive::Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init(InitArgs),
    Parse(ParseArgs),
    Changelog(ChangelogArgs),
    Commit(CommitArgs),
}

#[derive(Debug, Args)]
pub struct CommitArgs {
    #[arg(short, long, help = "Run git commit after generating message")]
    pub run: bool,
}

#[derive(Debug, Args)]
pub struct ParseArgs {
    #[arg(name = "commit", help = "Conventional commit message to parse")]
    pub commit: String,
}

#[derive(Debug, Args)]
pub struct ChangelogInitArgs {}

#[derive(Debug, Args)]
pub struct ChangelogGenerateArgs {
    #[arg(short, long, help = "Write output to CHANGELOG.md (replaces/inserts Unreleased section)")]
    pub write: bool,
    
    #[arg(short, long, help = "Include all commits from the beginning, ignoring tags")]
    pub all: bool,
}

#[derive(Debug, Args)]
pub struct ChangelogReleaseArgs {
    #[arg(help = "The version to release (e.g. 1.0.0)")]
    pub version: String,
}

#[derive(Debug, Subcommand)]
pub enum ChangelogCommands {
    /// Initialize a CHANGELOG.md file
    Init(ChangelogInitArgs),
    /// Generate changelog content from git commits
    Generate(ChangelogGenerateArgs),
    /// Promote Unreleased changes to a specific version
    Release(ChangelogReleaseArgs),
}

#[derive(Debug, Args)]
pub struct ChangelogArgs {
    #[command(subcommand)]
    pub command: ChangelogCommands,
}

#[derive(Debug, Args)]
pub struct InitArgs {}
