use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use anyhow::Result;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("missing header line")]
    MissingHeaderLine,
    #[error("missing commit type")]
    MissingCommitType,
    #[error("invalid commit type")]
    InvalidCommitType,
    #[error("missing description")]
    MissingDescription,
    #[error("missing token")]
    MissingToken,
    #[error("invalid footer")]
    InvalidFooter,
    #[error("unknown parsing error {0}")]
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommitType {
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
    Merge,
    Release,
    Other(String),
}

impl FromStr for CommitType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "feat" => Ok(CommitType::Feat),
            "fix" => Ok(CommitType::Fix),
            "docs" => Ok(CommitType::Docs),
            "style" => Ok(CommitType::Style),
            "refactor" => Ok(CommitType::Refactor),
            "perf" => Ok(CommitType::Perf),
            "test" => Ok(CommitType::Test),
            "build" => Ok(CommitType::Build),
            "ci" => Ok(CommitType::Ci),
            "chore" => Ok(CommitType::Chore),
            "revert" => Ok(CommitType::Revert),
            "merge" => Ok(CommitType::Merge),
            "release" => Ok(CommitType::Release),
            other => Ok(CommitType::Other(other.to_string())),
        }
    }
}

impl fmt::Display for CommitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommitType::Feat => write!(f, "feat"),
            CommitType::Fix => write!(f, "fix"),
            CommitType::Docs => write!(f, "docs"),
            CommitType::Style => write!(f, "style"),
            CommitType::Refactor => write!(f, "refactor"),
            CommitType::Perf => write!(f, "perf"),
            CommitType::Test => write!(f, "test"),
            CommitType::Build => write!(f, "build"),
            CommitType::Ci => write!(f, "ci"),
            CommitType::Chore => write!(f, "chore"),
            CommitType::Revert => write!(f, "revert"),
            CommitType::Merge => write!(f, "merge"),
            CommitType::Release => write!(f, "release"),
            CommitType::Other(other) => write!(f, "{}", other),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConventionalCommit<'a> {
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub description: String,
    pub body: Option<String>,
    pub footers: Vec<Footer<'a>>,
}

impl<'a> ConventionalCommit<'a> {
    pub fn new(
        commit_type: CommitType,
        scope: Option<String>,
        description: String,
        body: Option<String>,
        footers: Vec<Footer<'a>>,
    ) -> Self {
        ConventionalCommit {
            commit_type,
            scope,
            description,
            body,
            footers,
        }
    }

    pub fn parse(message: &'a str) -> Result<Self, ParsingError> {
        let mut lines = message.lines();

        // Parse the header line.
        let header_line = lines
            .next()
            .ok_or(ParsingError::MissingHeaderLine)?;
        let mut parts = header_line.split_inclusive(":");




        let commit_type = parts
            .next()
            .ok_or(ParsingError::MissingCommitType)?
            .parse()
            .map_err(|_| ParsingError::InvalidCommitType);

        let scope = parts
            .next()
            .map(|s| s.trim_start_matches('(').trim_end_matches(')'));

        let description = parts
            .next()
            .ok_or(ParsingError::MissingDescription)?
            .trim();

        // Parse the body.
        let body = lines.next().map(|s| s.trim());

        // Parse the footers.
        let footers = lines
            .map(|line| Footer::parse(line))
            .collect::<Result<Vec<_>, ParsingError>>()?;

        Ok(ConventionalCommit {
            commit_type: commit_type?,
            scope: scope.map(|s| s.to_string()),
            description: description.to_string(),
            body: body.map(|s| s.to_string()),
            footers,
        })
    }
}

impl fmt::Display for ConventionalCommit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", self.commit_type)?;

        if let Some(scope) = &self.scope {
            writeln!(f, "({})", scope)?;
        }

        writeln!(f, "{}", self.description)?;

        if let Some(body) = &self.body {
            writeln!(f)?;
            writeln!(f, "{}", body)?;
        }

        for footer in &self.footers {
            writeln!(f)?;
            writeln!(f, "{}", footer)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Footer<'a> {
    pub token: String,
    pub value: Cow<'a, str>,
}

impl<'a> Footer<'a> {
    pub fn new(token: String, value: Cow<'a, str>) -> Self {
        Footer { token, value }
    }

    pub fn parse(line: &'a str) -> Result<Self, ParsingError> {
        let mut parts = line.splitn(2, ':');

        let token = parts
            .next()
            .ok_or(ParsingError::MissingToken)?
            .trim();
        let value = parts.next().unwrap_or("").trim();

        Ok(Footer {
            token: token.to_string(),
            value: value.into(),
        })
    }
}

