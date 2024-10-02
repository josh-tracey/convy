use std::fs;
use toml;

use clap::Parser;
use convy::lexer::{default_config, parse_commit_message};

fn main() -> Result<(), String> {
    let cli = convy::cli::Cli::parse();

    match cli.commands {
        convy::cli::Commands::Parse(arg) => {
            let mut config_file = fs::read_to_string(".convy.toml").unwrap_or("".to_string());

            if config_file.is_empty() {
                let default_config_str =
                    toml::to_string(&default_config()).expect("Error creating default config");
                fs::write(".convy.toml", default_config_str)
                    .expect("Error writing default config to file");
            }

            config_file = fs::read_to_string(".convy.toml").expect("Error reading config file");

            let config = toml::from_str(&config_file).expect("Error parsing TOML");

            match parse_commit_message(&arg.commit, config) {
                Ok(_) => {
                    println!("Commit message is valid!");
                }
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                }
            };

            Ok(())
        }
    }
}
