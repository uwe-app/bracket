//! Errors generated when rendering templates.
use std::fmt;
use crate::error::{HelperError, IoError};

pub enum RenderError<'source> {
    PartialNameResolve(&'source str),
    PartialNotFound(String),
    Helper(HelperError),
    Io(IoError),
    Json(serde_json::Error),
}

impl From<HelperError> for RenderError<'_> {
    fn from(err: HelperError) -> Self {
        Self::Helper(err)
    }
}

impl From<std::io::Error> for RenderError<'_> {
    fn from(err: std::io::Error) -> Self {
        Self::Io(IoError::Io(err))
    }
}

impl From<serde_json::Error> for RenderError<'_> {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl fmt::Display for RenderError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::PartialNameResolve(name) => {
                write!(f, "Unable to resolve partial name from '{}'", name)
            }
            Self::PartialNotFound(ref name) => {
                write!(f, "Partial '{}' not found", name)
            }
            Self::Helper(ref e) => fmt::Display::fmt(e, f),
            Self::Io(ref e) => fmt::Debug::fmt(e, f),
            Self::Json(ref e) => fmt::Debug::fmt(e, f),
        }
    }
}

impl fmt::Debug for RenderError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::PartialNameResolve(_) => fmt::Display::fmt(self, f),
            Self::PartialNotFound(_) => fmt::Display::fmt(self, f),
            Self::Helper(ref e) => fmt::Display::fmt(e, f),
            Self::Io(ref e) => fmt::Debug::fmt(e, f),
            Self::Json(ref e) => fmt::Debug::fmt(e, f),
        }
    }
}

impl PartialEq for RenderError<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PartialNotFound(ref s), Self::PartialNotFound(ref o)) => {
                s == o
            }
            _ => false,
        }
    }
}

impl Eq for RenderError<'_> {}

