use std::fs;
use toml;

use clap::Parser;
use convy::validation::default_config;

fn main() -> Result<(), String> {
    let cli = convy::cli::Cli::parse();

    match cli.commands {
        convy::cli::Commands::Parse(arg) => {
            let tokenizer = convy::lexer::LexicalTokenizer::new();

            let tokens = tokenizer.tokenize(&arg.commit);

            let config_path = ".convy.toml";
            let config: convy::validation::Config = match fs::read_to_string(config_path) {
                Ok(config_str) => toml::from_str(&config_str).unwrap_or_else(|_| default_config()),
                Err(_) => default_config(),
            };

            match convy::validation::validate_commit_message(&arg.commit, tokens, Some(&config)) {
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
