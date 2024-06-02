use clap::Parser;

fn main() -> Result<(), String> {
    let cli = convy::cli::Cli::parse();

    match cli.commands {
        convy::cli::Commands::Parse(arg) => {
            let tokenizer = convy::lexer::LexicalTokenizer::new();

            let tokens = tokenizer.tokenize(&arg.commit);

            match convy::validation::validate_commit_message(&arg.commit, tokens) {
                Ok(_) => println!("Commit message is valid!"),
                Err(e) => {
                    eprintln!("Error: {}\n----\n", e);
                    return Err("Commit message is invalid!".to_string());
                }
            };
            Ok(())
        }
    }
}