impl<'a> fmt::Display for Footer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.value.is_empty() {
            write!(f, "{}", self.token)
        } else {
            write!(f, "{}: {}", self.token, self.value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commit() -> Result<()> {
        let message = "feat: add a new feature

This is a longer description of the feature.

BREAKING CHANGE: This feature breaks backwards compatibility.";

        let commit = ConventionalCommit::parse(message)?;

        assert_eq!(commit.commit_type, CommitType::Feat);
        assert_eq!(commit.scope, None);
        assert_eq!(commit.description, "add a new feature");
        assert_eq!(
            commit.body,
            Some("This is a longer description of the feature.".to_string())
        );
        assert_eq!(
            commit.footers,
            vec![Footer::new(
                "BREAKING CHANGE".to_string(),
                "This feature breaks backwards compatibility.".into()
            )]
        );
        Ok(())
    }

    #[test]
    fn test_parse_commit_with_scope() -> Result<()> {
        let message = "feat(parser): add a new feature to the parser

This is a longer description of the feature.";

        let commit = ConventionalCommit::parse(message)?;

        assert_eq!(commit.commit_type, CommitType::Feat);
        assert_eq!(commit.scope, Some("parser".to_string()));
        assert_eq!(commit.description, "add a new feature to the parser");
        assert_eq!(
            commit.body,
            Some("This is a longer description of the feature.".to_string())
        );
        assert_eq!(commit.footers, vec![]);
        Ok(())
    }

    #[test]
    fn test_parse_commit_with_body() -> Result<()> {
        let message = "feat: add a new feature

This is a longer description of the feature.

BREAKING CHANGE: This feature breaks backwards compatibility.

See #1234 for more details.";

        let commit = ConventionalCommit::parse(message)?;

        assert_eq!(commit.commit_type, CommitType::Feat);
        assert_eq!(commit.scope, None);
        assert_eq!(commit.description, "add a new feature");
        assert_eq!(
            commit.body,
            Some("This is a longer description of the feature.".to_string())
        );
        assert_eq!(
            commit.footers,
            vec![
                Footer::new(
                    "BREAKING CHANGE".to_string(),
                    "This feature breaks backwards compatibility.".into()
                ),
                Footer::new("See".to_string(), "#1234".into())
            ]
        );
        Ok(())
    }

    #[test]
    fn test_parse_commit_with_multiple_footers() -> Result<()> {
        let message = "feat: add a new feature

This is a longer description of the feature.

BREAKING CHANGE: This feature breaks backwards compatibility.

See #1234 for more details.

Acked-by: John Smith <john.smith@example.com>";

        let commit = ConventionalCommit::parse(message)?;

        assert_eq!(commit.commit_type, CommitType::Feat);
        assert_eq!(commit.scope, None);
        assert_eq!(commit.description, "add a new feature");
        assert_eq!(
            commit.body,
            Some("This is a longer description of the feature.".to_string())
        );
        assert_eq!(
            commit.footers,
            vec![
                Footer::new(
                    "BREAKING CHANGE".to_string(),
                    "This feature breaks backwards compatibility.".into()
                ),
                Footer::new("See".to_string(), "#1234".into()),
                Footer::new(
                    "Acked-by".to_string(),
                    "John Smith <john.smith@example.com>".into()
                )
            ]
        );
        Ok(())
    }

    #[test]
    fn test_parse_commit_with_breaking_change_in_header() -> Result<()> {
        let message = "feat!: add a new feature

This is a longer description of the feature.";

        let commit = ConventionalCommit::parse(message)?;

        assert_eq!(commit.commit_type, CommitType::Feat);
        assert_eq!(commit.scope, None);
        assert_eq!(commit.description, "add a new feature");
        assert_eq!(
            commit.body,
            Some("This is a longer description of the feature.".to_string())
        );
        assert_eq!(
            commit.footers,
            vec![Footer::new("BREAKING CHANGE".to_string(), "".into())]
        );
        Ok(())
    }

    #[test]
    fn test_parse_commit_with_breaking_change_in_footer() -> Result<()> {
        let message = "feat: add a new feature

This is a longer description of the feature.

BREAKING CHANGE: This feature breaks backwards compatibility.";

        let commit = ConventionalCommit::parse(message)?;

        assert_eq!(commit.commit_type, CommitType::Feat);
        assert_eq!(commit.scope, None);
        assert_eq!(commit.description, "add a new feature");
        assert_eq!(
            commit.body,
            Some("This is a longer description of the feature.".to_string())
        );
        assert_eq!(
            commit.footers,
            vec![Footer::new(
                "BREAKING CHANGE".to_string(),
                "This feature breaks backwards compatibility.".into()
            )]
        );
        Ok(())
    }

    #[test]
    fn test_parse_commit_with_invalid_header() -> Result<()> {
        let message = "invalid: add a new feature

This is a longer description of the feature.";

        let result = ConventionalCommit::parse(message);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_parse_commit_with_missing_description() {
        let message = "feat:";

        let result = ConventionalCommit::parse(message);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_commit_with_invalid_footer() -> Result<()> {
        let message = "feat: add a new feature

This is a longer description of the feature.

INVALID: This is an invalid footer.";

        let result = ConventionalCommit::parse(message);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_format_commit() -> Result<()> {
        let commit = ConventionalCommit {
            commit_type: CommitType::Feat,
            scope: Some("parser".to_string()),
            description: "add a new feature to the parser".to_string(),
            body: Some("This is a longer description of the feature.".to_string()),
            footers: vec![
                Footer::new(
                    "BREAKING CHANGE".to_string(),
                    "This feature breaks backwards compatibility.".into(),
                ),
                Footer::new("See".to_string(), "#1234".into()),
            ],
        };

        let expected_message = "feat(parser): add a new feature to the parser

This is a longer description of the feature.

BREAKING CHANGE: This feature breaks backwards compatibility.

See #1234";

        assert_eq!(commit.to_string(), expected_message);
        Ok(())
    }
}
