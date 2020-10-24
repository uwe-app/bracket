use serde::Serialize;
use serde_json::Value;

use crate::{
    error::RenderError,
    lexer::ast::{Block, BlockType},
    output::Output,
    registry::Registry,
};

pub trait Renderer<'reg, 'render> {
    fn render(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result<(), RenderError>;
}

pub struct RenderState {}

impl RenderState {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct RenderContext<'reg, 'render> {
    registry: &'reg Registry<'reg>,
    root: Value,
    state: RenderState,
    writer: Box<&'render mut dyn Output>,
}

impl<'reg, 'render> RenderContext<'reg, 'render> {
    pub fn new<T: Serialize>(
        registry: &'reg Registry<'reg>,
        data: &T,
        state: RenderState,
        writer: Box<&'render mut dyn Output>,
    ) -> Result<Self, RenderError> {
        let root = serde_json::to_value(data).map_err(RenderError::from)?;
        Ok(Self {
            registry,
            root,
            state,
            writer,
        })
    }

    pub fn write_str(&mut self, s: &str) -> Result<usize, RenderError> {
        Ok(self.writer.write_str(s).map_err(RenderError::from)?)
    }
}

pub struct Render<'source> {
    block: &'source Block<'source>,
}

impl<'source> Render<'source> {
    pub fn new(block: &'source Block<'source>) -> Self {
        Self { block }
    }

    /*
    fn render_expr<'reg, 'render>(
        &self,
        expr: &Expr<'source>,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result<(), RenderError> {
        if expr.is_raw() {
            rc.write_str(expr.value())?;
        } else {
            todo!(
                "Evaluate the expression and escape the content if necessary"
            );
        }
        Ok(())
    }
    */

    /*
    fn render_token<'reg, 'render>(
        &self,
        token: &Token<'source>,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result<(), RenderError> {
        match token {
            Token::Text(ref t) => {
                rc.write_str(t.as_str())?;
            }
            Token::RawBlock(ref t) => {
                println!("RENDER A RAW BLOCK");
            }
            Token::RawComment(ref t) => {}
            Token::Expression(ref e) => self.render_expr(e, rc)?,
            Token::Block(ref b) => {
                self.render_block(b, rc)?;
            }
        }
        Ok(())
    }
    */

    fn render_block<'reg, 'render>(
        &self,
        block: &Block<'source>,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result<(), RenderError> {

        //println!("rendering a block {:?}", block.block_type());
        match block.block_type() {
            BlockType::Text => {
                rc.write_str(block.open())?;
            }
            BlockType::RawBlock => {
                rc.write_str(block.between())?;
            }
            BlockType::RawComment | BlockType::Comment => {
                // NOTE: must ignore raw comments when rendering
            }
            BlockType::RawStatement => {
                let raw = &block.as_str()[1..];
                rc.write_str(raw)?;
            }
            _ => {
                for b in block.blocks().iter() {
                    println!("Rendering block {:?}", b.as_str());
                    self.render_block(b, rc)?;
                }
            }
        }

        Ok(())
    }
}

impl<'reg, 'render> Renderer<'reg, 'render> for Render<'_> {
    fn render(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result<(), RenderError> {
        self.render_block(self.block, rc)
    }
}
