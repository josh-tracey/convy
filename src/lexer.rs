use logos::Logos;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Configuration Structure
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub additional_types: Option<Vec<String>>, // Optional additional types
    pub require_breaking_change_footer: Option<bool>,
}

// Default Configuration
pub fn default_config() -> Config {
    Config {
        additional_types: None,
        require_breaking_change_footer: Some(true),
    }
}

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    // Types like feat, fix, etc.
    #[regex(
        r"(feat|fix|docs|style|refactor|test|chore|perf|build|ci|revert|merge|wip)",
        |lex| lex.slice().to_string(),
        priority = 6
    )]
    Type(String),

    #[regex(
        r"\([^\)]+\)",
        |lex| lex.slice()[1..lex.slice().len() - 1].to_string(),
        priority = 3
    )]
    Scope(String),

    // Exclamation mark for breaking changes
    #[token("!", priority = 5)]
    ExclamationMark,

    // Colon separator
    #[token(":", priority = 3)]
    Colon,

    // Newline characters
    #[token("\n\n")]
    DoubleNewline,
    #[token("\n")]
    Newline,

    // Whitespace (to be skipped)
    #[regex(r"[ \t]+", priority = 3)]
    Whitespace,

    // Tags (like BREAKING CHANGE)
    #[regex(r"([A-Z-]+)", |lex| lex.slice().to_string(), priority = 2)]
    Tag(String),

    // Subject and body (catch-all for now)
    #[regex(r"[^\n:() !]+", |lex| lex.slice().to_string(), priority = 0)]
    Text(String),

    // catches any mention (ie @user)
    #[regex(r"@[^\s]+\w", |lex| lex.slice().to_string(),priority = 6)]
    Mention(String),

    // catches any email (ie user@domain)
    #[regex(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", |lex| lex.slice().to_string(), priority = 2)]
    Email(String),
}

pub fn lex_commit_message(input: &str) -> Vec<Token> {
    let mut lexer = Token::lexer(input);
    let mut tokens = Vec::new();

    while let Some(token) = lexer.next() {
        tokens.push(match token {
            Ok(token) => token,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                continue;
            }
        });
    }

    tokens
}

#[derive(Debug)]
pub struct CommitMessage {
    pub commit_type: String,
    pub scope: Option<String>,
    pub subject: String,
    pub body: Option<String>,
    pub footers: HashMap<String, String>,
}

pub fn parse_commit_message(input: &str, config: Config) -> Result<CommitMessage, String> {
    let mut lexer = Token::lexer(input);
    let mut commit_type: Option<String> = None;
    let mut exclamation_mark = false;
    let mut scope: Option<String> = None;
    let mut subject: Option<String> = None;
    let mut body: Option<String> = None;
    let mut footers = HashMap::new();
    let mut position = 0;

    // Parse the header line
    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Type(t)) => {
                commit_type = Some(t);
                position = lexer.span().end;
            }
            Ok(Token::ExclamationMark) => {
                exclamation_mark = true;
                position = lexer.span().end + 1;
                break;
            }
            Ok(Token::Scope(s)) => {
                scope = Some(s);
                position = lexer.span().end;
            }
            Ok(Token::Colon) => {
                position = lexer.span().end;
                break; // Stop parsing header after colon
            }
            Ok(Token::Whitespace) => {
                // Ignore whitespace
            }
            e => return Err(format!("Invalid commit message format: {:?}", e)),
        }
    }

    if subject.is_none() {
        // Parsing subject
        if let Some(rest) = input[position..].splitn(2, '\n').next() {
            subject = Some(rest.trim().to_string());
            position += rest.len();
        } else {
            return Err("Subject is missing".to_string());
        }
    }

    // Check if there's more content
    let remaining_input = &input[position..];
    if remaining_input.starts_with("\n\n") {
        position += 2; // Skip the double newline
                       // Now we need to parse body and footers
        let rest = &input[position..];
        if let Some((body_text, footer_text)) = split_body_and_footers(rest) {
            body = Some(body_text.trim().to_string());
            footers = parse_footers(&footer_text)?;
        } else {
            // No footers, all remaining text is body
            body = Some(rest.trim().to_string());
        }
    } else if remaining_input.starts_with('\n') {
        position += 1; // Skip the newline

        // Check if there's more content
        let rest = &input[position..];
        if let Some((body_text, footer_text)) = split_body_and_footers(rest) {
            body = Some(body_text.trim().to_string());
            footers = parse_footers(&footer_text)?;
        } else {
            // No footers, all remaining text is body
            body = Some(rest.trim().to_string());
        }
    }

    if config.require_breaking_change_footer.unwrap_or(true) {
        // Enforce BREAKING-CHANGE tag if exclamation mark is present
        if exclamation_mark {
            let has_breaking_change = footers.contains_key("BREAKING-CHANGE");

            if !has_breaking_change {
                return Err(
                    "Commit message with '!' must include 'BREAKING-CHANGE' in the footers"
                        .to_string(),
                );
            }
        }

        if footers.contains_key("BREAKING-CHANGE") {
            if !exclamation_mark {
                return Err(
                    "Commit message with 'BREAKING-CHANGE' must include '!' in the header"
                        .to_string(),
                );
            }
        }
    }

    if let Some(commit_type) = commit_type {
        if let Some(subject) = subject {
            if subject.is_empty() {
                return Err("Subject is empty".to_string());
            }
            Ok(CommitMessage {
                commit_type,
                scope,
                subject,
                body,
                footers,
            })
        } else {
            Err("Subject is missing".to_string())
        }
    } else {
        Err("Commit type is missing".to_string())
    }
}

