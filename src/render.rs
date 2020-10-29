use serde::Serialize;
use serde_json::Value;

use crate::{
    error::RenderError,
    output::Output,
    parser::ast::{BlockType, Node},
    registry::Registry,
};

pub trait Renderer<'reg, 'render> {
    fn render(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result<(), RenderError>;
}

pub struct RenderContext<'reg, 'render> {
    registry: &'reg Registry<'reg>,
    root: Value,
    writer: Box<&'render mut dyn Output>,
}

impl<'reg, 'render> RenderContext<'reg, 'render> {
    pub fn new<T: Serialize>(
        registry: &'reg Registry<'reg>,
        data: &T,
        writer: Box<&'render mut dyn Output>,
    ) -> Result<Self, RenderError> {
        let root = serde_json::to_value(data).map_err(RenderError::from)?;
        Ok(Self {
            registry,
            root,
            writer,
        })
    }

    pub fn write_str(&mut self, s: &str) -> Result<usize, RenderError> {
        Ok(self.writer.write_str(s).map_err(RenderError::from)?)
    }
}

pub struct Render<'source> {
    source: &'source str,
    node: &'source Node<'source>,
}

impl<'source> Render<'source> {
    pub fn new(source: &'source str, node: &'source Node<'source>) -> Self {
        Self { source, node }
    }

    fn render_node<'reg, 'render>(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
        node: &Node<'source>,
    ) -> Result<(), RenderError> {

        //println!("rendering node {:?}", node);

        match node {
            Node::Text(ref n) => {
                rc.write_str(n.as_str())?;
            }
            Node::Statement(ref n) => {
                println!("TODO: Evaluate statement in render!");
                rc.write_str(n.as_str())?;
            }
            Node::RawBlock(ref n) => {
                rc.write_str(n.between())?;
            }
            Node::RawStatement(ref n) => {
                let raw = &n.as_str()[1..];
                rc.write_str(raw)?;
            }
            Node::RawComment(_) => {}
            Node::Comment(_) => {}
            Node::Block(ref block) => {
                //println!("rendering a block {:?}", block.kind());
                match block.kind() {
                    _ => {
                        for b in block.nodes().iter() {
                            self.render_node(rc, b)?;
                        }
                    }
                }
            }
            _ => todo!("Render other node types")
        }

        Ok(())
    }
}

impl<'reg, 'render> Renderer<'reg, 'render> for Render<'_> {
    fn render(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result<(), RenderError> {
        self.render_node(rc, self.node)
    }
}
