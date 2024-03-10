use clap::Parser;
use convy::validation::CommitMessageError;

fn main() -> Result<(), CommitMessageError> {
    let cli = convy::cli::Cli::parse();

    match cli.commands {
        convy::cli::Commands::Parse(arg) => {
            let tokenizer = convy::lexer::LexicalTokenizer::new();

            let tokens = tokenizer.tokenize(&arg.commit);

            convy::validation::validate_commit_message(&arg.commit, tokens)?;
            Ok(())
        }
    }
}
