use std::{error::Error, str::Chars};
use std::collections::HashMap;

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
            _ => AngularConventionTypes::Unknown,
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
    !value.is_empty()
}

fn is_body(value: &str) -> bool {
    value.starts_with("\n\n")
        || value.starts_with("\n") && value.ends_with("\n\n")
        || value.ends_with("\n")
}

fn is_footer(value: &str) -> bool {
    value.starts_with("BREAKING-CHANGE:")
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

pub struct Lexer {
    input: String,
    position: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Lexer { input, position: 0 }
    }

    pub fn next_token(&mut self) -> Result<Option<Node>, Box<dyn Error>> {
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
            } else {
                token_type = TokenType::Space; // TODO: This is a hack, fix it
                println!("Unknown token type: {}", token_value);
            }
        }

        Ok(Some(Node::new(token_type, token_value)))
    }
}

///------------ Conventional Commit ------------ ///
///

#[derive(Debug, PartialEq)]
pub struct ConventionalCommit {
    pub commit_type: Node,
    pub scope: Option<Node>,
    pub description: Node,
    pub body: Option<Node>,
    pub footer: Option<Node>,
}

impl ConventionalCommit {
    fn from(s: &str) -> Self {

        let mut _graph: HashMap<i8, TokenType> = HashMap::new(); // maybe track the order of the tokens in the commit message here as well as the token type like a graph or something? idk

        let mut lexer = Lexer::new(s.to_string());

        let mut commit_type = Node::new(TokenType::CommitType, String::new());
        let mut scope = Node::new(TokenType::Scope, String::new());
        let mut description = Node::new(TokenType::Description, String::new());
        let mut body = Node::new(TokenType::Body, String::new());
        let mut footer = Node::new(TokenType::Footer, String::new());

        let mut current_token_type = TokenType::CommitType;

        while let Some(token) = lexer.next_token().unwrap() {
            if token.token_type == TokenType::Space {
                continue; // Ignore spaces
            }

            if token.token_type == TokenType::NewLine {
                if current_token_type == TokenType::Description {
                    current_token_type = TokenType::Body;
                }
                continue; // Ignore newlines
            }

            match current_token_type {
                TokenType::CommitType => {
                    commit_type.token_type = TokenType::CommitType;
                    commit_type.value.push_str(&token.value);
                }
                TokenType::Scope => {
                    scope.token_type = TokenType::Scope;
                    scope.value.push_str(&token.value);
                }
                TokenType::Description => {
                    description.token_type = TokenType::Description;
                    description.value.push_str(&token.value);
                }
                TokenType::Body => {
                    body.token_type = TokenType::Body;
                    body.value.push_str(&token.value);
                }
                TokenType::Footer => {
                    footer.token_type = TokenType::Footer;
                    footer.value.push_str(&token.value);
                }
                _ => {}
            }
        }

        ConventionalCommit {
            commit_type,
            scope: Some(scope),
            description,
            body: Some(body),
            footer: Some(footer),
        }
    }
}

impl From<String> for ConventionalCommit {
    fn from(s: String) -> Self {
        ConventionalCommit::from(s.as_str())
    }
}

impl ConventionalCommit {}

#[cfg(test)]
#[test]
fn test() {
    let input = "feat: implement a new feature";

    let commit = ConventionalCommit::from(input);

    assert_eq!(commit.commit_type.value, "feat");
    assert_eq!(commit.scope.unwrap().value, "");
    assert_eq!(commit.description.value, "implement a new feature");
    assert_eq!(commit.body.unwrap().value, "");
    assert_eq!(commit.footer.unwrap().value, "");
}
