use std::fmt;

use crate::{
    error::{RenderError, SyntaxError},
    lexer::{ast::Node, parser::{Parser, ParserOptions}},
    render::{Render, RenderContext, Renderer},
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

impl<'reg, 'render> Renderer<'reg, 'render> for Template<'_> {
    fn render(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result<(), RenderError> {
        let renderer = Render::new(self.node());
        renderer.render(rc)
    }
}

impl<'source> Template<'source> {
    /// Compile a block.
    pub fn compile(source: &'source str, options: ParserOptions) -> Result<Template, SyntaxError<'source>> {
        let mut parser = Parser::new(options);
        let node = parser.parse(source)?;
        Ok(Template { source, node })
    }
}
