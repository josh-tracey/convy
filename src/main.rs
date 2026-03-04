use std::{fs, os::unix::fs::PermissionsExt, process::Command};
use toml;

use clap::Parser;
use colored::Colorize;
use convy::{
    cli::{ChangelogCommands, Commands, Cli},
    lexer::{default_config, parse_commit_message, Config},
    tui::run_wizard,
};

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.commands {
        Commands::Parse(arg) => {
            let config = load_config();

            match parse_commit_message(&arg.commit, config) {
                Ok(_) => {
                    println!("{} Commit message is valid!", "✔".green());
                }
                Err(e) => {
                    eprintln!("{} {}", "✘ Error:".red(), e);
                    std::process::exit(1);
                }
            };

            Ok(())
        }
        Commands::Init(_) => {
            // Check if Git is installed
            let git_version_output = Command::new("git").arg("--version").output();
            if git_version_output.is_err() || !git_version_output.unwrap().status.success() {
                eprintln!("{} Git is not installed. Please install Git and try again.", "Error:".red());
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
                        eprintln!("{} Not inside a Git repository. Please run `convy init` from the root of a Git repository.", "Error:".red());
                        std::process::exit(1);
                    }
                }
                Err(_) => {
                    eprintln!("{} Failed to check Git repository status. Make sure you are in a Git repository.", "Error:".red());
                    std::process::exit(1);
                }
            }

            // Config check
            if fs::metadata(".convy.toml").is_ok() {
                 println!("{} .convy.toml already exists. Skipping creation.", "!".yellow());
            } else {
                let default_config_str =
                    toml::to_string(&default_config()).expect("Error creating default config");
                fs::write(".convy.toml", default_config_str)
                    .expect("Error writing default config to file");
                println!("{} Created .convy.toml", "✔".green());
            }

            // Hook check
            let hook_path = ".git/hooks/commit-msg";
            if fs::metadata(hook_path).is_ok() {
                 println!("{} Hook already exists at {}. Skipping overwrite to avoid data loss.", "!".yellow(), hook_path);
                 println!("To use convy, ensure your hook runs: `convy parse \"$(cat $1)\"`");
            } else {
                //embed commit-msg in binary
                let commit_msg = include_str!("commit_msg");

                // Ensure .git/hooks directory exists
                if !fs::metadata(".git/hooks").is_ok() {
                    fs::create_dir_all(".git/hooks")
                        .expect("Error creating .git/hooks directory");
                }
                fs::write(hook_path, commit_msg)
                    .expect("Error writing config file to git hooks");

                // make commit-msg executable
                fs::set_permissions(hook_path, fs::Permissions::from_mode(0o755))
                    .expect("Error setting permissions on commit-msg");
                
                println!("{} Installed git hook at {}", "✔".green(), hook_path);
            }

            println!("\nSuccessfully initialized convy!");
            Ok(())
        }
        Commands::Changelog(changelog_args) => {
             match changelog_args.command {
                ChangelogCommands::Init(_) => convy::changelog::init(),
                ChangelogCommands::Generate(args) => convy::changelog::generate(args.write, args.all),
                ChangelogCommands::Release(args) => convy::changelog::release(&args.version),
            }
        }
        Commands::Commit(args) => {
            let config = load_config_safe().unwrap_or_else(default_config);
            
            let msg = run_wizard(config).map_err(|e| e.to_string())?;

            if let Some(msg) = msg {
                if args.run {
                    let status = Command::new("git")
                        .arg("commit")
                        .arg("-m")
                        .arg(&msg)
                        .status()
                        .map_err(|e| e.to_string())?;
                    
                    if !status.success() {
                        return Err("Git commit failed".to_string());
                    }
                    println!("{} Committed!", "✔".green());
                } else {
                    println!("\n{}", "Preview:".bold().underline());
                    println!("{}", msg.dimmed());
                    println!("\nRun with --run to execute git commit automatically.");
                }
            } else {
                println!("Aborted.");
            }

            Ok(())
        }
    }
}

fn load_config() -> Config {
    match fs::read_to_string(".convy.toml") {
        Ok(content) => toml::from_str(&content).expect("Error parsing TOML"),
        Err(_) => {
            // Check current dir, if not found, we could look recursively up, 
            // but for now let's strict check as per original behavior or return default?
            // Original behavior exit(1).
            eprintln!("Error: .convy.toml not found. Please run `convy init`.");
            std::process::exit(1);
        }
    }
}

fn load_config_safe() -> Option<Config> {
    match fs::read_to_string(".convy.toml") {
        Ok(content) => toml::from_str(&content).ok(),
        Err(_) => None,
    }
}
