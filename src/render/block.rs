//! Type for block helpers.
use crate::parser::ast::Node;

use super::Render;
use crate::error::HelperError;

/// Encapsulates the templates passed to a block helper.
#[derive(Debug)]
pub struct BlockTemplate<'source> {
    template: &'source Node<'source>,
}

impl<'source> BlockTemplate<'source> {
    pub fn new(template: &'source Node<'source>) -> Self {
        Self { template }
    }

    /// Get the primary template node for the block.
    pub fn template(&self) -> &'source Node<'source> {
        self.template
    }

    /// Evaluate the block conditionals and find
    /// the first node that should be rendered.
    pub fn inverse<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
    ) -> Result<Option<&'source Node<'source>>, HelperError> {
        let mut alt: Option<&'source Node<'source>> = None;
        let mut branch: Option<&'source Node<'source>> = None;
        match &self.template {
            Node::Block(ref block) => {
                if !block.conditions().is_empty() {
                    for node in block.conditions().iter() {
                        match node {
                            Node::Condition(clause) => {
                                // Got an else clause, last one wins!
                                if clause.call().is_empty() {
                                    alt = Some(node);
                                } else {
                                    if let Some(value) = rc
                                        .call(clause.call())
                                        .map_err(Box::new)?
                                    {
                                        if rc.is_truthy(&value) {
                                            branch = Some(node);
                                            break;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(branch.or(alt))
    }
}
