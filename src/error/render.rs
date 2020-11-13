//! Errors generated when rendering templates.
use std::fmt;
use crate::error::{HelperError, IoError};
use thiserror::Error;

#[derive(Error)]
pub enum RenderError {
    #[error("Partial '{0}' not found")]
    PartialNotFound(String),
    #[error("Variable '{0}' not found, check the variable path and verify the template data")]
    VariableNotFound(String),
    #[error("Syntax error while evaluating path '{0}'")]
    EvaluatePath(String),
    #[error("Cycle detected whilst processing partial '{0}'")]
    PartialCycle(String),
    #[error("Cycle detected whilst processing helper '{0}'")]
    HelperCycle(String),
    #[error(transparent)]
    Helper(#[from] HelperError),
    #[error(transparent)]
    Io(#[from] IoError),
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
