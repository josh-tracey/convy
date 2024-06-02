use crate::lexer::{FooterToken, Token, TokenType};

// Example Error Type - You might define a more comprehensive one
#[derive(Debug)]
pub enum CommitMessageError {
    MissingBreakingChangeFooter,
    InvalidType(String),
}

pub fn validate_commit_message(commit: &str, tokens: Vec<Token>) -> Result<(), String> {
    // Validation Rule 1: Check for allowed types
    if let Some(Token::Type(token_type)) = tokens.iter().find(|t| matches!(t, Token::Type(_))) {
        match token_type {
            TokenType::Feature => "feat",
            TokenType::Fix => "fix",
            TokenType::Chore => "chore",
            TokenType::Docs => "docs",
            TokenType::Style => "style",
            TokenType::Refactor => "refactor",
            TokenType::Perf => "perf",
            TokenType::Test => "test",
            TokenType::Build => "build",
            TokenType::Ci => "ci",
            _ => return Err(
                format!(
                    "Invalid commit type: `{}`.\n\nValid Options:\nfeat, fix, chore, docs, style, refactor, perf, test, build, and ci", 
                    token_type.to_string()
                )
            ),
        };
    }

    // Validation Rule 2: Mandatory BREAKING CHANGE footer
    if tokens.iter().any(|t| matches!(t, Token::Type(_)))
        && commit.contains('!')
        && !tokens
            .iter()
            .any(|t| matches!(t, Token::Footer(FooterToken::BreakingChange, _)))
    {
        return Err("Missing BREAKING CHANGE footer".to_string());
    }

    Ok(())
}
