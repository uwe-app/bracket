//! Templates add rendering capability to nodes.
use std::collections::HashMap;

use serde::Serialize;
use std::fmt;

use crate::{
    escape::EscapeFn,
    helper::HelperRegistry,
    output::Output,
    parser::{ast::Node, Parser, ParserOptions},
    render::Render,
    RenderResult, SyntaxResult,
};

/// Collection of named templates.
pub type Templates<'a> = HashMap<String, Template<'a>>;

/// Type that adds rendering capability to a document node.
#[derive(Debug)]
pub struct Template<'source> {
    node: Node<'source>,
}

impl<'source> Template<'source> {
    /// Create a new template.
    pub(crate) fn new(node: Node<'source>) -> Self {
        Self { node }
    }

    /// The document node for the template.
    pub fn node(&self) -> &'source Node {
        &self.node
    }

    /// Compile a block.
    pub fn compile(
        source: &'source str,
        options: ParserOptions,
    ) -> SyntaxResult<Template<'source>> {
        let mut parser = Parser::new(source, options);
        let node = parser.parse()?;
        Ok(Template::new(node))
    }

    /// Render this template to the given writer.
    pub(crate) fn render<'a, T>(
        &self,
        strict: bool,
        escape: &EscapeFn,
        helpers: &'a HelperRegistry<'a>,
        templates: &'a Templates<'a>,
        name: &str,
        data: &T,
        writer: &'a mut impl Output,
    ) -> RenderResult<()>
    where
        T: Serialize,
    {
        let mut rc = Render::new(
            strict,
            escape,
            helpers,
            templates,
            name,
            data,
            Box::new(writer),
        )?;

        rc.render(&self.node)
    }
}

impl fmt::Display for Template<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.node.fmt(f)
    }
}

