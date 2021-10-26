//! Templates add rendering capability to nodes.
use std::collections::HashMap;

use serde::Serialize;
use std::fmt;

use crate::{
    output::Output,
    parser::{ast::Node, Parser, ParserOptions},
    render::{CallSite, Render},
    Registry, RenderResult, SyntaxResult,
};

use self_cell::self_cell;

/// Collection of named templates.
pub type Templates = HashMap<String, Template>;

self_cell!(
    struct Ast {
        owner: String,

        #[covariant]
        dependent: Node,
    }

    impl {Debug}
);

// SEE: https://github.com/projectfluent/fluent-rs/blob/master/fluent-bundle/src/resource.rs#L5-L14

/// Template that owns the underlying string and a corresponding document node.
#[derive(Debug)]
pub struct Template {
    file_name: Option<String>,
    ast: Ast,
}

impl Template {
    /// Compile a new template.
    pub fn compile(
        source: String,
        options: ParserOptions,
    ) -> SyntaxResult<Self> {
        let mut err = None;

        let file_name = if options.file_name != crate::parser::UNKNOWN {
            Some(options.file_name.clone())
        } else {
            None
        };

        let ast = Ast::new(source, |s: &String| {
            match Parser::new(s, options).parse() {
                Ok(ast) => ast,
                Err(e) => {
                    err = Some(e);
                    Default::default()
                }
            }
        });

        if let Some(e) = err {
            Err(e)
        } else {
            Ok(Self { file_name, ast })
        }
    }

    /// The document node for the template.
    pub fn node(&self) -> &Node<'_> {
        self.ast.borrow_dependent()
    }

    /// Get the file name given when this template was compiled.
    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_ref().map(|s| s.as_str())
    }

    /// Render this template to the given writer.
    pub fn render<'a, T>(
        &self,
        registry: &'a Registry<'a>,
        name: &str,
        data: &T,
        writer: &'a mut impl Output,
        stack: Vec<CallSite>,
    ) -> RenderResult<()>
    where
        T: Serialize,
    {
        let mut rc =
            Render::new(registry, name, data, Box::new(writer), stack)?;
        rc.render(self.node())
    }
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.node().fmt(f)
    }
}
