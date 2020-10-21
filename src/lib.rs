mod error;
mod handlebars;
mod lexer;

pub type Result<T> = std::result::Result<T, error::Error>;

pub use error::Error;
pub use handlebars::Template;
