use std::sync::RwLock;
use std::{error::Error, str::Chars};

const LITERAL_COLON: char = ':';
const LITERAL_LPAREN: char = '(';
const LITERAL_RPAREN: char = ')';
const LITERAL_NEWLINE: char = '\n';
const LITERAL_SPACE: char = ' ';
const LITERAL_EXCLAMATION: char = '!';

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

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    CommitType,
    Scope,
    Description,
    Body,
    LParen,
    RParen,
    Footer,
    Colon,
    Exclamation,
    NewLine,
    Space,
}

fn is_commit_type(
    value: &str,
    _prev_token: &Option<Node>,
    next_token: Option<Node>,
) -> Result<bool, Box<dyn Error>> {
    // Convert the value to AngularConventionTypes enum
    let commit_type = AngularConventionTypes::from(value);

    // Check if it's equal to AngularConventionTypes::Feat and the next token is a colon
    let is_feat = commit_type == AngularConventionTypes::Feat
        && next_token.map_or(false, |node| {
            node.token_type == TokenType::Colon
                || node.token_type == TokenType::Exclamation
                || node.token_type == TokenType::LParen
                || node.token_type == TokenType::RParen
                || node.token_type == TokenType::NewLine
        });

    Ok(is_feat)
}

fn is_scope(value: &str, prev_token: &Option<Node>, next_token: Option<Node>) -> bool {
    // A scope is considered valid if it starts with "(" and ends with ")"
    // and contains only valid characters (no spaces, colons, exclamations, or newlines)
    !value.chars().any(|c| {
        c == LITERAL_SPACE || c == LITERAL_COLON || c == LITERAL_EXCLAMATION || c == LITERAL_NEWLINE
    }) && next_token.map_or(false, |node| {
        node.token_type == TokenType::Colon
            || node.token_type == TokenType::Exclamation
            || node.token_type == TokenType::RParen
    })
}

fn is_description(value: &str, prev_token: &Option<Node>, next_token: Option<Node>) -> bool {
    !value.is_empty()
}

fn is_body(value: &str, prev_token: &Option<Node>, next_token: Option<Node>) -> bool {
    value.starts_with("\n\n")
        || value.starts_with("\n") && value.ends_with("\n\n")
        || value.ends_with("\n")
}

fn is_footer(value: &str, prev_token: &Option<Node>, next_token: Option<Node>) -> bool {
    value.starts_with("BREAKING-CHANGE:")
}

fn is_colon(value: &str) -> bool {
    value == LITERAL_COLON.to_string()
}

fn is_exclamation(value: &str) -> bool {
    value == LITERAL_EXCLAMATION.to_string()
}

fn is_newline(value: &str) -> bool {
    value == LITERAL_NEWLINE.to_string()
}

fn is_space(value: &str) -> bool {
    value == LITERAL_SPACE.to_string()
}

fn is_lparen(value: &str) -> bool {
    value == LITERAL_LPAREN.to_string()
}

fn is_rparen(value: &str) -> bool {
    value == LITERAL_RPAREN.to_string()
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    token_type: TokenType,
    value: String,
}

impl Node {
    pub fn new(token_type: TokenType, value: String) -> Self {
        Node { token_type, value }
    }

    pub fn clean(&mut self) {
        self.value = self.value.trim().to_string();
    }
}

pub struct Parser<'a> {
    input: RwLock<&'a str>,
    position: RwLock<&'a usize>,
}

impl<'a> Parser<'a> {
    fn new(input: String) -> Parser<'a> {
        Parser { 
            input: RwLock::new(&input),
            position: RwLock::new(&0), 
        }
    }

}

pub struct Lexer<'a> {
    prev_token: Option<Node>,
    parser: Parser<'a>,
}

