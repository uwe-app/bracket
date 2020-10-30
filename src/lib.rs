pub mod error;
pub mod escape;
pub mod helper;
pub mod json;
pub mod lexer;
pub mod log;
pub mod output;
pub mod parser;
pub mod registry;
pub mod render;
pub mod template;

pub type Result<'a, T> = std::result::Result<T, error::Error<'a>>;

pub use error::Error;
pub use registry::Registry;
pub use template::Template;
