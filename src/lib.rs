#![deny(missing_docs)]
//#![cfg_attr(test, deny(warnings))]

pub mod error;
pub mod escape;
pub mod helper;
pub(crate) mod json;
pub mod lexer;
pub mod output;
pub mod parser;
pub mod registry;
pub mod render;
pub mod template;

/// Result type returned by the registry.
pub type Result<T> = std::result::Result<T, error::Error>;

/// Result type returned when rendering templates.
pub type RenderResult<T> = std::result::Result<T, error::RenderError>;

/// Result type returned when compiling templates.
pub type SyntaxResult<T> = std::result::Result<T, error::SyntaxError>;

pub use error::Error;
pub use registry::Registry;
pub use template::Template;

pub use escape::EscapeFn;
