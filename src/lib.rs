mod error;
pub mod lexer;
mod registry;
mod template;

pub type Result<T> = std::result::Result<T, error::Error>;

pub use error::Error;
pub use registry::Registry;
pub use template::Template;
