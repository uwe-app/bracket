pub mod error;
pub mod escape;
pub mod helper;
pub(crate) mod json;
pub mod lexer;
#[cfg(feature = "log-helper")]
pub mod log;
pub mod output;
pub mod parser;
pub mod registry;
pub mod render;
pub mod template;

pub type Result<'a, T> = std::result::Result<T, error::Error<'a>>;
pub type RenderResult<'a, T> = std::result::Result<T, error::RenderError<'a>>;

pub(crate) use error::Error;
pub use registry::Registry;
pub use template::Template;

pub use escape::EscapeFn;
