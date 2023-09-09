use std::{
    error::Error,
    str::Chars,
    sync::{Arc, Mutex},
};

pub const LITERAL_COLON: char = ':';
pub const LITERAL_LPAREN: char = '(';
pub const LITERAL_RPAREN: char = ')';
pub const LITERAL_NEWLINE: char = '\n';
pub const LITERAL_SPACE: char = ' ';
pub const LITERAL_EXCLAMATION: char = '!';
pub const LITERAL_EOI: char = '\0';
pub const LITERAL_EMPTY: &str = "";
pub const LITERAL_COLONSPACE: &str = ": ";
pub const SYMBOL_BREAKING_CHANGE: &str = "BREAKING-CHANGE:";

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
    ColonSpace,
    Word,
    Exclamation,
    NewLine,
    Space,
    EOI,
    BreakingChange,
    Empty,
}

fn is_word(value: &str, _prev_token: &Option<Node>, _next_token: &Option<Node>) -> bool {
    !value.is_empty()
}

fn is_commit_type(value: &str, prev_token: &Option<Node>, next_token: &Option<Node>) -> bool {
    is_word(value, prev_token, next_token)
        && AngularConventionTypes::from(value) != AngularConventionTypes::Unknown
        && match next_token {
            Some(token) => {
                token.token_type == TokenType::LParen
                    || token.token_type == TokenType::Colon
                    || token.token_type != TokenType::Exclamation
                    || token.token_type != TokenType::NewLine
                    || token.token_type != TokenType::Space
            }
            None => false,
        }
        && prev_token.is_none()
}

fn is_scope(value: &str, prev_token: &Option<Node>, next_token: &Option<Node>) -> bool {
    is_word(value, prev_token, next_token)
        && match prev_token {
            Some(token) => token.token_type == TokenType::LParen,
            None => false,
        }
        && match next_token {
            Some(token) => token.token_type == TokenType::RParen,
            None => false,
        }
}

fn is_description(value: &str, prev_token: &Option<Node>, _next_token: &Option<Node>) -> bool {
    is_word(value, prev_token, &None)
        && prev_token
            .clone()
            .map_or(false, |token| token.token_type == TokenType::ColonSpace)
}

fn is_body(value: &str, _prev_token: &Option<Node>, _next_token: &Option<Node>) -> bool {
    is_word(value, &None, &None)
        && value.starts_with("\n")
        && value.ends_with("\n")
        && !value.starts_with("\n\n")
        && !value.ends_with("\n\n")
}

fn is_footer(value: &str, _prev_token: &Option<Node>, _next_token: &Option<Node>) -> bool {
    value.starts_with(SYMBOL_BREAKING_CHANGE)
}

fn is_colonspace(value: &str) -> bool {
    value == LITERAL_COLONSPACE.to_string()
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
    pub token_type: TokenType,
    pub value: String,
}

impl Node {
    pub fn new(token_type: TokenType, value: String) -> Self {
        Node { token_type, value }
    }

