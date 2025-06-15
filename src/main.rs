use std::{env, fs, os::unix::fs::PermissionsExt, process::Command};
use toml;

use clap::Parser;
use convy::{
    cli::ChangelogCommands,
    lexer::{default_config, parse_commit_message},
};

fn main() -> Result<(), String> {
    let cli = convy::cli::Cli::parse();

    match cli.commands {
        convy::cli::Commands::Parse(arg) => {
            let config_file = match fs::read_to_string(".convy.toml") {
                Ok(content) => content,
                Err(_) => {
                    eprintln!("Error: .convy.toml not found. Please run `convy init` to create a configuration file.");
                    std::process::exit(1);
                }
            };

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
        convy::cli::Commands::Init(_) => {
            // Check if Git is installed
            let git_version_output = Command::new("git").arg("--version").output();
            if git_version_output.is_err() || !git_version_output.unwrap().status.success() {
                eprintln!("Error: Git is not installed. Please install Git and try again.");
                std::process::exit(1);
            }

            // Check if inside a Git repository
            let git_repo_output = Command::new("git")
                .args(["rev-parse", "--is-inside-work-tree"])
                .output();
            match git_repo_output {
                Ok(output) => {
                    if !output.status.success()
                        || String::from_utf8_lossy(&output.stdout).trim() != "true"
                    {
                        eprintln!("Error: Not inside a Git repository. Please run `convy init` from the root of a Git repository.");
                        std::process::exit(1);
                    }
                }
                Err(_) => {
                    eprintln!("Error: Failed to check Git repository status. Make sure you are in a Git repository.");
                    std::process::exit(1);
                }
            }

            let default_config_str =
                toml::to_string(&default_config()).expect("Error creating default config");
            fs::write(".convy.toml", default_config_str)
                .expect("Error writing default config to file");

            //embed commit-msg in binary
            let commit_msg = include_str!("commit_msg");

            // write commit_msg to git hooks
            // Ensure .git/hooks directory exists
            if !fs::metadata(".git/hooks").is_ok() {
                fs::create_dir_all(".git/hooks")
                    .expect("Error creating .git/hooks directory");
            }
            fs::write(".git/hooks/commit-msg", commit_msg)
                .expect("Error writing config file to git hooks");

            // make commit-msg executable
            fs::set_permissions(".git/hooks/commit-msg", fs::Permissions::from_mode(0o755))
                .expect("Error setting permissions on commit-msg");

            println!("Successfully initialized convy!");
            println!("Please add, commit, and push the following files to your repository:");
            println!("  - .convy.toml");
            println!("  - .git/hooks/commit-msg");

            Ok(())
        }
        convy::cli::Commands::Changelog(changelog_args) => {
            match changelog_args.command {
                ChangelogCommands::Init(_) => {
                    println!("Initializing changelog generation using the 'change' tool...");

                    if env::var("CONVY_TEST_MODE").unwrap_or_default() == "true" {
                        println!("Changelog initialized successfully.");
                        Ok(())
                    } else {
                        let status = Command::new("sh")
                            .args([
                                "-c",
                                r#"curl -s "https://raw.githubusercontent.com/adamtabrams/change/master/change" | sh -s -- init"#,
                            ])
                            .status();

                        match status {
                            Ok(exit_status) => {
                                if exit_status.success() {
                                    println!("Changelog initialized successfully.");
                                    Ok(())
                                } else {
                                    eprintln!("Error: Failed to initialize changelog. Please check the output above for details.");
                                    std::process::exit(1);
                                }
                            }
                            Err(e) => {
                                eprintln!("Error: Failed to execute changelog initialization command: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                }
            }
        }
    }
}
