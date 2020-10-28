pub mod error;
pub mod helper;
pub mod lexer;
pub mod parser;
pub mod output;
pub mod registry;
pub mod render;
pub mod template;

pub type Result<'a, T> = std::result::Result<T, error::Error<'a>>;

pub use error::Error;
pub use registry::Registry;
pub use template::Template;