fn split_body_and_footers(input: &str) -> Option<(String, String)> {
    // Look for double newline indicating the start of footers
    if let Some(index) = input.find("\n\n") {
        let (body, footers) = input.split_at(index);
        Some((body.to_string(), footers[2..].to_string())) // Skip the double newline
    } else {
        None
    }
}

fn parse_footers(input: &str) -> Result<HashMap<String, String>, String> {
    let mut footers = HashMap::new();
    for line in input.lines() {
        if let Some((key, value)) = line.split_once(':') {
            footers.insert(key.trim().to_string(), value.trim().to_string());
        } else {
            return Err(format!("Invalid footer line: '{}'", line));
        }
    }
    Ok(footers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_with_footers() {
        let message = "feat: add new API endpoint\n\nThis introduces a new endpoint.\n\nSigned-off-by: Jane Doe <jane@example.com>\nCo-authored-by: John Smith <john@example.com>";

        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, None);
        assert_eq!(commit.subject, "add new API endpoint");
        assert_eq!(commit.body.unwrap(), "This introduces a new endpoint.");
        assert_eq!(
            commit.footers.get("Signed-off-by").unwrap(),
            "Jane Doe <jane@example.com>"
        );
        assert_eq!(
            commit.footers.get("Co-authored-by").unwrap(),
            "John Smith <john@example.com>"
        );
    }

    #[test]
    fn test_breaking_change_in_footer() {
        let message = "feat!: remove deprecated API\nThis commit removes the deprecated API.\n\nBREAKING-CHANGE: The 'oldFunction' has been removed.";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, None);
        assert_eq!(commit.subject, "remove deprecated API");
        assert_eq!(
            commit.body.unwrap(),
            "This commit removes the deprecated API."
        );
        assert_eq!(
            commit.footers.get("BREAKING-CHANGE").unwrap(),
            "The 'oldFunction' has been removed."
        );
    }

    #[test]
    fn test_invalid_footer_format() {
        let message = "fix: correct typo\n\nSmall typo correction.\n\nInvalidFooterLine";
        let config = default_config();
        let result = parse_commit_message(message, config);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid footer line: 'InvalidFooterLine'"
        );
    }

    #[test]
    fn test_commit_without_footers() {
        let message = "chore: update dependencies\n\nUpdated to the latest versions.";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "chore");
        assert_eq!(commit.scope, None);
        assert_eq!(commit.subject, "update dependencies");
        assert_eq!(commit.body.unwrap(), "Updated to the latest versions.");
        assert!(commit.footers.is_empty());
    }

    #[test]
    fn test_commit_with_only_subject() {
        let message = "docs: improve documentation";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "docs");
        assert_eq!(commit.scope, None);
        assert_eq!(commit.subject, "improve documentation");
        assert!(commit.body.is_none());
        assert!(commit.footers.is_empty());
    }
}
