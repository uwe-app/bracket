use std::fmt;

use crate::{
    error::{RenderError, SyntaxError},
    lexer::{ast::Block, parser::Parser},
    render::{Render, RenderContext, Renderer},
};

// TODO: support rendering to original source form
//pub trait SourceDisplay {
    //fn write_source(&self, s: &str, w: &mut Write) -> crate::Result<usize>;
//}

#[derive(Debug)]
pub struct Template<'source> {
    source: &'source str,
    block: Block<'source>,
}

impl<'source> Template<'source> {
    pub fn block(&self) -> &'source Block {
        &self.block
    }
}

impl fmt::Display for Template<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.block.fmt(f)
    }
}

impl<'reg, 'render> Renderer<'reg, 'render> for Template<'_> {
    fn render(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result<(), RenderError> {
        let renderer = Render::new(self.block());
        renderer.render(rc)
    }
}

impl<'source> Template<'source> {
    /// Compile a block.
    pub fn compile(source: &'source str) -> Result<Template, SyntaxError> {
        let mut parser = Parser::new();
        let block = parser.parse(source)?;
        Ok(Template { source, block })
    }
}