    pub fn clean(&mut self) {
        self.value = self.value.trim().to_string();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parser {
    input: String,
    position: usize,
}

impl Parser {
    fn new(input: String) -> Arc<Mutex<Parser>> {
        Arc::new(Mutex::new(Parser { input, position: 0 }))
    }
}

pub struct Lexer {
    _prev_token: Option<Node>,
    parser: Arc<Mutex<Parser>>,
    _next_token: Option<Node>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Lexer {
            _prev_token: None,
            parser: Parser::new(input),
            _next_token: None,
        }
    }

    pub fn next_token(&mut self, position: Option<usize>) -> Result<Option<Node>, Box<dyn Error>> {
        let mut token_type = TokenType::Empty;
        let mut token_value = String::new();

        // Loop through the input string
        // and accumulate characters until we reach a delimiter
        // or the end of the string
        let guarded_parser = self.parser.clone();

        let mut parser = guarded_parser.lock().unwrap();

        if position.is_some() {
            parser.position = position.unwrap();
        }

        while let Some(c) = parser.input.chars().nth(parser.position) {
            let next_position = parser.position + 1;

            drop(parser);

            match c {
                LITERAL_COLON => {
                    let next_token = self.peek_token(next_position)?;
                    if next_token.is_some() {
                        let next_token = next_token.unwrap();
                        if next_token.token_type == TokenType::Space {
                            token_type = TokenType::ColonSpace;
                            self._next_token =
                                Some(Node::new(TokenType::ColonSpace, token_value.clone()));
                            parser = guarded_parser.lock().unwrap();
                            break;
                        }
                    }
                    token_type = TokenType::Colon;
                    self._next_token = Some(Node::new(TokenType::Colon, c.to_string()));
                }
                LITERAL_LPAREN => {
                    token_type = TokenType::LParen;
                    self._next_token = Some(Node::new(TokenType::LParen, c.to_string()));
                }
                LITERAL_RPAREN => {
                    token_type = TokenType::RParen;
                    self._next_token = Some(Node::new(TokenType::RParen, c.to_string()));
                }
                LITERAL_NEWLINE => {
                    token_type = TokenType::NewLine;
                    self._next_token = Some(Node::new(TokenType::NewLine, c.to_string()));
                }
                LITERAL_SPACE => {
                    token_type = TokenType::Space;
                    self._next_token = Some(Node::new(TokenType::Space, c.to_string()));
                }
                LITERAL_EXCLAMATION => {
                    token_type = TokenType::Exclamation;
                    self._next_token = Some(Node::new(TokenType::Exclamation, c.to_string()));
                }
                LITERAL_EOI => {
                    token_type = TokenType::EOI;
                    self._next_token = Some(Node::new(TokenType::EOI, LITERAL_EOI.to_string()));
                }
                _ => {
                    token_value.push(c);
                    parser = guarded_parser.lock().unwrap();
                    parser.position += 1;
                    continue;
                }
            };

            token_value.push(c);
            parser = guarded_parser.lock().unwrap();
            parser.position += 1;
            break;
        }

        drop(parser);

        match &token_value {
            LITERAL_BREAKING_CHANGE => {
                token_type = TokenType::BreakingChange;
                self._next_token = Some(Node::new(
                    TokenType::BreakingChange,
                    token_value.to_string(),
                ))
            }
            _ => {}
        };

        // Determine the token type based on the accumulated value
        if is_colonspace(&token_value) {
            token_type = TokenType::ColonSpace;
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
        } else if is_scope(&token_value, &self._prev_token, &self._next_token) {
            token_type = TokenType::Scope;
        } else if is_commit_type(&token_value, &self._prev_token, &self._next_token) {
            token_type = TokenType::CommitType;
        } else if is_description(&token_value, &self._prev_token, &self._next_token) {
            token_type = TokenType::Description;
        } else if is_body(&token_value, &self._prev_token, &self._next_token) {
            token_type = TokenType::Body;
        } else if is_footer(&token_value, &self._prev_token, &self._next_token) {
            token_type = TokenType::Footer;
        } else if is_word(&token_value, &self._prev_token, &self._next_token) {
            token_type = TokenType::Word;
        } else {
            token_type = TokenType::Empty;
        }

        Ok(Some(Node::new(token_type, token_value)))
    }

    fn peek_token(&mut self, position: usize) -> Result<Option<Node>, Box<dyn Error>> {
        self.next_token(Some(position))
    }

    pub fn scan(&mut self) -> Result<Option<Node>, Box<dyn Error>> {
        let parser = self.parser.lock().unwrap();
        let input = parser.input.clone();
        drop(parser);

        for _ in input.chars() {
            let token = self.next_token(None)?;

            if token.is_some() {
                match token.unwrap() {
                    Node {
                        token_type: TokenType::CommitType,
                        value,
                    } => {
                        self._prev_token = Some(Node::new(TokenType::CommitType, value));
                    }
                    Node {
                        token_type: TokenType::Scope,
                        value,
                    } => {
                        self._prev_token = Some(Node::new(TokenType::Scope, value));
                    }
                    Node {
                        token_type: TokenType::Description,
                        value,
                    } => {
                        self._prev_token = Some(Node::new(TokenType::Description, value));
                    }
                    Node {
                        token_type: TokenType::Body,
                        value,
                    } => {
                        self._prev_token = Some(Node::new(TokenType::Body, value));
                    }
                    Node {
                        token_type: TokenType::Footer,
                        value,
                    } => {
                        self._prev_token = Some(Node::new(TokenType::Footer, value));
                    }
                    _ => {}
                }
            }
        }

        Ok(None)
    }
}
