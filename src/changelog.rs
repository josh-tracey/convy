use std::fs;
use std::path::Path;
use std::process::Command;
use chrono::Local;
use colored::Colorize;
use regex::Regex;

use crate::lexer::{default_config, parse_commit_message};

const CHANGELOG_FILE: &str = "CHANGELOG.md";

pub fn init() -> Result<(), String> {
    if Path::new(CHANGELOG_FILE).exists() {
        return Err(format!("{} already exists.", CHANGELOG_FILE));
    }

    let header = r#"# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
"#;

    fs::write(CHANGELOG_FILE, header).map_err(|e| e.to_string())?;
    println!("{} Created {}", "✔".green(), CHANGELOG_FILE);
    Ok(())
}

pub fn generate(write: bool, all: bool) -> Result<(), String> {
    // 1. Find range
    let range = if all {
        "HEAD".to_string()
    } else {
        // Try to find the latest tag
        let output = Command::new("git")
            .args(["describe", "--tags", "--abbrev=0"])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
            format!("{}..HEAD", tag)
        } else {
            // No tags found, assume all
            "HEAD".to_string()
        }
    };

    println!("{} Generating changelog for range: {}", "ℹ".blue(), range);

    // 2. Get commits
    let output = Command::new("git")
        .args(["log", &range, "--format=%B%n---CONVY_DELIM---"])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("Failed to read git log".to_string());
    }

    let raw_log = String::from_utf8_lossy(&output.stdout);
    let raw_commits: Vec<&str> = raw_log.split("---CONVY_DELIM---\n").collect();

    let mut feats = Vec::new();
    let mut fixes = Vec::new();
    let mut docs = Vec::new();
    let mut other = Vec::new();
    let mut breaking = Vec::new();

    let config = default_config();

    for raw_msg in raw_commits {
        if raw_msg.trim().is_empty() {
            continue;
        }

        if let Ok(commit) = parse_commit_message(raw_msg.trim(), config.borrow_config()) {
            let desc = format!("{}: {}", 
                commit.scope.as_ref().map(|s| format!("**{}**", s)).unwrap_or_default(),
                commit.subject
            ).trim_start_matches(": ").to_string();

            // Check for breaking changes
            let is_breaking = raw_msg.contains("BREAKING CHANGE") || raw_msg.contains("!");
            if is_breaking {
                breaking.push(desc.clone());
            }

            match commit.commit_type.as_str() {
                "feat" => feats.push(desc),
                "fix" => fixes.push(desc),
                "docs" => docs.push(desc),
                _ => other.push(format!("{}: {}", commit.commit_type, desc)),
            }
        }
    }

    // 3. Render Markdown
    let mut md = String::new();
    
    if !breaking.is_empty() {
        md.push_str("### ⚠ BREAKING CHANGES\n");
        for c in breaking { md.push_str(&format!("- {}\n", c)); }
        md.push('\n');
    }
    
    if !feats.is_empty() {
        md.push_str("### Features\n");
        for c in feats { md.push_str(&format!("- {}\n", c)); }
        md.push('\n');
    }

    if !fixes.is_empty() {
        md.push_str("### Bug Fixes\n");
        for c in fixes { md.push_str(&format!("- {}\n", c)); }
        md.push('\n');
    }
    
    if !docs.is_empty() {
        md.push_str("### Documentation\n");
        for c in docs { md.push_str(&format!("- {}\n", c)); }
        md.push('\n');
    }

    // 4. Output
    if write {
        if !Path::new(CHANGELOG_FILE).exists() {
            return Err(format!("{} not found. Run `init` first.", CHANGELOG_FILE));
        }

        let content = fs::read_to_string(CHANGELOG_FILE).map_err(|e| e.to_string())?;
        
        // Regex to find [Unreleased] section
        // We replace everything between "## [Unreleased]" and the next "## [" or EOF
        let re = Regex::new(r"(?s)(## \[Unreleased\]\n)(.*?)(## \[|$)").unwrap();
        
        let new_content = if re.is_match(&content) {
            re.replace(&content, |caps: &regex::Captures| {
                format!("{}{}{}", &caps[1], md, &caps[3])
            }).to_string()
        } else {
            // If no [Unreleased] section found, try to insert after header
             // This is a simple fallback
            if content.contains("## [Unreleased]") {
                 // Should have been caught by regex, maybe formatting weirdness
                 content + "\n" + &md
            } else {
                // Insert at top? No, usually after header.
                 // Try to find the end of the header
                 let split_idx = content.find("## [").unwrap_or(content.len());
                 let (head, tail) = content.split_at(split_idx);
                 format!("{}## [Unreleased]\n\n{}{}", head, md, tail)
            }
        };

        fs::write(CHANGELOG_FILE, new_content).map_err(|e| e.to_string())?;
        println!("{} Updated {}", "✔".green(), CHANGELOG_FILE);

    } else {
        println!("{}", md);
    }

    Ok(())
}

pub fn release(version: &str) -> Result<(), String> {
    if !Path::new(CHANGELOG_FILE).exists() {
        return Err(format!("{} not found. Run `init` first.", CHANGELOG_FILE));
    }

    let content = fs::read_to_string(CHANGELOG_FILE).map_err(|e| e.to_string())?;
    
    // Check if version already exists
    if content.contains(&format!("## [{}]", version)) {
        return Err(format!("Version {} already exists in changelog.", version));
    }

    // Replace [Unreleased] with [Version] - Date
    // And add a new empty [Unreleased] section on top
    
    let date = Local::now().format("%Y-%m-%d").to_string();
    let new_header = format!("## [Unreleased]\n\n## [{}] - {}", version, date);
    
    let new_content = content.replace("## [Unreleased]", &new_header);
    
    if new_content == content {
         return Err("Could not find '## [Unreleased]' section to release.".to_string());
    }

    fs::write(CHANGELOG_FILE, new_content).map_err(|e| e.to_string())?;
    println!("{} Released version {} in {}", "✔".green(), version, CHANGELOG_FILE);
    
    Ok(())
}

// Helper trait to allow passing config by value or reference if needed, 
// but since lexer takes by value, we need to clone.
trait BorrowConfig {
    fn borrow_config(&self) -> crate::lexer::Config;
}

impl BorrowConfig for crate::lexer::Config {
    fn borrow_config(&self) -> crate::lexer::Config {
        crate::lexer::Config {
             additional_types: self.additional_types.clone(),
             scopes: self.scopes.clone(),
             require_breaking_change_footer: self.require_breaking_change_footer,
             emoji: self.emoji,
        }
    }
}
