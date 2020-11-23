//! Errors generated when compiling templates.
use std::fmt;
use thiserror::Error;

/// Errors generated when compiling a template.
#[derive(Error, Eq, PartialEq)]
pub enum SyntaxError {
    /// Error when an identifier is expected.
    #[error("Syntax error, expecting identifier")]
    ExpectedIdentifier(String),

    /// Error when a path is expected.
    #[error("Syntax error, expecting path")]
    ExpectedPath(String),

    /// Error if a block name is not a simple identifier.
    #[error("Syntax error, block name must be an identifier")]
    BlockName(String),

    /// Error when a newline is enccountered in a raw literal.
    #[error("Syntax error, new lines in raw literals must be escaped (\\n)")]
    LiteralNewline(String),

    /// Error when the partial operator is not the first token in a call statement.
    #[error("Syntax error, partial operator (>) must come first")]
    PartialPosition(String),

    /// Error when a sub-expression is closed by no sub-expression is open.
    #[error(
        "Syntax error, got close sub-expression but no sub-expression is open"
    )]
    SubExprNotOpen(String),

    /// Error when a sub-expression attempts to use a sub-expression for it's target.
    ///
    /// Currently sub-expressions for the target are only supported when evaluating
    /// partials.
    #[error(
        "Syntax error, sub-expression must use an identifier for the target"
    )]
    SubExprTargetNotAllowed(String),

    /// Error when a path delimiter is encountered in an invalid position.
    #[error("Syntax error, path delimiter (.) not allowed here")]
    PathDelimiterNotAllowed(String),

    /// Error when the `else` keyword is encountered in an invalid position.
    #[error("Syntax error, 'else' keyword is not allowed here")]
    ElseNotAllowed(String),

    /// Error when the `this` keywords is not at the start of a path.
    #[error(
        "Syntax error, explicit this reference must be at the start of a path"
    )]
    UnexpectedPathExplicitThis(String),

    /// Error when a parent path reference (../) is not at the start of a path.
    #[error("Syntax error, parent scopes must be at the start of a path")]
    UnexpectedPathParent(String),

    /// Error when a local identifier is not at the start of a path.
    #[error(
        "Syntax error, local scope identifiers must be at the start of a path"
    )]
    UnexpectedPathLocal(String),

    /// Error when an identifier is expected but a path delimiter was encountered.
    #[error("Syntax error, expected identifier but got path delimiter")]
    UnexpectedPathDelimiter(String),

    /// Error when parent scope references and local identifiers are combined illegally.
    #[error("Syntax error, parent scopes and local identifiers are mutually exclusive")]
    UnexpectedPathParentWithLocal(String),

    /// Error attempting to mix parent scope references and explicit this.
    #[error(
        "Syntax error, parent scopes and explicit this are mutually exclusive"
    )]
    UnexpectedPathParentWithExplicit(String),

    /// Error when a path delimiter is expected.
    #[error("Syntax error, expected path delimiter (.)")]
    ExpectedPathDelimiter(String),

    /// Error when a sub-expression was not terminated.
    #[error("Syntax error, sub-expression not terminated")]
    OpenSubExpression(String),

    /// Error when a closing tag name does not match the opening name.
    #[error("Syntax error, closing name does not match")]
    TagNameMismatch(String),

    /// Error when an end tag is encountered but no block is open.
    #[error("Syntax error, got a closing tag but no block is open")]
    BlockNotOpen(String),

    /// Error when a sub-expression is not terminated.
    #[error("Syntax error, sub-expression was not terminated")]
    SubExpressionNotTerminated(String),
    /// Erro when a link is not terminated.
    #[error("Syntax error, link was not terminated")]
    LinkNotTerminated(String),

    /// Error when the opening tag for a raw block is not terminated.
    #[error("Syntax error, raw block open tag was not terminated")]
    RawBlockOpenNotTerminated(String),

    /// Error when a raw block is not terminated.
    #[error("Syntax error, raw block was not terminated")]
    RawBlockNotTerminated(String),
    /// Error when a raw comment is not terminated.
    #[error("Syntax error, raw comment was not terminated")]
    RawCommentNotTerminated(String),
    /// Error when a raw statement is not terminated.
    #[error("Syntax error, raw statement was not terminated")]
    RawStatementNotTerminated(String),
    /// Error when a comment is not terminated.
    #[error("Syntax error, comment was not terminated")]
    CommentNotTerminated(String),

    /// Error attempting to use a sub-expression outside of a partial target context.
    #[error("Syntax error, block target sub expressions are only supported for partials")]
    BlockTargetSubExpr(String),
    /// Error when an empty path is encountered.
    #[error("Syntax error, path is empty")]
    EmptyPath(String),
    /// Error if we could not identify the type of a path component (internal error).
    #[error("Syntax error, path component type could not be identified")]
    ComponentType(String),
    /// Error attempting to combine partials with conditionals.
    #[error("Syntax error, partials and conditionals may not be combined")]
    MixedPartialConditional(String),

    /// Invalid token error (internal error).
    #[error("Syntax error, unexpected error token for context '{0}'")]
    TokenError(String, String),
    /// Invalid token error (internal error).
    #[error("Syntax error, expecting path or sub-expression for call target")]
    TokenCallTarget(String),
    /// Invalid token error (internal error).
    #[error("Syntax error, expecting JSON literal token")]
    TokenJsonLiteral(String),
    /// Invalid token error (internal error).
    #[error("Syntax error, expecting parameter token")]
    TokenParameter(String),
    /// Invalid token error (internal error).
    #[error("Syntax error, expecting key/value token")]
    TokenHashKeyValue(String),
    /// Invalid token error (internal error).
    #[error("Syntax error, expecting raw literal token")]
    TokenRawLiteral(String),
    /// Invalid token error (internal error).
    #[error("Syntax error, unexpected token parsing quoted literal (\"\")")]
    TokenDoubleQuoteLiteral(String),
    /// Invalid token error (internal error).
    #[error("Syntax error, unexpected token parsing quoted literal ('')")]
    TokenSingleQuoteLiteral(String),
    /// Invalid token error (internal error).
    #[error("Syntax error, unexpected token parsing quoted literal ([])")]
    TokenArrayLiteral(String),
    /// Invalid token error (internal error).
    #[error("Syntax error, unexpected token parsing link")]
    TokenLink(String),
    /// Invalid token error (internal error).
    #[error("Syntax error, unexpected token parsing path")]
    TokenParameterPath(String),
    /// Invalid token error (internal error).
    #[error("Syntax error, unexpected token, expecting end of raw block")]
    TokenEndRawBlock(String),
}

