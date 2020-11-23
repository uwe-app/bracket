//! Errors generated when rendering templates.
use crate::error::{HelperError, IoError};
use std::fmt;
use thiserror::Error;

/// Errors generated during rendering.
#[derive(Error)]
pub enum RenderError {
    /// Error when a partial could not be found.
    #[error("Partial '{0}' not found")]
    PartialNotFound(String),

    /// Error when a variable could not be resolved.
    #[error("Variable '{0}' not found, check the variable path and verify the template data")]
    VariableNotFound(String),

    /// Error when a helper could not be found.
    #[error("Helper '{0}' not found, check the name")]
    HelperNotFound(String),

    /// Error when evaluating a path and a syntax error occurs.
    ///
    /// Paths can be dynamically evaluated when the
    /// [evaluate()](crate::render::Render#method.evaluate) function is called
    /// inside a helper.
    #[error("Syntax error while evaluating path '{0}'")]
    EvaluatePath(String),

    /// Error when a cycle is detected whilst handling a partial.
    #[error("Cycle detected whilst processing partial '{0}'")]
    PartialCycle(String),

    /// Error when a cycle is detected whilst handling a helper.
    #[error("Cycle detected whilst processing helper '{0}'")]
    HelperCycle(String),

    /// Error when a partial is not a simple identifier.
    #[error("Partial names must be simple identifiers, got path '{0}'")]
    PartialIdentifier(String),
    /// Error when a block is not a simple identifier.
    #[error("Block names must be simple identifiers, got path '{0}'")]
    BlockIdentifier(String),
    /// Error attempting to invoke a sub-expression outside of a partial target context.
    #[error("Block target sub expressions are only supported for partials")]
    BlockTargetSubExpr,

    /// Wrap a helper error.
    #[error(transparent)]
    Helper(#[from] HelperError),

    /// Proxy for IO errors.
    #[error(transparent)]
    Io(#[from] IoError),

    /// Proxy for JSON errors.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl fmt::Debug for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
        // TODO: support source code snippets
    }
}

impl From<std::io::Error> for RenderError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(IoError::Io(err))
    }
}

impl PartialEq for RenderError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PartialNotFound(ref s), Self::PartialNotFound(ref o)) => {
                s == o
            }
            _ => false,
        }
    }
}

impl Eq for RenderError {}
