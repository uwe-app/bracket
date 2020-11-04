//! Errors generated when compiling templates.
use std::fmt;

static SYNTAX_PREFIX: &str = "Syntax error";

#[derive(Eq, PartialEq)]
pub enum SyntaxError {
    EmptyStatement(String),
    ExpectedIdentifier(String),
    ExpectedSimpleIdentifier(String),
    PartialIdentifier(String),
    PartialSimpleIdentifier(String),
    BlockIdentifier(String),
    OpenStatement(String),
    StringLiteralNewline(String),
    UnexpectedPathExplicitThis(String),
    UnexpectedPathParent(String),
    UnexpectedPathLocal(String),
    UnexpectedPathDelimiter(String),
    UnexpectedPathParentWithLocal(String),
    UnexpectedPathParentWithExplicit(String),
    ExpectedPathDelimiter(String),
    OpenSubExpression(String),
    TagNameMismatch(String),
    BlockNotOpen(String),
}

impl SyntaxError {
    fn message(&self) -> &'static str {
        match *self {
            Self::EmptyStatement(_) => "statement is empty",
            Self::ExpectedIdentifier(_) => "expecting identifier",
            Self::ExpectedSimpleIdentifier(_) => {
                "expecting identifier not a path or sub-expression"
            }
            Self::PartialIdentifier(_) => "partial requires an identifier",
            Self::PartialSimpleIdentifier(_) => {
                "partial requires a simple identifier (not a path)"
            }
            Self::BlockIdentifier(_) => "block scope requires an identifier",
            Self::OpenStatement(_) => "statement not terminated",
            Self::StringLiteralNewline(_) => {
                "new lines in string literals must be escaped (\\n)"
            }
            Self::UnexpectedPathExplicitThis(_) => {
                "explicit this reference must be at the start of a path"
            }
            Self::UnexpectedPathParent(_) => {
                "parent scopes must be at the start of a path"
            }
            Self::UnexpectedPathLocal(_) => {
                "local scope identifiers must be at the start of a path"
            }
            Self::UnexpectedPathDelimiter(_) => {
                "expected identifier but got path delimiter"
            }
            Self::UnexpectedPathParentWithLocal(_) => {
                "parent scopes and local identifiers are mutually exclusive"
            }
            Self::UnexpectedPathParentWithExplicit(_) => {
                "parent scopes and explicit this are mutually exclusive"
            }
            Self::ExpectedPathDelimiter(_) => "expected path delimiter (.)",
            Self::OpenSubExpression(_) => "sub-expression not terminated",
            Self::TagNameMismatch(_) => "closing name does not match",
            Self::BlockNotOpen(_) => "got a closing tag but no block is open",
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", SYNTAX_PREFIX, self.message())
    }
}

impl fmt::Debug for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {}\n", self.to_string())?;
        match *self {
            Self::EmptyStatement(ref source)
            | Self::ExpectedIdentifier(ref source)
            | Self::ExpectedSimpleIdentifier(ref source)
            | Self::PartialIdentifier(ref source)
            | Self::PartialSimpleIdentifier(ref source)
            | Self::BlockIdentifier(ref source)
            | Self::OpenStatement(ref source)
            | Self::StringLiteralNewline(ref source)
            | Self::UnexpectedPathExplicitThis(ref source)
            | Self::UnexpectedPathParent(ref source)
            | Self::UnexpectedPathLocal(ref source)
            | Self::UnexpectedPathDelimiter(ref source)
            | Self::UnexpectedPathParentWithLocal(ref source)
            | Self::UnexpectedPathParentWithExplicit(ref source)
            | Self::ExpectedPathDelimiter(ref source)
            | Self::OpenSubExpression(ref source)
            | Self::TagNameMismatch(ref source)
            | Self::BlockNotOpen(ref source) => write!(f, "{}", source)?,
        }
        Ok(())
    }
}
