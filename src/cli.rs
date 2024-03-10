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
    Parse(ParseArgs),
}

#[derive(Debug, Args)]
pub struct ParseArgs {
    #[arg(name = "commit", help = "Conventional commit message to parse")]
    pub commit: String,
}
