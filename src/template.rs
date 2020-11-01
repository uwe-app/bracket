//! Compiled template that may be stored in the registry.
use serde::Serialize;
use std::fmt;

use crate::{
    error::SyntaxError,
    output::Output,
    parser::{ast::Node, Parser, ParserOptions},
    render::Render,
    RenderResult,
    Registry,
};

#[derive(Debug)]
pub struct Template<'source> {
    source: &'source str,
    node: Node<'source>,
}

impl<'source> Template<'source> {
    pub fn new(source: &'source str, node: Node<'source>) -> Self {
        Self { source, node }
    }

    pub fn as_str(&self) -> &'source str {
        self.source
    }

    pub fn node(&self) -> &'source Node {
        &self.node
    }
}

impl fmt::Display for Template<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.node.fmt(f)
    }
}

impl<'source> Template<'source> {

    /// Compile a block.
    pub fn compile(
        source: &'source str,
        options: ParserOptions,
    ) -> Result<Template, SyntaxError<'source>> {
        let mut parser = Parser::new(source, options);
        let node = parser.parse()?;
        Ok(Template { source, node })
    }

    /// Render this template to the given writer.
    pub fn render<'reg, T>(
        &self,
        registry: &'reg Registry<'reg>,
        name: &str,
        data: &T,
        writer: &mut impl Output,
    ) -> RenderResult<'_, ()>
    where
        T: Serialize,
    {
        let mut rc =
            Render::new(self.source, registry, data, Box::new(writer))?;
        rc.render_node(&self.node)
    }
}
