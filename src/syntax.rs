use std::{error::Error, time::Instant};

use crate::lexer::{
    Lexer, Node, TokenType, LITERAL_COLON, LITERAL_COLONSPACE, LITERAL_EXCLAMATION, LITERAL_LPAREN,
    LITERAL_NEWLINE, LITERAL_RPAREN, LITERAL_SPACE,
};

#[derive(Debug, PartialEq)]
pub struct ConventionalCommit {
    pub commit_type: Node,
    pub scope: Option<Node>,
    pub description: Node,
    pub body: Option<Node>,
    pub footer: Option<Node>,
}

impl ConventionalCommit {
    pub fn new(input: &str) -> Result<(), Box<dyn Error>> {
        let mut lexer = Lexer::new(input.to_string());
        let now = Instant::now();
        for _ in input.chars() {
            let node = lexer.next_token(None)?;
            if node.is_some() {
                match node.clone().unwrap().token_type {
                    // Commit type
                    TokenType::CommitType => {
                        println!("Commit type: '{}'", node.unwrap().value);
                    }

                    // Scope
                    TokenType::Scope => {
                        println!("Scope: '{}'", node.unwrap().value);
                    }

                    // Description
                    TokenType::Description => {
                        println!("Description: '{}'", node.unwrap().value);
                    }

                    // Body
                    TokenType::Body => {
                        println!("Body: '{}'", node.unwrap().value);
                    }

                    // Footer
                    TokenType::Footer => {
                        println!("Footer: '{}'", node.unwrap().value);
                    }

                    // Breaking change
                    TokenType::BreakingChange => {
                        println!("Breaking change: '{}'", node.unwrap().value);
                    }

                    TokenType::Word => {
                        println!("Word: '{}'", node.unwrap().value);
                    }

                    TokenType::Colon => {
                        println!("Colon: '{}'", LITERAL_COLON);
                    }

                    TokenType::Exclamation => {
                        println!("Exclamation: '{}'", LITERAL_EXCLAMATION);
                    }

                    TokenType::NewLine => {
                        println!("Newline: '{}'", LITERAL_NEWLINE);
                    }

                    TokenType::Space => {
                        println!("Space: '{}'", LITERAL_SPACE);
                    }

                    TokenType::EOI => {
                        println!("EOI: '{}'", node.unwrap().value);
                    }

                    TokenType::LParen => {
                        println!("LParen: '{}'", LITERAL_LPAREN);
                    }

                    TokenType::RParen => {
                        println!("RParen: '{}'", LITERAL_RPAREN);
                    }

                    TokenType::ColonSpace => {
                        println!("ColonSpace: '{}'", LITERAL_COLONSPACE);
                    }

                    _ => {}
                }
            }
        }
        println!("{:?}", now.elapsed());

        Ok(())
    }
}
