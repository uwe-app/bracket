#![deny(missing_docs)]

//! Bracket is a fast and correct implementation of the [Handlebars][]
//! general purpose template engine.
//!
//! It is designed to keep allocations to a minimum by using pointers
//! and string slices into the underlying template wherever possible.
//!
//! It detects cyclic partials and helper calls and returns an error
//! rather than overflow the stack so should be robust when used with
//! untrusted input.
//!
//! The lexer generates a stream of tokens which are consumed by a
//! parser that transforms them into AST nodes; these nodes can then
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
//! Syntax error, expecting identifier
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
//! let template = registry.parse("file-name.md", "{{foo}}")?;
//! ```
//!
//! If you are extracting a template from a larger document use
//! [ParserOptions](parser::ParserOptions) to set a line and byte offset:
//!
//! ```ignore
//! let registry = Registry::new();
//! let options = ParserOptions::new(String::from("module.rs"), 12, 2048);
//! let template = registry.compile("{{foo}}", options)?;
//! ```
//!
//! Use [insert()](Registry#method.insert) to compile and add a template
//! to the registry:
//!
//! ```ignore
//! registry.insert("dynamic", "{{title}}")?;
//! ```
//!
//! To load files from disc requires the `fs` feature which is enabled by default;
//! once the file contents are loaded they are compiled and added to the registry:
//!
//! ```ignore
//! let mut registry = Registry::new();
//! // Template name is derived from the file stem
//! registry.read_dir(PathBuf::from("partials/"), "hbs")?;
//! // Explicit template name
//! registry.add("info", PathBuf::from("documents/info.md"))?;
//! // Template name is the file path
//! registry.load(PathBuf::from("documents/page.md"))?;
//! ```
//!
//! ## Render
//!
//! If a template has already been registered it can be rendered by name:
//!
//! ```ignore
//! let result = registry.render("info", json!({"title": "Document Title"}))?;
//! println!("{}", result);
//! ```
//!
//! For dynamic templates use the [once()](Registry#method.once) function to render a string template directly:
//!
//! ```ignore
//! let result = registry.once(
//!     "dynamic-template.md",
//!     "# {{title}}",
//!     json!({"title": "Document Title"}))?;
//! println!("{}", result);
//! ```
//!
//! ## Lint
//!
//! Sometimes it is useful to check whether a template is well-formed. The
//! `lint` function will return a list of syntax errors:
//!
//! ```ignore
//! let errors = registry.lint("file-name.md", "{{.bad.path}}")?;
//! ```
//!
//! ## Escape
//!
//! By default templates are escaped for HTML output; you can call `set_escape()`
//! on a registry with an escape function to change this behaviour.
//!
//! For example to disable escaping:
//!
//! ```ignore
//! use bracket::escape;
//! registry.set_escape(escape::noop);
//! ```
//! ## Strict Mode
//!
//! By default the handlebars behaviour for variable interpolation is a noop
//! when a variable cannot be found; to always error when a variable or helper
//! is missing enable strict mode:
//!
//! ```ignore
//! registry.set_strict(true);
//! ```
//!
//! ## Helpers
//!
//! Helper functions make handlebars a versatile template engine; all
//! helpers are enabled by default but can be disabled via feature flags if
//! you need to.
//!
//! By default all the built-in helpers are enabled:
//!
//! * [log](helper::log::Log) Print log messages.
//! * [lookup](helper::lookup::Lookup) Lookup a field of an object or array.
//! * [#if](helper::if::If) Conditional block helper.
//! * [#unless](helper::unless::Unless) Negated conditional block helper.
//! * [#each](helper::each::Each) Iterate arrays and objects.
//! * [#with](helper::with::With) Set the block context scope.
//!
//! Some useful extra helpers are also enabled by default:
//!
//! * [json](helper::json::Json) Convert values to JSON strings.
//! * [and](helper::logical::And) Logical boolean AND operation.
//! * [or](helper::logical::Or) Logical boolean OR operation.
//! * [not](helper::logical::Not) Logical boolean NOT operation.
//!
//! Numerical comparison helpers:
//!
//! * [eq](helper::comparison::Equal) Test for equality.
//! * [ne](helper::comparison::NotEqual) Test for inequality.
//! * [lt](helper::comparison::LessThan) Test for less than.
//! * [gt](helper::comparison::GreaterThan) Test for greater than.
//! * [lte](helper::comparison::LessThanEqual) Test for less than or equal to.
//! * [gte](helper::comparison::GreaterThanEqual) Test for greater than or equal to.
//!
//! To add a helper to the registry use `helpers_mut()`:
//!
//! ```ignore
//! registry.helpers_mut().insert("custom", Box::new(CustomHelper {}));
//! ```
//! Then you can call it like any other helper:
//!
//! ```ignore
//! {{custom "Hello world!" param=true}}
//! ```
//!
//! See the [Helper Module](helper) to learn more about creating your own
//! helpers.
//!
//! ## Links
//!
//! The `links` feature which is enabled by default parses wiki-style links
//! into a [Link](parser::ast::Link) node. When this feature is enabled the renderer will look
//! for a link handler and it will be invoked with the link `href`, `label` and `title` as arguments.
//!
//! Note that a link helper is a standard helper implementation but is registered
//! using an event handler as it should not be invoked directly via a template:
//!
//! ```ignore
//! registry.handlers_mut().link = Some(Box::new(LinkHelper {}));
//! ```
//!
//! Such that a wiki-style link like this one `[[https://example.com|Example Website]]`
//! would call the `link` helper with the first argument as the website URL and
//! the second argument as the label (*Example Website*).
//!
//! After the label an optional title can be specified using another pipe delimiter:
//! `[[/path/to/page|Link Label|Alternative Title]]` this is passed as the third
//! argument to the link helper.
//!
//! If this feature is disabled or no handler is defined the link is
//! rendered to the output as text.
//!
//! Links do not accept new lines; to include a new line, vertical pipe or right square bracket
//! it must be preceeded by a backslash, eg: `\n`, `\|` or `\]`.
//!
//! ## Handlers
//!
//! Support for `helperMissing` and `blockHelperMissing` handlers can be enabled using the registry
//! handlers. The handlers should be regular helper implementations:
//!
//! ```ignore
//! registry.handlers_mut().helper_missing = Some(Box::new(HelperMissing {}));
//! registry.handlers_mut().block_helper_missing = Some(Box::new(BlockHelperMissing {}));
//! ```
//! When a block helper missing handler is invoked it also has access to the underlying
//! [property()](render::Context#method.property).
//!
//! The rules for when these handlers are invoked are described in
//! the [Handlebars Hooks][] documentation.
//!
//! [Handlebars]: https://handlebarsjs.com
//! [Handlebars Hooks]: https://handlebarsjs.com/guide/hooks.html
//!

#[macro_use]
extern crate rental;

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
