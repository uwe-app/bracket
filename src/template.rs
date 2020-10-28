use serde::Serialize;
use std::fmt;

use crate::{
    error::{RenderError, SyntaxError},
    parser::{Parser, ParserOptions, ast::Node},
    output::Output,
    render::{Render, RenderContext, Renderer},
    Registry,
};

#[derive(Debug)]
pub struct Template<'source> {
    source: &'source str,
    node: Node<'source>,
}

impl<'source> Template<'source> {
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
        let mut parser = Parser::new(options);
        let node = parser.parse(source)?;
        Ok(Template { source, node })
    }

    pub fn render<'reg, T>(
        &self,
        registry: &Registry<'reg>,
        name: &'reg str,
        data: &T,
        writer: &mut impl Output,
    ) -> Result<(), RenderError>
    where
        T: Serialize,
    {
        let mut rc = RenderContext::new(registry, data, Box::new(writer))?;
        let renderer = Render::new(self.source, self.node());
        renderer.render(&mut rc)
    }
}
