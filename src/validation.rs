use crate::lexer::{FooterToken, Token};
use serde::Deserialize;

// Configuration Structure
#[derive(Debug, Deserialize)]
pub struct Config {
    additional_types: Option<Vec<String>>, // Optional additional types
    require_breaking_change_footer: Option<bool>,
}

// Default Configuration
pub fn default_config() -> Config {
    Config {
        additional_types: None,
        require_breaking_change_footer: Some(true),
    }
}

// Example Error Type - You might define a more comprehensive one
#[derive(Debug)]
pub enum CommitMessageError {
    MissingBreakingChangeFooter,
    InvalidType(String),
}

pub fn validate_commit_message(
    commit: &str,
    tokens: Vec<Token>,
    config: Option<&Config>,
) -> Result<(), String> {
    let mut allowed_types = vec![
        // Start with the built-in types
        "feat".to_string(),
        "fix".to_string(),
        "chore".to_string(),
        "docs".to_string(),
        "style".to_string(),
        "refactor".to_string(),
        "perf".to_string(),
        "test".to_string(),
        "build".to_string(),
        "ci".to_string(),
    ];

    if let Some(config) = config {
        // Validation Rule 1: Check if the commit type is allowed

        // Append additional types if specified in the config
        if let Some(additional) = &config.additional_types {
            allowed_types.extend(additional.iter().cloned());
        }

        // Validation Rule 2: Mandatory BREAKING CHANGE footer
        // Check if the commit message contains a '!' and if the config requires a BREAKING CHANGE footer
        if commit.contains('!') && config.require_breaking_change_footer.unwrap_or(true) {
            if !tokens
                .iter()
                .any(|t| matches!(t, Token::Footer(FooterToken::BreakingChange, _)))
            {
                return Err("Missing BREAKING CHANGE footer".to_string());
            }
        }
    }

    // Check if the commit type is allowed
    if let Some(Token::Type(token_type)) = tokens.iter().find(|t| matches!(t, Token::Type(_))) {
        if !allowed_types.contains(&token_type.to_string()) {
            return Err(format!(
                "Invalid commit type: `{}`.\n\nValid Options:\n{}",
                token_type.to_string(),
                allowed_types.join(", ")
            ));
        }
    }

    Ok(())
}
