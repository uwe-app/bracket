//! Error types.
use std::fmt;

pub mod helper;
pub mod source;
pub mod render;
pub mod syntax;

pub use helper::HelperError;
pub use render::RenderError;
pub use source::{SourcePos, ErrorInfo};
pub use syntax::SyntaxError;

/// Generic error type that wraps more specific types and is 
/// returned when using the `Registry`.
#[derive(Eq, PartialEq)]
pub enum Error<'source> {
    Syntax(SyntaxError),
    Render(RenderError<'source>),
    TemplateNotFound(String),
    Io(IoError),
}

impl fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Syntax(ref e) => fmt::Display::fmt(e, f),
            Self::Render(ref e) => fmt::Display::fmt(e, f),
            Self::TemplateNotFound(ref name) => {
                write!(f, "Template not found '{}'", name)
            }
            Self::Io(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl fmt::Debug for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Syntax(ref e) => fmt::Debug::fmt(e, f),
            Self::Render(ref e) => fmt::Debug::fmt(e, f),
            Self::TemplateNotFound(ref e) => fmt::Display::fmt(self, f),
            Self::Io(ref e) => fmt::Debug::fmt(e, f),
        }
    }
}

impl From<std::io::Error> for Error<'_> {
    fn from(err: std::io::Error) -> Self {
        Self::Io(IoError::Io(err))
    }
}

impl<'source> From<RenderError<'source>> for Error<'source> {
    fn from(err: RenderError<'source>) -> Self {
        Self::Render(err)
    }
}

impl<'source> From<SyntaxError> for Error<'source> {
    fn from(err: SyntaxError) -> Self {
        Self::Syntax(err)
    }
}

/// Wrapper for IO errors that implements `PartialEq` to
/// facilitate easier testing using `assert_eq!()`.
#[derive(Debug)]
pub enum IoError {
    Io(std::io::Error),
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Io(ref e) => fmt::Debug::fmt(e, f),
        }
    }
}

impl PartialEq for IoError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(ref s), Self::Io(ref o)) => s.kind() == o.kind(),
        }
    }
}

impl Eq for IoError {}
