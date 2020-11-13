//! Error types.
use std::fmt;
use thiserror::Error;

pub mod helper;
pub mod render;
pub mod source;
pub mod syntax;

pub use helper::HelperError;
pub use render::RenderError;
pub use source::{ErrorInfo, SourcePos};
pub use syntax::SyntaxError;

/// Generic error type that wraps more specific types and is
/// returned when using the `Registry`.
#[derive(Error, Eq, PartialEq)]
pub enum Error {
    #[error(transparent)]
    Syntax(#[from] SyntaxError),
    #[error(transparent)]
    Render(#[from] RenderError),
    #[error("Template not found '{0}'")]
    TemplateNotFound(String),
    #[error(transparent)]
    Io(#[from] IoError),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Syntax(ref e) => fmt::Debug::fmt(e, f),
            Self::Render(ref e) => fmt::Debug::fmt(e, f),
            Self::TemplateNotFound(_) => fmt::Display::fmt(self, f),
            Self::Io(ref e) => fmt::Debug::fmt(e, f),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(IoError::Io(err))
    }
}

/// Wrapper for IO errors that implements `PartialEq` to
/// facilitate easier testing using `assert_eq!()`.
#[derive(thiserror::Error, Debug)]
pub enum IoError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl PartialEq for IoError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(ref s), Self::Io(ref o)) => s.kind() == o.kind(),
        }
    }
}

impl Eq for IoError {}
