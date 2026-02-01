use std::{env, fs, os::unix::fs::PermissionsExt, process::Command};
use toml;

use clap::Parser;
use colored::Colorize;
use inquire::{Confirm, Select, Text};
use convy::{
    cli::{ChangelogCommands, Commands, Cli},
    lexer::{default_config, parse_commit_message, Config},
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
        Commands::Commit(args) => {
            let config = load_config_safe().unwrap_or_else(default_config);
            
            // 1. Type
            let mut types = vec![
                "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert"
            ];
            if let Some(extra) = &config.additional_types {
                for t in extra {
                    if !types.contains(&t.as_str()) {
                        types.push(t);
                    }
                }
            }
            types.sort();
            
            let commit_type = Select::new("Type:", types).prompt().map_err(|e| e.to_string())?;
            
            // 2. Scope
            let scope = if let Some(scopes) = &config.scopes {
                let mut opts = scopes.iter().map(|s| s.as_str()).collect::<Vec<_>>();
                opts.insert(0, "(none)");
                let sel = Select::new("Scope:", opts).prompt().map_err(|e| e.to_string())?;
                if sel == "(none)" { None } else { Some(sel.to_string()) }
            } else {
                let input = Text::new("Scope (optional):").prompt().map_err(|e| e.to_string())?;
                if input.trim().is_empty() { None } else { Some(input.trim().to_string()) }
            };
            
            // 3. Breaking Change
            let is_breaking = Confirm::new("Is this a breaking change?").with_default(false).prompt().map_err(|e| e.to_string())?;
            
            // 4. Description
            let description = Text::new("Description:")
                .with_validator(|input: &str| {
                    if input.len() > 100 {
                         Ok(inquire::validator::Validation::Invalid("Description too long (max 100)".into()))
                    } else if input.trim().is_empty() {
                         Ok(inquire::validator::Validation::Invalid("Description cannot be empty".into()))
                    } else {
                        Ok(inquire::validator::Validation::Valid)
                    }
                })
                .prompt()
                .map_err(|e| e.to_string())?;

            // 5. Body
            let body = Text::new("Body (optional):").prompt().map_err(|e| e.to_string())?;
            
            // 6. Breaking footer text if needed
            let breaking_footer = if is_breaking {
                 let text = Text::new("Breaking Change Description:").prompt().map_err(|e| e.to_string())?;
                 Some(text)
            } else {
                None
            };

            // 7. Additional Footers
            let mut footers = Vec::new();
            loop {
                let add_footer = Confirm::new("Add a footer (e.g. Co-authored-by)?").with_default(false).prompt().map_err(|e| e.to_string())?;
                if !add_footer { break; }
                
                let key = Text::new("Footer Key (e.g. Issue):").prompt().map_err(|e| e.to_string())?;
                let value = Text::new("Footer Value (e.g. #123):").prompt().map_err(|e| e.to_string())?;
                if !key.trim().is_empty() && !value.trim().is_empty() {
                    footers.push((key, value));
                }
            }

            // Construct message
            let mut msg = String::new();
            msg.push_str(commit_type);
            if let Some(s) = scope {
                msg.push('(');
                msg.push_str(&s);
                msg.push(')');
            }
            if is_breaking {
                msg.push('!');
            }
            msg.push_str(": ");
            
            if config.emoji.unwrap_or(false) {
                let emoji = match commit_type {
                    "feat" => "✨ ",
                    "fix" => "🐛 ",
                    "docs" => "📚 ",
                    "style" => "💎 ",
                    "refactor" => "♻️ ",
                    "perf" => "🚀 ",
                    "test" => "🚨 ",
                    "build" => "🛠 ",
                    "ci" => "⚙️ ",
                    "chore" => "🔧 ",
                    "revert" => "⏪ ",
                    _ => "",
                };
                msg.push_str(emoji);
            }

            msg.push_str(&description);
            
            if !body.trim().is_empty() {
                msg.push_str("\n\n");
                msg.push_str(&body);
            }
            
            if let Some(ref bf) = breaking_footer {
                msg.push_str("\n\nBREAKING CHANGE: ");
                msg.push_str(bf);
            }
            
            // Add other footers
            if !footers.is_empty() {
                // If we haven't added newlines for body or breaking change yet, add them
                if body.trim().is_empty() && breaking_footer.is_none() {
                    msg.push_str("\n\n");
                } else if !msg.ends_with('\n') {
                    msg.push('\n');
                }
                
                for (k, v) in footers {
                    msg.push_str(&format!("{}: {}\n", k, v));
                }
                // Trim trailing newline from loop
                if msg.ends_with('\n') {
                    msg.pop();
                }
            }

            println!("\n{}", "Preview:".bold().underline());
            println!("{}", msg.dimmed());
            println!();

            if args.run {
                let confirm = Confirm::new("Commit this message?").with_default(true).prompt().map_err(|e| e.to_string())?;
                if confirm {
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
                    println!("Aborted.");
                }
            } else {
                println!("Run with --run to execute git commit automatically.");
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
