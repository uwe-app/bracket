//! Bracket is a fast and correct implementation of the handlebars
//! general purpose template engine.
//!
//! It is designed to keep allocations to a minimum by using pointers
//! and string slices into the underlying template wherever possible.
//!
//! The lexer generates a stream of tokens which are consumed by a
//! parser that transforms them into AST nodes. These nodes can then
//! be stored as compiled templates or passed directly to a renderer.
//!
//! The goal is to be 100% compatible with the Javascript handlebars
//! implementation; if you notice a discrepancy please report it as
//! a bug.
//!
//! The main public API is accessed using a [Registry](registry::Registry)
//! which can be used for compiling, rendering, registering partials and
//! configuring helpers.
//!
//! Errors generated during compilation are of the
//! [SyntaxError](error::SyntaxError) type and implement the `Debug` trait
//! which will include the source code that generated the error.
//!
//! ```ignore
//! Syntax error, statement is empty
//!  --> examples/files/document.md:3:3
//!   |
//! 3 | {{}}
//!   | --^
//! ```
//!
//! ## Templates
//!
//! Templates must always be named so that useful error messages can be
//! generated; if a name is not available the value of *unknown* will be
//! used as the template name.
//!
//! Use the registry to compile a template:
//!
//! ```ignore
//! let registry = Registry::new();
//! let template = registry.parse("file-name.md", "{{foo}}").unwrap();
//! ```
//!
//! If you are extracting a template from a larger document use
//! [ParserOptions](parser::ParserOptions) to set a line and byte offset:
//!
//! ```ignore
//! let registry = Registry::new();
//! let options = ParserOptions::new(String::from("file-name.md"), 12, 2048);
//! let template = registry.compile("{{foo}}", options).unwrap();
//! ```
//!
//! ## Lint
//!
//! Sometimes it is useful to check whether a template is well-formed. The
//! `lint` function will return a list of syntax errors:
//!
//! ```ignore
//! let registry = Registry::new();
//! let errors = registry.lint("file-name.md", "{{.bad.path}}").unwrap();
//! ```

//#![deny(missing_docs)]
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
pub mod trim;

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
