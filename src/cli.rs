use clap::{clap_derive::Args, command, Parser, Subcommand};

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
}

#[derive(Debug, Args)]
pub struct ParseArgs {
    #[arg(name = "commit", help = "Conventional commit message to parse")]
    pub commit: String,
}

#[derive(Debug, Args)]
pub struct ChangelogInitArgs {}

#[derive(Debug, Subcommand)]
pub enum ChangelogCommands {
    /// Initializes changelog generation using the 'change' tool
    Init(ChangelogInitArgs),
}

#[derive(Debug, Args)]
pub struct ChangelogArgs {
    #[command(subcommand)]
    pub command: ChangelogCommands,
}

#[derive(Debug, Args)]
pub struct InitArgs {}
