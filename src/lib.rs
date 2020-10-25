pub mod error;
pub mod lexer;
mod output;
mod registry;
mod render;
mod template;

pub type Result<'a, T> = std::result::Result<T, error::Error<'a>>;

pub use error::Error;
pub use registry::Registry;
pub use template::Template;
