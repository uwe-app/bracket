//! Errors generated when calling helpers.
//!
//! The renderer will wrap these in `RenderError` so you only
//! need to use this type when implementing helpers.
use crate::error::IoError;
use std::fmt;

#[derive(Debug)]
pub enum HelperError {
    /// Generic error message for helpers.
    Message(String),
    /// Error when supplied arguments do not match an exact arity.
    ArityExact(String, usize),
    /// Error when supplied arguments do not match an arity range.
    ArityRange(String, usize, usize),
    /// Error when a helper expects a string argument.
    ArgumentTypeString(String, usize),
    /// Error when a helper expects an iterable (object or array).
    IterableExpected(String, usize),
    /// Proxy for render errors that occur via helpers; for example
    /// when rendering inner templates.
    Render(String),
    /// Proxy I/O errors.
    Io(IoError),
    /// Proxy JSON errors.
    Json(serde_json::Error),
}

impl fmt::Display for HelperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Message(ref name) => write!(f, "{}", name),
            Self::ArityExact(ref name, ref num) => write!(
                f,
                "Helper '{}' got invalid arity expects {} arguments(s)",
                name, num
            ),
            Self::ArityRange(ref name, ref from, ref to) => write!(
                f,
                "Helper '{}' got invalid arity expects {}-{} argument(s)",
                name, from, to
            ),
            Self::ArgumentTypeString(ref name, ref index) => write!(
                f,
                "Helper '{}' got invalid argument at index {}, string expected",
                name, index
            ),
            Self::IterableExpected(ref name, ref index) => write!(
                f,
                "Helper '{}' got invalid argument at index {}, expected array or object",
                name, index
            ),
            Self::Render(ref e) => fmt::Display::fmt(e, f),
            Self::Io(ref e) => fmt::Debug::fmt(e, f),
            Self::Json(ref e) => fmt::Debug::fmt(e, f),
        }
    }
}

impl From<std::io::Error> for HelperError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(IoError::Io(err))
    }
}

impl From<serde_json::Error> for HelperError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}
