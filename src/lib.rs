use lexer::{Lexer, Node, TokenType};

pub mod lexer;

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
    pub fn parse(s: &str) -> Self {
        let mut lexer = Lexer::new(s.to_string());

        let mut commit_type = Node::new(Some(TokenType::CommitType), None);
        let mut scope = None;
        let mut description = Node::new(Some(TokenType::Description), None);
        let mut body = None;
        let mut footer = None;

        // Process tokens within a loop
        let mut empty_count = 0;

        for idx in s.char_indices() {
            let (_, c) = idx;

            if c == '\n' {
                empty_count += 1;
            }

            if empty_count == 2 {
                break;
            }

            let node = lexer.next_token(Some(idx.0)).unwrap();

            if node.clone().unwrap().token_type == Some(TokenType::CommitType) {
                println!("node: {:?}", node);
                commit_type = node.unwrap();
            } else if node.clone().unwrap().token_type == Some(TokenType::Scope) {
                println!("node: {:?}", node);
                scope = node;
            } else if node.clone().unwrap().token_type == Some(TokenType::Description) {
                println!("node: {:?}", node);
                description = node.unwrap();
            } else if node.clone().unwrap().token_type == Some(TokenType::Body) {
                println!("node: {:?}", node);
                body = node;
            } else if node.clone().unwrap().token_type == Some(TokenType::Footer) {
                println!("node: {:?}", node);
                footer = node;
            } else {
                println!("node: {:?}", node);
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

#[cfg(test)]
#[test]
fn test() {
    let input = r#"feat(deps)!: implement a new feature

    This is a body

    BREAKING-CHANGE: this is a breaking change This is a footer
    "#;

    let commit = ConventionalCommit::parse(input);

    println!("\nConventionalCommit \ncommit_type: {:?} \nscope: {:?} \ndescription: {:?} \nbody: {:?} \nfooter: {:?}\n",
      commit.commit_type,
      commit.scope,
      commit.description,
      commit.body,
      commit.footer
    );
}
