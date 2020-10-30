use serde::Serialize;
use serde_json::Value;

use crate::{
    error::RenderError,
    output::Output,
    parser::ast::{Node},
    registry::Registry,
};

pub struct Render<'reg, 'render> {
    registry: &'reg Registry<'reg>,
    root: Value,
    writer: Box<&'render mut dyn Output>,
}

impl<'reg, 'render> Render<'reg, 'render> {
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

    fn write_str(&mut self, s: &str) -> Result<usize, RenderError> {
        Ok(self.writer.write_str(s).map_err(RenderError::from)?)
    }

    pub fn render(&mut self, node: &'render Node<'render>) -> Result<(), RenderError> {

        //println!("rendering node {:?}", node);

        match node {
            Node::Text(ref n) => {
                self.write_str(n.as_str())?;
            }
            Node::RawBlock(ref n) => {
                self.write_str(n.between())?;
            }
            Node::RawStatement(ref n) => {
                let raw = &n.as_str()[1..];
                self.write_str(raw)?;
            }
            Node::RawComment(_) => {}
            Node::Comment(_) => {}
            Node::Document(ref doc) => {
                for node in doc.nodes().iter() {
                    self.render(node)?;
                }
            }
            Node::Statement(ref n) => {
                println!("TODO: Evaluate statement in render!");
                self.write_str(n.as_str())?;
            }
            Node::Block(ref block) => {
                // TODO: call partial / helper for blocks
                for node in block.nodes().iter() {
                    self.render(node)?;
                }
            }
            _ => todo!("Render other node types")
        }

        Ok(())
    }
}