impl fmt::Debug for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", self.to_string())?;
        match *self {
            Self::ExpectedIdentifier(ref source)
            | Self::ExpectedPath(ref source)
            | Self::BlockName(ref source)
            | Self::LiteralNewline(ref source)
            | Self::PartialPosition(ref source)
            | Self::SubExprNotOpen(ref source)
            | Self::SubExprTargetNotAllowed(ref source)
            | Self::PathDelimiterNotAllowed(ref source)
            | Self::ElseNotAllowed(ref source)
            | Self::UnexpectedPathExplicitThis(ref source)
            | Self::UnexpectedPathParent(ref source)
            | Self::UnexpectedPathLocal(ref source)
            | Self::UnexpectedPathDelimiter(ref source)
            | Self::UnexpectedPathParentWithLocal(ref source)
            | Self::UnexpectedPathParentWithExplicit(ref source)
            | Self::ExpectedPathDelimiter(ref source)
            | Self::OpenSubExpression(ref source)
            | Self::TagNameMismatch(ref source)
            | Self::SubExpressionNotTerminated(ref source)
            | Self::LinkNotTerminated(ref source)
            | Self::RawBlockNotTerminated(ref source)
            | Self::RawCommentNotTerminated(ref source)
            | Self::RawStatementNotTerminated(ref source)
            | Self::CommentNotTerminated(ref source)
            | Self::BlockTargetSubExpr(ref source)
            | Self::EmptyPath(ref source)
            | Self::ComponentType(ref source)
            | Self::MixedPartialConditional(ref source)
            | Self::RawBlockOpenNotTerminated(ref source)
            | Self::TokenError(_, ref source)
            | Self::TokenCallTarget(ref source)
            | Self::TokenJsonLiteral(ref source)
            | Self::TokenParameter(ref source)
            | Self::TokenHashKeyValue(ref source)
            | Self::TokenRawLiteral(ref source)
            | Self::TokenDoubleQuoteLiteral(ref source)
            | Self::TokenSingleQuoteLiteral(ref source)
            | Self::TokenArrayLiteral(ref source)
            | Self::TokenLink(ref source)
            | Self::TokenParameterPath(ref source)
            | Self::TokenEndRawBlock(ref source)
            | Self::BlockNotOpen(ref source) => write!(f, "{}", source)?,
        }
        Ok(())
    }
}
