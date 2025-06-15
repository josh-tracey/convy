use logos::Logos;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Range;

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


// Define the error type for lexing. If Token::Error is not specified, it defaults to `()`.
// For this exercise, we'll assume it's `()` as Token does not define a custom Error type.
pub type LexerError = (); // This should be <Token as Logos>::Error

pub fn lex_commit_message(input: &str) -> Result<Vec<(Token, Range<usize>)>, LexerError> {
    let mut lexer = Token::lexer(input).spanned(); // Use .spanned() to get spans
    let mut tokens_with_spans = Vec::new();

    while let Some(item_tuple) = lexer.next() {
        // According to compiler, item_tuple is (Result<Token, ()>, Range<usize>)
        // This is unusual for logos .spanned() unless the token being spanned is itself a Result.
        // This implies Token::lexer(input) produces Iterator<Item=Result<Token, ()>>
        let (token_result, span) = item_tuple;
        match token_result {
            Ok(token) => tokens_with_spans.push((token, span)),
            Err(e) => {
                // e is of type <Token as Logos>::Error, which we assume is ()
                return Err(e);
            }
        }
    }
    // The original code's `match token { Ok(token) => ..., Err(e) => ...}` structure
    // implies that `lexer.next()` yields an `Option<Result<Token, ErrorType>>`.
    // If so, then `lexer.spanned().next()` would yield `Option<Result<(Token, Span), ErrorTypeSpanning>>`
    // if `Token` itself is the item being spanned.
    // Or, if `Result<Token, ErrorType>` is treated as the "token" by .spanned(), then we get the tuple reported by compiler.
    // This is complex. The current fix aligns with the compiler's type information.
    // Re-compile check
    Ok(tokens_with_spans)
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
    let tokens_with_spans = lex_commit_message(input).map_err(|_e| "Lexing error".to_string())?;
    let mut token_iter = tokens_with_spans.into_iter();

    let mut commit_type: Option<String> = None;
    let mut exclamation_mark = false;
    let mut scope: Option<String> = None;
    // subject will be declared and assigned after header parsing
    let mut body: Option<String> = None;
    // footers will be assigned from parse_body_and_footers
    let mut position = 0; // Tracks current position in the original input string

    // Parse the header line
    while let Some((token, span)) = token_iter.next() {
        match token {
            Token::Type(t) => {
                commit_type = Some(t);
                position = span.end;
            }
            Token::ExclamationMark => {
                exclamation_mark = true;
                position = span.end;
                // Original code had `span.end + 1` and then a break.
                // The `+1` might be to skip the `!` itself if it was part of the span.
                // If `ExclamationMark` token is just `!`, then `span.end` is correct.
                // Then we need to find the colon.
                // Let's adjust to look for colon explicitly after this.
                // The original code broke after `ExclamationMark` IF it was followed by colon.
                // This logic needs to be a bit more careful.
                // For now, let's assume `position` is correctly at the end of `!`.
                // The next token should be Colon or Scope then Colon.
            }
            Token::Scope(s) => {
                scope = Some(s);
                position = span.end;
            }
            Token::Colon => {
                position = span.end;
                break; // Stop parsing header after colon
            }
            Token::Whitespace => {
                // Whitespace might shift the effective 'position' if not handled carefully
                // For now, assume critical tokens update position correctly.
                // If whitespace is the last token before subject, its span.end might be used.
                position = span.end;
            }
            _ => return Err(format!("Invalid token in header: {:?} at {:?}", token, span)),
        }
    }

    // Ensure commit type was found
    if commit_type.is_none() {
        return Err("Commit type is missing".to_string());
    }

    // Parsing subject: Take text from 'position' up to the first newline
    // This part critically relies on 'position' being the char index after the header colon.
    let subject_text_slice = input.get(position..).ok_or_else(|| "Invalid position for subject".to_string())?;
    let subject = if let Some(subject_str) = subject_text_slice.splitn(2, '\n').next() {
        let s = subject_str.trim().to_string();
        if s.is_empty() {
            return Err("Subject is empty".to_string());
        }
        position += subject_str.len(); // Advance position by length of subject string part
        Some(s)
    } else {
        // This case means subject_text_slice was empty or did not contain \n,
        // but splitn should still yield one part.
        // If subject_str is empty here, it means nothing after colon.
        return Err("Subject is missing or invalid".to_string());
    };

    // Check if there's more content
    // Extract body and footers
    let body_and_footers_text = if input.len() > position {
        // Check if there's a newline immediately after the subject
        if input[position..].starts_with('\n') {
            position += 1; // Move past the first newline
             // If there's another newline (double newline), move past it as well
            if input[position..].starts_with('\n') {
                position +=1;
            }
            &input[position..]
        } else {
            // This case should ideally not happen if subject parsing is correct
            // and there's content after subject.
            // However, if it does, consider it as having no body/footers.
            ""
        }
    } else {
        ""
    };

    let (parsed_body, parsed_footers) = parse_body_and_footers(body_and_footers_text)?;
    if !parsed_body.is_empty() {
        body = Some(parsed_body);
    }
    let mut footers = parsed_footers; // Declare and assign footers here


    // Validate BREAKING-CHANGE footer requirements
    let has_breaking_change_footer = footers.keys().any(|k| k == "BREAKING-CHANGE" || k == "BREAKING CHANGE");

    if config.require_breaking_change_footer.unwrap_or(true) {
        if exclamation_mark && !has_breaking_change_footer {
            return Err(
                "Commit message with '!' in header must include 'BREAKING-CHANGE' or 'BREAKING CHANGE' in footers"
                    .to_string(),
            );
        }
    }

    // This check should always be enforced: if a BREAKING-CHANGE footer is present, '!' must be in the header.
    // This was previously inside the config block, but it should be independent.
    if has_breaking_change_footer && !exclamation_mark {
        return Err(
            "Commit message with 'BREAKING-CHANGE' or 'BREAKING CHANGE' in footers must include '!' in the header"
                .to_string(),
        );
    }

    // Normalize "BREAKING CHANGE" to "BREAKING-CHANGE"
    if footers.contains_key("BREAKING CHANGE") {
        if let Some(value) = footers.remove("BREAKING CHANGE") {
            footers.insert("BREAKING-CHANGE".to_string(), value);
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

// Parses the body and footers from the text following the commit subject.
// Footers are lines at the end of the commit that look like "Token: Value" or "Token # Value".
// Everything before the footers (if any) is considered the body.
// Body can have multiple paragraphs separated by blank lines.
fn parse_body_and_footers(text: &str) -> Result<(String, HashMap<String, String>), String> {
    let mut footers = HashMap::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut footer_start_index = lines.len();

    for (i, line) in lines.iter().enumerate().rev() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            // A blank line signifies a potential separation between body and footers.
            // If we've already started collecting footers, this blank line is part of the body.
            // If we haven't, it might just be a blank line before footers or part of a multi-paragraph body.
            // For simplicity, we'll assume a blank line means "stop collecting footers" if it's not the absolute last line.
            if i < lines.len() -1 && !footers.is_empty() { // if not the very last line and we have footers, it's body
                 break;
            }
            // If it's a trailing blank line or a blank line before any footers are found, continue searching upwards.
            footer_start_index = i;
            continue;
        }

        let parts: Option<(String, String)> = {
            let mut key_str: Option<String> = None;
            let mut val_str: Option<String> = None;

            // Try "Token # Value" first
            if let Some(hash_idx) = trimmed_line.find(" #") {
                let pk = trimmed_line[..hash_idx].trim();
                // Key for "#" separated footer should not be empty and ideally not contain ":"
                if !pk.is_empty() && !pk.contains(':') {
                    let pv = trimmed_line[hash_idx + 2..].trim(); // +2 for " #"
                    if !pv.is_empty() {
                        key_str = Some(pk.to_string());
                        val_str = Some(pv.to_string());
                    }
                }
            }

            // If not successfully parsed as "Token # Value", try "Token: Value"
            if key_str.is_none() { // Check key_str specifically, val_str might be None if key was parsed but val was empty
                if let Some(colon_idx) = trimmed_line.find(':') {
                    let pk = trimmed_line[..colon_idx].trim();
                    if !pk.is_empty() {
                        let pv = trimmed_line[colon_idx + 1..].trim();
                        if !pv.is_empty() {
                            // Only assign if we haven't already got a key from hash parsing
                            // This condition is slightly redundant due to outer key_str.is_none(), but safe.
                            if key_str.is_none() {
                                key_str = Some(pk.to_string());
                                val_str = Some(pv.to_string());
                            }
                        }
                    }
                }
            }

            if let (Some(k), Some(v)) = (key_str, val_str) {
                Some((k, v))
            } else {
                None
            }
        };

        if let Some((key, value)) = parts {
            // Git trailer convention: only the first of duplicated keys is used.
            // However, conventional commits might imply overriding or specific handling.
            // For now, let's mimic git trailers: if key already exists, do not overwrite.
            // Except for BREAKING-CHANGE, where we might want the last one if multiple are provided (though unusual).
            // For simplicity, let's use HashMap's default behavior (last one wins for simple inserts)
            // or decide on a specific strategy.
            // Conventional commits spec doesn't explicitly state how to handle duplicate footers.
            // Git's behavior is first-one-wins. Let's stick to that for non-BREAKING CHANGE.
            let normalized_key = if key.to_uppercase() == "BREAKING CHANGE" {
                "BREAKING-CHANGE".to_string()
            } else {
                key
            };

            if !footers.contains_key(&normalized_key) || normalized_key == "BREAKING-CHANGE" {
                 footers.insert(normalized_key, value);
            }
            footer_start_index = i;
        } else {
            // This line is not a footer, so everything from here upwards is body.
            break;
        }
    }

    // Reverse footers because we collected them bottom-up
    // No, HashMap doesn't preserve order, so reversing keys isn't meaningful here.
    // The collection logic ensures the correct `footer_start_index`.

    let body_lines = &lines[0..footer_start_index];
    let body = body_lines.join("\n").trim_end_matches('\n').to_string();

    // If body consists only of whitespace (e.g. after trimming only newlines), make it empty.
    if body.trim().is_empty() && !body.contains("\n\n") { // Preserve multi-paragraphs that are just spaces
        Ok(("".to_string(), footers))
    } else {
        Ok((body, footers))
    }
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
    fn test_commit_with_body_and_no_footers() {
        let message = "fix: a bug\n\nThis is a detailed explanation of the fix.\nIt has multiple lines.";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "fix");
        assert_eq!(commit.subject, "a bug");
        assert_eq!(commit.body.as_deref(), Some("This is a detailed explanation of the fix.\nIt has multiple lines."));
        assert!(commit.footers.is_empty());
    }

    #[test]
    fn test_commit_with_multiparagraph_body_and_footers() {
        let message = "feat: new feature\n\nFirst paragraph of the body.\n\nSecond paragraph of the body.\n\nReviewed-by: reviewer@example.com\nTicket #123";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.subject, "new feature");
        assert_eq!(commit.body.as_deref(), Some("First paragraph of the body.\n\nSecond paragraph of the body."));
        assert_eq!(commit.footers.get("Reviewed-by").unwrap(), "reviewer@example.com");
        assert_eq!(commit.footers.get("Ticket").unwrap(), "123");
    }


    #[test]
    fn test_breaking_change_in_footer() {
        let message = "feat!: remove deprecated API\n\nThis commit removes the deprecated API.\n\nBREAKING-CHANGE: The 'oldFunction' has been removed.";
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
    fn test_breaking_change_space_in_footer() {
        let message = "refactor!: major API overhaul\n\nDetails about the overhaul.\n\nBREAKING CHANGE: The entire API surface has changed.";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "refactor");
        assert_eq!(commit.subject, "major API overhaul");
        assert_eq!(commit.body.as_deref(), Some("Details about the overhaul."));
        assert_eq!(commit.footers.get("BREAKING-CHANGE").unwrap(), "The entire API surface has changed.");
    }

    #[test]
    fn test_footer_with_hash_separator() {
        let message = "fix: resolve issue\n\nFixed a critical bug.\n\nIssue #42\nReviewed-by: Another Dev <another@example.com>";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "fix");
        assert_eq!(commit.subject, "resolve issue");
        assert_eq!(commit.body.as_deref(), Some("Fixed a critical bug."));
        assert_eq!(commit.footers.get("Issue").unwrap(), "42");
        assert_eq!(commit.footers.get("Reviewed-by").unwrap(), "Another Dev <another@example.com>");
    }

    #[test]
    fn test_invalid_footer_format_mixed_with_valid() {
        // According to new logic, "InvalidFooterLine" will become part of the body.
        let message = "fix: correct typo\n\nSmall typo correction.\n\nInvalidFooterLine\nAuthor: test@example.com";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.body.as_deref(), Some("Small typo correction.\n\nInvalidFooterLine"));
        assert_eq!(commit.footers.get("Author").unwrap(), "test@example.com");
    }

    #[test]
    fn test_footer_like_lines_in_body() {
        let message = "docs: explain something\n\nBody line that looks like a footer: Not a real footer.\nThis is because the next line is not a footer.\n\nReal-Footer: value";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.body.as_deref(), Some("Body line that looks like a footer: Not a real footer.\nThis is because the next line is not a footer."));
        assert_eq!(commit.footers.get("Real-Footer").unwrap(), "value");
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

    #[test]
    fn test_multi_paragraph_body_then_footers() {
        let message = "feat: complex feature\n\nThis is the first paragraph.\nIt has several lines.\n\nThis is the second paragraph.\nAlso with multiple lines.\n\nReviewed-by: reviewer@example.com\nTicket: #456";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.subject, "complex feature");
        assert_eq!(commit.body.as_deref(), Some("This is the first paragraph.\nIt has several lines.\n\nThis is the second paragraph.\nAlso with multiple lines."));
        assert_eq!(commit.footers.get("Reviewed-by").unwrap(), "reviewer@example.com");
        assert_eq!(commit.footers.get("Ticket").unwrap(), "#456");
    }

    #[test]
    fn test_multi_paragraph_body_no_footers() {
        let message = "fix: detailed bug fix\n\nFirst part of the explanation.\n\nSecond part, elaborating further.\nStill no footers here.";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "fix");
        assert_eq!(commit.subject, "detailed bug fix");
        assert_eq!(commit.body.as_deref(), Some("First part of the explanation.\n\nSecond part, elaborating further.\nStill no footers here."));
        assert!(commit.footers.is_empty());
    }

    #[test]
    fn test_footers_with_hash_separator_variant() {
        let message = "refactor: use new pattern\n\nUpdated the core logic.\n\nOld-Component # OldClass\nNew-Component # NewClass\nFixes #123";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "refactor");
        assert_eq!(commit.subject, "use new pattern");
        assert_eq!(commit.body.as_deref(), Some("Updated the core logic."));
        assert_eq!(commit.footers.get("Old-Component").unwrap(), "OldClass");
        assert_eq!(commit.footers.get("New-Component").unwrap(), "NewClass");
        assert_eq!(commit.footers.get("Fixes").unwrap(), "123");
    }

    #[test]
    fn test_mixed_valid_invalid_footers_as_body() {
        // Invalid footer lines should be considered part of the body.
        let message = "chore: cleanup\n\nSome cleanup tasks.\nThis line is not a footer.\nAnother: valid-footer\nInvalid Footer Line\nAlso-Invalid;\nKey # Value";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "chore");
        assert_eq!(commit.subject, "cleanup");
        assert_eq!(commit.body.as_deref(), Some("Some cleanup tasks.\nThis line is not a footer.\nAnother: valid-footer\nInvalid Footer Line\nAlso-Invalid;"));
        assert_eq!(commit.footers.len(), 1);
        assert_eq!(commit.footers.get("Key").unwrap(), "Value");
    }

    #[test]
    fn test_breaking_change_missing_footer_error() {
        let message = "feat!: message\n\nBody only.";
        let config = default_config(); // require_breaking_change_footer = true
        let result = parse_commit_message(message, config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Commit message with '!' in header must include 'BREAKING-CHANGE' or 'BREAKING CHANGE' in footers");
    }

    #[test]
    fn test_breaking_change_footer_missing_exclamation_error() {
        let message = "feat: message\n\nBREAKING-CHANGE: description";
        let config = default_config(); // require_breaking_change_footer = true
        let result = parse_commit_message(message, config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Commit message with 'BREAKING-CHANGE' or 'BREAKING CHANGE' in footers must include '!' in the header");
    }

    #[test]
    fn test_breaking_change_footer_with_hash_ok() {
        let message = "feat!: message\n\nBREAKING CHANGE # description using hash";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert!(commit.footers.contains_key("BREAKING-CHANGE"));
        assert_eq!(commit.footers.get("BREAKING-CHANGE").unwrap(), "description using hash");
    }

    #[test]
    fn test_breaking_change_footer_optional_when_config_allows() {
        let message = "feat!: message\n\nBody only, no BREAKING-CHANGE footer.";
        let config = Config {
            additional_types: None,
            require_breaking_change_footer: Some(false),
        };
        let result = parse_commit_message(message, config);
        assert!(result.is_ok());
        let commit = result.unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert!(commit.footers.get("BREAKING-CHANGE").is_none());
    }

    #[test]
    fn test_breaking_change_footer_present_no_exclamation_optional_when_config_allows() {
        // This should still fail, config only affects ! requiring footer, not footer requiring !
        let message = "feat: message\n\nBREAKING-CHANGE: description here";
         let config = Config {
            additional_types: None,
            require_breaking_change_footer: Some(false),
        };
        let result = parse_commit_message(message, config);
       assert!(result.is_err());
       assert_eq!(result.unwrap_err(), "Commit message with 'BREAKING-CHANGE' or 'BREAKING CHANGE' in footers must include '!' in the header");
    }

    #[test]
    fn test_commit_with_only_body_no_footers() {
        let message = "fix: some fix\n\nThis is just a body.";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "fix");
        assert_eq!(commit.subject, "some fix");
        assert_eq!(commit.body.as_deref(), Some("This is just a body."));
        assert!(commit.footers.is_empty());
    }

    #[test]
    fn test_commit_with_blank_lines_in_body_and_footers() {
        let message = "feat: stuff\n\nBody line 1.\n\n\nBody line 2 after extra blank lines.\n\nFooter-One: Val1\n\nAnother-Footer: Val2";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.subject, "stuff");
        // Expectation: Only "Another-Footer: Val2" is a footer.
        // "Footer-One: Val1" becomes part of the body because a blank line separates it from the true last footer block.
        assert_eq!(commit.body.as_deref(), Some("Body line 1.\n\n\nBody line 2 after extra blank lines.\n\nFooter-One: Val1"));
        assert_eq!(commit.footers.len(), 1);
        assert_eq!(commit.footers.get("Another-Footer").unwrap(), "Val2");
    }

    #[test]
    fn test_commit_with_footer_values_containing_hash_or_colon() {
        let message = "docs: clarify something\n\nReviewed-By: User <user@example.com>\nTicket # Ref: #123\nDetails: Contains a colon : in value";
        let config = default_config();
        let commit = parse_commit_message(message, config).unwrap();
        assert_eq!(commit.footers.get("Reviewed-By").unwrap(), "User <user@example.com>");
        assert_eq!(commit.footers.get("Ticket").unwrap(), "Ref: #123"); // Key "Ticket", Value "Ref: #123"
        assert_eq!(commit.footers.get("Details").unwrap(), "Contains a colon : in value");
    }
}
