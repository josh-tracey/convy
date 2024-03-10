use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::iter::Iterator;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Type(TokenType),
    Scope(String),
    Description(String),
    Body(String),
    Footer(FooterToken, String),
}

impl Token {
    pub fn is_breaking_change(&self) -> bool {
        if let Token::Footer(FooterToken::BreakingChange, _) = self {
            true
        } else {
            false
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Token::Type(ty) => write!(f, "{}", ty),
            Token::Scope(scope) => write!(f, "({})", scope),
            Token::Description(desc) => write!(f, "{}", desc),
            Token::Body(body) => write!(f, "{}", body),
            Token::Footer(token, value) => write!(f, "{}: {}", token, value),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Feature,
    Fix,
    Chore,
    Docs,
    Style,
    Refactor,
    Perf,
    Test,
    Build,
    Ci,
    Other(String),
}

impl TokenType {
    pub fn from_str(s: &str) -> Option<TokenType> {
        match s.to_lowercase().as_str() {
            "feat" => Some(TokenType::Feature),
            "fix" => Some(TokenType::Fix),
            "chore" => Some(TokenType::Chore),
            "docs" => Some(TokenType::Docs),
            "style" => Some(TokenType::Style),
            "refactor" => Some(TokenType::Refactor),
            "perf" => Some(TokenType::Perf),
            "test" => Some(TokenType::Test),
            "build" => Some(TokenType::Build),
            "ci" => Some(TokenType::Ci),
            _ => Some(TokenType::Other(s.to_string())),
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TokenType::Feature => write!(f, "feat"),
            TokenType::Fix => write!(f, "fix"),
            TokenType::Chore => write!(f, "chore"),
            TokenType::Docs => write!(f, "docs"),
            TokenType::Style => write!(f, "style"),
            TokenType::Refactor => write!(f, "refactor"),
            TokenType::Perf => write!(f, "perf"),
            TokenType::Test => write!(f, "test"),
            TokenType::Build => write!(f, "build"),
            TokenType::Ci => write!(f, "ci"),
            TokenType::Other(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum FooterToken {
    BreakingChange,
    ReviewedBy,
    AckedBy,
    Refs,
}

impl FooterToken {
    pub fn from_str(s: &str) -> Option<FooterToken> {
        match s.to_uppercase().as_str() {
            "BREAKING CHANGE" => Some(FooterToken::BreakingChange),
            "BREAKING-CHANGE" => Some(FooterToken::BreakingChange),
            "REVIEWED-BY" => Some(FooterToken::ReviewedBy),
            "ACKED-BY" => Some(FooterToken::AckedBy),
            "REFS" => Some(FooterToken::Refs),
            _ => None,
        }
    }
}

impl Display for FooterToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            FooterToken::BreakingChange => write!(f, "BREAKING CHANGE"),
            FooterToken::ReviewedBy => write!(f, "Reviewed-by"),
            FooterToken::AckedBy => write!(f, "Acked-by"),
            FooterToken::Refs => write!(f, "Refs"),
        }
    }
}

pub struct LexicalTokenizer {
    keywords: HashMap<String, TokenType>,
    footer_keywords: HashMap<String, FooterToken>,
}

impl LexicalTokenizer {
    pub fn new() -> LexicalTokenizer {
        LexicalTokenizer {
            keywords: HashMap::from([
                ("feat".to_string(), TokenType::Feature),
                ("fix".to_string(), TokenType::Fix),
                ("chore".to_string(), TokenType::Chore),
                ("docs".to_string(), TokenType::Docs),
                ("style".to_string(), TokenType::Style),
                ("refactor".to_string(), TokenType::Refactor),
                ("perf".to_string(), TokenType::Perf),
                ("test".to_string(), TokenType::Test),
                ("build".to_string(), TokenType::Build),
                ("ci".to_string(), TokenType::Ci),
            ]),
            footer_keywords: HashMap::from([
                ("BREAKING CHANGE".to_string(), FooterToken::BreakingChange),
                ("REVIEWED-BY".to_string(), FooterToken::ReviewedBy),
                ("ACKED-BY".to_string(), FooterToken::AckedBy),
                ("REFS".to_string(), FooterToken::Refs),
            ]),
        }
    }

    pub fn tokenize(&self, message: &str) -> Vec<Token> {
        let mut tokens = vec![];
        let mut parts = message.split('\n');

        if let Some(first_line) = parts.next() {
            let mut first_line_parts = first_line.splitn(2, ':'); // Split into only two parts

            if let Some(type_and_scope) = first_line_parts.next() {
                let mut type_and_scope = type_and_scope.to_lowercase();

                if type_and_scope.ends_with('!') {
                    type_and_scope.pop(); // Remove the '!'
                }

                if let Some(type_str) = self.keywords.get(&type_and_scope) {
                    tokens.push(Token::Type(type_str.clone()));
                } else {
                    // Try to extract scope, if present
                    let type_and_scope_parts: Vec<&str> = type_and_scope.split('(').collect();
                    if type_and_scope_parts.len() == 2 && type_and_scope_parts[1].ends_with(')') {
                        if let Some(type_str) =
                            self.keywords.get(&type_and_scope_parts[0].to_lowercase())
                        {
                            tokens.push(Token::Type(type_str.clone()));
                            let scope =
                                type_and_scope_parts[1][..type_and_scope_parts[1].len() - 1].trim();
                            tokens.push(Token::Scope(scope.to_string()));
                        } else {
                            tokens.push(Token::Type(TokenType::Other(type_and_scope.to_string())));
                        }
                    } else {
                        tokens.push(Token::Type(TokenType::Other(type_and_scope.to_string())));
                    }
                }
            }

            // Check for description after type/scope
            if let Some(desc) = first_line_parts.next() {
                tokens.push(Token::Description(desc.trim().to_string()));
            }
        }

        let mut in_footer = false;
        let mut body = String::new();

        for part in parts {
            if in_footer {
                if let Some(token) = self.parse_footer_line(part.trim()) {
                    tokens.push(Token::Footer(token.0, token.1));
                } else {
                    panic!("Unknown footer token: {}", part.trim());
                }
            } else {
                if let Some(token) =
                    FooterToken::from_str(part.trim().splitn(2, ':').next().unwrap())
                {
                    in_footer = true;
                    let value = part.trim()[token.to_string().len() + 2..]
                        .trim()
                        .to_string();
                    tokens.push(Token::Footer(token, value));
                } else {
                    body.push_str(part.trim());
                    body.push('\n');
                }
            }
        }

        if !body.is_empty() {
            tokens.push(Token::Body(body));
        }

        tokens
    }

    fn parse_footer_line(&self, line: &str) -> Option<(FooterToken, String)> {
        let mut parts = line.splitn(2, ':');
        let token_str = parts.next()?.trim().to_uppercase();
        let value = parts.next()?.trim().to_string();

        self.footer_keywords
            .get(&token_str)
            .map(|token| (token.clone(), value))
    }
}
