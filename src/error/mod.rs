//! Error types.
use std::fmt;

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
#[derive(Eq, PartialEq)]
pub enum Error {
    Syntax(SyntaxError),
    Render(RenderError),
    TemplateNotFound(String),
    Io(IoError),
}

impl fmt::Display for Error {
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

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Syntax(ref e) => fmt::Debug::fmt(e, f),
            Self::Render(ref e) => fmt::Debug::fmt(e, f),
            Self::TemplateNotFound(ref e) => fmt::Display::fmt(self, f),
            Self::Io(ref e) => fmt::Debug::fmt(e, f),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(IoError::Io(err))
    }
}

impl From<RenderError> for Error {
    fn from(err: RenderError) -> Self {
        Self::Render(err)
    }
}

impl From<SyntaxError> for Error {
    fn from(err: SyntaxError) -> Self {
        Self::Syntax(err)
    }
}

/// Wrapper for IO errors that implements `PartialEq` to
/// facilitate easier testing using `assert_eq!()`.
#[derive(thiserror::Error, Debug)]
pub enum IoError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

//impl fmt::Display for IoError {
    //fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //match *self {
            //Self::Io(ref e) => fmt::Debug::fmt(e, f),
        //}
    //}
//}

impl PartialEq for IoError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(ref s), Self::Io(ref o)) => s.kind() == o.kind(),
        }
    }
}

impl Eq for IoError {}
