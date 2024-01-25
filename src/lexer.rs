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

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    pub token_type: Option<TokenType>,
    pub value: Option<String>,
}

impl Node {
    pub fn new(token_type: Option<TokenType>, value: Option<String>) -> Self {
        Node { token_type, value }
    }

    pub fn clean(&mut self) {
        self.value = Some(
            match self.value.clone() {
                Some(value) => value.trim().to_string(),
                None => LITERAL_EMPTY.to_string(),
            }
            .trim()
            .to_string(),
        );
    }

    pub fn set_token_type(&mut self, token_type: TokenType) {
        self.token_type = Some(token_type);
    }

    pub fn set_value(&mut self, value: String) {
        self.value = Some(value);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parser {
    input: String,
    position: usize,
}

impl Parser {
    pub fn new(input: String) -> Arc<Mutex<Parser>> {
        Arc::new(Mutex::new(Parser { input, position: 0 }))
    }

    pub fn get_position(&self) -> usize {
        self.position
    }
}

pub struct Lexer {
    _prev_token: Option<Node>,
    parser: Arc<Mutex<Parser>>,
    token: Node,
    _next_token: Option<Node>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Lexer {
            _prev_token: None,
            parser: Parser::new(input),
            token: Node {
                token_type: None,
                value: None,
            },
            _next_token: None,
        }
    }

    pub fn next_token(&mut self, position: Option<usize>) -> Result<Option<Node>, Box<dyn Error>> {
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
                        if next_token.token_type == Some(TokenType::Space) {
                            self.token.set_token_type(TokenType::ColonSpace);
                            self.token.set_value(LITERAL_COLONSPACE.to_string());
                            self._next_token = Some(Node::new(
                                Some(TokenType::ColonSpace),
                                Some(token_value.clone()),
                            ));
                            parser = guarded_parser.lock().unwrap();
                            break;
                        }
                    }
                    self.token.set_token_type(TokenType::Colon);
                    self.token.set_value(c.to_string());
                    self._next_token = Some(Node::new(Some(TokenType::Colon), Some(c.to_string())));
                }
                LITERAL_LPAREN => {
                    self.token.set_token_type(TokenType::LParen);
                    self.token.set_value(c.to_string());
                    self._next_token =
                        Some(Node::new(Some(TokenType::LParen), Some(c.to_string())));
                }
                LITERAL_RPAREN => {
                    self.token.set_token_type(TokenType::RParen);
                    self.token.set_value(c.to_string());
                    self._next_token =
                        Some(Node::new(Some(TokenType::RParen), Some(c.to_string())));
                }
                LITERAL_NEWLINE => {
                    self.token.set_token_type(TokenType::NewLine);
                    self.token.set_value(c.to_string());
                    self._next_token =
                        Some(Node::new(Some(TokenType::NewLine), Some(c.to_string())));
                }
                LITERAL_SPACE => {
                    self.token.set_token_type(TokenType::Space);
                    self.token.set_value(c.to_string());
                    self._next_token = Some(Node::new(Some(TokenType::Space), Some(c.to_string())));
                }
                LITERAL_EXCLAMATION => {
                    self.token.set_token_type(TokenType::Exclamation);
                    self.token.set_value(c.to_string());
                    self._next_token =
                        Some(Node::new(Some(TokenType::Exclamation), Some(c.to_string())));
                }
                LITERAL_EOI => {
                    self.token.set_token_type(TokenType::EOI);
                    self.token.set_value(c.to_string());
                    self._next_token = Some(Node::new(
                        Some(TokenType::EOI),
                        Some(LITERAL_EOI.to_string()),
                    ));
                }
                _ => {
                    token_value.push(c);
                    parser = guarded_parser.lock().unwrap();
                    parser.position += 1;
                    continue;
                }
            };

            parser = guarded_parser.lock().unwrap();
            parser.position += 1;
        }

        drop(parser);

        Ok(Some(self.token.clone()))
    }

    fn peek_token(&mut self, position: usize) -> Result<Option<Node>, Box<dyn Error>> {
        self.next_token(Some(position))
    }

    pub fn scan(&mut self, input: String) -> Result<Option<Node>, Box<dyn Error>> {
        for _ in input.chars() {
            let token = self.next_token(None)?;

            if token.is_some() {
                match token.unwrap() {
                    Node {
                        token_type: Some(TokenType::CommitType),
                        value,
                    } => {
                        self._prev_token = Some(Node::new(Some(TokenType::CommitType), value));
                    }
                    Node {
                        token_type: Some(TokenType::Scope),
                        value,
                    } => {
                        self._prev_token = Some(Node::new(Some(TokenType::Scope), value));
                    }
                    Node {
                        token_type: Some(TokenType::Description),
                        value,
                    } => {
                        self._prev_token = Some(Node::new(Some(TokenType::Description), value));
                    }
                    Node {
                        token_type: Some(TokenType::Body),
                        value,
                    } => {
                        self._prev_token = Some(Node::new(Some(TokenType::Body), value));
                    }
                    Node {
                        token_type: Some(TokenType::Footer),
                        value,
                    } => {
                        self._prev_token = Some(Node::new(Some(TokenType::Footer), value));
                    }
                    _ => {}
                }
            }
        }

        Ok(None)
    }
}
