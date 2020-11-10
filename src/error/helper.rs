//! Errors generated when calling helpers.
//!
//! The renderer will wrap these in `RenderError` so you only
//! need to use this type when implementing helpers.
use crate::error::{render::RenderError, IoError};
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

impl From<std::io::Error> for HelperError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(IoError::Io(err))
    }
}