impl Lexer<'_> {
    pub fn new(input: String) -> Self {
        Lexer {
            prev_token: None,
            parser: Parser::new(input),
        }
    }

    pub fn next_token(&self) -> Result<Option<Node>, Box<dyn Error>> {
        let input = self.parser.input.try_read()?;
        let mut position = self.parser.position.try_write()?;

        if *position >= &mut input.len() {
            return Ok(None);
        }

        let mut token_value = String::new();
        let mut token_type = TokenType::Space;

        while let Some(c) = input.chars().nth(**position) {
            match c {
                LITERAL_COLON | LITERAL_EXCLAMATION | LITERAL_NEWLINE | LITERAL_SPACE
                | LITERAL_LPAREN | LITERAL_RPAREN => {
                    // Delimiter or whitespace encountered, break the loop
                    **position += 1;
                    break;
                }
                _ => {
                    // Accumulate characters for the token value
                    token_value.push(c);
                    **position += 1;
                }
            }
        }
        if !token_value.is_empty() {
            // Determine the token type based on the accumulated value
            if is_commit_type(&token_value, &self.prev_token, self.peek_token()?)? {
                token_type = TokenType::CommitType;
            } else if is_scope(&token_value, &self.prev_token, self.peek_token()?) {
                token_type = TokenType::Scope;
            } else if is_description(&token_value, &self.prev_token, self.peek_token()?) {
                token_type = TokenType::Description;
            } else if is_body(&token_value, &self.prev_token, self.peek_token()?) {
                token_type = TokenType::Body;
            } else if is_footer(&token_value, &self.prev_token, self.peek_token()?) {
                token_type = TokenType::Footer;
            } else if is_colon(&token_value) {
                token_type = TokenType::Colon;
            } else if is_exclamation(&token_value) {
                token_type = TokenType::Exclamation;
            } else if is_newline(&token_value) {
                token_type = TokenType::NewLine;
            } else if is_space(&token_value) {
                token_type = TokenType::Space;
            } else if is_lparen(&token_value) {
                token_type = TokenType::LParen;
            } else if is_rparen(&token_value) {
                token_type = TokenType::RParen;
            } else {
                println!("Unknown token type: {:?}??", token_type);
            }
        }

        Ok(Some(Node::new(token_type, token_value)))
    }

    pub fn peek_token(&self) -> Result<Option<Node>, Box<dyn Error>> {
        let position = self.parser.position.try_read()?;
        let input = self.parser.input.try_read()?;
        let mut current_position = position.clone();
        if current_position >= input.len() {
            return Ok(None);
        }

        let mut token_value = String::new();
        let mut token_type = TokenType::Space;

        while let Some(c) = input.chars().nth(current_position) {
            match c {
                LITERAL_COLON | LITERAL_EXCLAMATION | LITERAL_NEWLINE | LITERAL_SPACE
                | LITERAL_LPAREN | LITERAL_RPAREN => {
                    // Delimiter or whitespace encountered, break the loop
                    break;
                }
                _ => {
                    // Accumulate characters for the token value
                    token_value.push(c);
                    current_position += 1;
                }
            }
        }
        if !token_value.is_empty() {
            // Determine the token type based on the accumulated value
            if is_commit_type(&token_value, &None, self.peek_token()?)? {
                token_type = TokenType::CommitType;
            } else if is_scope(&token_value, &None, self.peek_token()?) {
                token_type = TokenType::Scope;
            } else if is_description(&token_value, &None, self.peek_token()?) {
                token_type = TokenType::Description;
            } else if is_body(&token_value, &None, self.peek_token()?) {
                token_type = TokenType::Body;
            } else if is_footer(&token_value, &None, self.peek_token()?) {
                token_type = TokenType::Footer;
            } else if is_colon(&token_value) {
                token_type = TokenType::Colon;
            } else if is_exclamation(&token_value) {
                token_type = TokenType::Exclamation;
            } else if is_newline(&token_value) {
                token_type = TokenType::NewLine;
            } else if is_space(&token_value) {
                token_type = TokenType::Space;
            } else if is_lparen(&token_value) {
                token_type = TokenType::LParen;
            } else if is_rparen(&token_value) {
                token_type = TokenType::RParen;
            } else {
                println!("Unknown token type: {:?}??", token_type);
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

impl From<&str> for ConventionalCommit {
    fn from(s: &str) -> Self {
        let lexer = Lexer::new(s.to_string());

        let mut commit_type = Node::new(TokenType::CommitType, String::new());
        let mut scope = None;
        let mut description = Node::new(TokenType::Description, String::new());
        let mut body = None;
        let mut footer = None;

        let mut current_field = &mut description.clone(); // Start with the description field
                                                          //

        let t = match lexer.next_token() {
            Ok(t) => t,
            Err(e) => panic!("Error: {:?}", e),
        };

        while let Some(token) = t.clone() {
            match &token.token_type {
                TokenType::CommitType => {
                    commit_type.token_type = TokenType::CommitType;
                    commit_type.value.push_str(&token.value);
                }
                TokenType::Scope => {
                    scope = Some(Node::new(TokenType::Scope, token.value));
                }
                TokenType::Description => {
                    description.token_type = TokenType::Description;
                    description.value.push_str(&format!("{} ", token.value));
                }
                TokenType::Body => {
                    body = Some(Node::new(TokenType::Body, token.value));
                    current_field = body.as_mut().unwrap();
                }
                TokenType::Footer => {
                    footer = Some(Node::new(TokenType::Footer, token.value));
                    current_field = footer.as_mut().unwrap();
                }
                _ => {
                    current_field.value.push_str(&format!("{} ", &token.value));
                }
            }
        }

        commit_type.clean();
        description.clean();
        if let Some(body_node) = &mut body {
            body_node.clean();
        }
        if let Some(footer_node) = &mut footer {
            footer_node.clean();
        }

        ConventionalCommit {
            commit_type,
            scope,
            description,
            body,
            footer,
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
    let input = r#"feat(deps)!: implement a new feature

    This is a body

    BREAKING-CHANGE: this is a breaking change This is a footer
    "#;

    let commit = ConventionalCommit::from(input);

    println!("\nConventionalCommit \ncommit_type: {:?} \nscope: {:?} \ndescription: {:?} \nbody: {:?} \nfooter: {:?}\n",
      commit.commit_type,
      commit.scope,
      commit.description,
      commit.body,
      commit.footer
    );
}
