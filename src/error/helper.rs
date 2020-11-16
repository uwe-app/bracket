//! Errors generated when calling helpers.
//!
//! The renderer will wrap these in `RenderError` so you only
//! need to use this type when implementing helpers.
use crate::error::{render::RenderError, syntax::SyntaxError, IoError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HelperError {
    /// Generic error message for helpers.
    #[error("{0}")]
    Message(String),
    /// Error when supplied arguments do not match an exact arity.
    #[error("Helper '{0}' got invalid arity expects {1} arguments(s)")]
    ArityExact(String, usize),
    /// Error when supplied arguments do not match an arity range.
    #[error("Helper '{0}' got invalid arity expects {1}-{2} argument(s)")]
    ArityRange(String, usize, usize),
    /// Error when a helper expects a string argument.
    #[error("Helper '{0}' got invalid argument at index {1}, string expected")]
    ArgumentTypeString(String, usize),
    /// Error when a helper expects an iterable (object or array).
    #[error("Helper '{0}' got invalid argument at index {1}, expected array or object")]
    IterableExpected(String, usize),

    #[error("Helper '{0}' failed to resolve field '{1}'")]
    LookupField(String, String),

    #[error("Helper '{0}' got invalid numerical operand")]
    InvalidNumericalOperand(String),

    #[error(
        "Helper '{0}' type assertion failed, expected '{1}' but got '{2}'"
    )]
    TypeAssert(String, String, String),

    /// Proxy for syntax errors that occur via helpers.
    ///
    /// For example when dynamically evaluating paths passed to
    /// the `evaluate()` function.
    #[error(transparent)]
    Syntax(#[from] SyntaxError),

    /// Proxy for render errors that occur via helpers; for example
    /// when rendering inner templates.
    #[error(transparent)]
    Render(#[from] Box<RenderError>),

    /// Proxy I/O errors.
    #[error(transparent)]
    Io(#[from] IoError),

    // Proxy JSON errors.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl HelperError {
    /// Create a new helper error with the given message.
    pub fn new(msg: &str) -> Self {
        HelperError::Message(msg.to_string()) 
    }
}

impl From<std::io::Error> for HelperError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(IoError::Io(err))
    }
}
