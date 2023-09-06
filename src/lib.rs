use std::{error::Error, str::Chars};

#[derive(Debug, PartialEq)]
pub struct ConventionalCommit {
    pub commit_type: Node,
    pub scope: Option<Node>,
    pub description: Node,
    pub body: Option<Node>,
    pub footer: Option<Node>,
}

impl ConventionalCommit {
    pub fn new() -> Self {
        ConventionalCommit {
            commit_type: Node::new(TokenType::CommitType, String::new()),
            scope: None,
            description: Node::new(TokenType::Description, String::new()),
            body: None,
            footer: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AngularConventionTypes {
    Feat,
    Fix,
    Docs,
    Style,
    Refactor,
    Perf,
    Test,
    Build,
    Ci,
    Chore,
    Revert,
    Unknown,
}

impl From<&str> for AngularConventionTypes {
    fn from(s: &str) -> Self {
        println!("{}", s);
        match s {
            "feat" => AngularConventionTypes::Feat,
            "fix" => AngularConventionTypes::Fix,
            "docs" => AngularConventionTypes::Docs,
            "style" => AngularConventionTypes::Style,
            "refactor" => AngularConventionTypes::Refactor,
            "perf" => AngularConventionTypes::Perf,
            "test" => AngularConventionTypes::Test,
            "build" => AngularConventionTypes::Build,
            "ci" => AngularConventionTypes::Ci,
            "chore" => AngularConventionTypes::Chore,
            "revert" => AngularConventionTypes::Revert,
            _ => {
                println!("unknown");
                AngularConventionTypes::Unknown
            }
        }
    }
}

impl<'a> From<Chars<'a>> for AngularConventionTypes {
    fn from(s: Chars) -> Self {
        AngularConventionTypes::from(s.as_str())
    }
}

impl From<String> for AngularConventionTypes {
    fn from(s: String) -> Self {
        AngularConventionTypes::from(s.as_str())
    }
}

impl From<&String> for AngularConventionTypes {
    fn from(s: &String) -> Self {
        AngularConventionTypes::from(s.as_str())
    }
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Colon,
    LPAREN,
    RPAREN,
    NewLine,
    Space,
    Exclamation,
}

impl From<Literal> for char {
    fn from(l: Literal) -> Self {
        match l {
            Literal::Colon => ':',
            Literal::LPAREN => '(',
            Literal::RPAREN => ')',
            Literal::NewLine => '\n',
            Literal::Space => ' ',
            Literal::Exclamation => '!',
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Token {
    token_type: TokenType,
    value: String,
}

impl Token {
    pub fn new(token_type: TokenType, value: String) -> Self {
        Token { token_type, value }
    }
}

pub struct Lexer {
    input: String,
    position: usize,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    CommitType,
    Scope,
    Description,
    Body,
    Footer,
    Colon,
    Exclamation,
    NewLine,
    Space,
}

#[derive(Debug, PartialEq)]
pub struct Node {
    token_type: TokenType,
    value: String,
}

impl Node {
    pub fn new(token_type: TokenType, value: String) -> Self {
        Node { token_type, value }
    }
}

fn is_commit_type(value: &str) -> Result<bool, Box<dyn Error>> {
    let commit_type = AngularConventionTypes::from(value);
    match commit_type {
        AngularConventionTypes::Unknown => Ok(false),
        _ => Ok(true),
    }
}

fn is_scope(value: &str) -> bool {
    value.starts_with("(") && value.ends_with(")")
}

fn is_description(value: &str) -> bool {
    value.starts_with(": ")
}

fn is_body(value: &str) -> bool {
    value.starts_with("\n\n")
        || value.starts_with("\n") && value.ends_with("\n\n")
        || value.ends_with("\n")
}

fn is_footer(value: &str) -> bool {
    value.starts_with("BREAKING-CHANGE:")
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Lexer { input, position: 0 }
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, Box<dyn Error>> {
        if self.position >= self.input.len() {
            return Ok(None);
        }

        let mut token_value = String::new();
        let mut token_type = TokenType::Space;

        while let Some(c) = self.input.chars().nth(self.position) {
            match c {
                ':' | '!' | '\n' | ' ' | '(' | ')' => {
                    // Delimiter or whitespace encountered, break the loop
                    self.position += 1;
                    break;
                }
                _ => {
                    // Accumulate characters for the token value
                    token_value.push(c);
                    self.position += 1;
                }
            }
        }
        if !token_value.is_empty() {
            // Determine the token type based on the accumulated value
            if is_commit_type(&token_value)? {
                token_type = TokenType::CommitType;
            } else if is_scope(&token_value) {
                token_type = TokenType::Scope;
            } else if is_description(&token_value) {
                token_type = TokenType::Description;
            } else if is_body(&token_value) {
                token_type = TokenType::Body;
            } else if is_footer(&token_value) {
                token_type = TokenType::Footer;
            }
        }

        Ok(Some(Token::new(token_type, token_value)))
    }
}

#[cfg(test)]
#[test]
fn test() {
    let input = "feat: implement a new feature".to_string();
    let mut lexer = Lexer::new(input);

    let commit = ConventionalCommit::new(
        Node::new(TokenType::CommitType, "feat".to_string()),
        None,
        Node::new(TokenType::Description, "implement a new feature".to_string()),
        None,
        None,
    );


    while let Some(token) = lexer.next_token().unwrap() {
        match token.token_type {
            TokenType::CommitType => {
                commit.token_type = TokenType::CommitType;
                commit.value.push_str(&token.value);
            }
            TokenType::Scope => {
                scope.token_type = TokenType::Scope;
                scope.value.push_str(&token.value);
            }
            TokenType::Description => {
                description.token_type = TokenType::Description;
                description.value.push_str(&token.value);
            }
            _ => {}
        }
    }

    println!("{:?}, {:?}, {:?}", commit, scope, description);
}
