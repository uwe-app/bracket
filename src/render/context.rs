//! Context information for the call to a helper.
use serde_json::{Map, Value};
use std::ops::Range;

use crate::{
    error::HelperError,
    helper::HelperResult,
    parser::ast::{Call, Node},
    render::Render,
};

/// Context for the call to a helper exposes immutable access to
/// the arguments and hash parameters for the helper.
///
/// It also provides some useful functions for asserting on argument
/// arity and the type of arguments and hash parameters.
pub struct Context<'call> {
    call: &'call Call<'call>,
    name: String,
    arguments: Vec<Value>,
    hash: Map<String, Value>,
    template: Option<&'call Node<'call>>,
}

impl<'call> Context<'call> {
    pub fn new(
        call: &'call Call<'call>,
        name: String,
        arguments: Vec<Value>,
        hash: Map<String, Value>,
        template: Option<&'call Node<'call>>,
    ) -> Self {
        Self {
            call,
            name,
            arguments,
            hash,
            template,
        }
    }

    pub fn template(&self) -> &Option<&'call Node<'_>> {
        &self.template
    }

    /// Evaluate the block conditionals and find
    /// the first node that should be rendered.
    pub fn inverse<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
    ) -> Result<Option<&Node<'_>>, HelperError> {
        let mut alt: Option<&Node<'_>> = None;
        let mut branch: Option<&Node<'_>> = None;

        if let Some(template) = self.template {
            match template {
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
        }

        Ok(branch.or(alt))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> &Vec<Value> {
        &self.arguments
    }

    pub fn hash(&self) -> &Map<String, Value> {
        &self.hash
    }

    pub fn arity(&self, range: Range<usize>) -> HelperResult<()> {
        if range.start == range.end {
            if self.arguments().len() != range.start {
                return Err(HelperError::ArityExact(
                    self.name.clone(),
                    range.start,
                ));
            }
        } else {
            if self.arguments().len() < range.start
                || self.arguments().len() > range.end
            {
                return Err(HelperError::ArityRange(
                    self.name.clone(),
                    range.start,
                    range.end,
                ));
            }
        }
        Ok(())
    }
}
