//! Helper trait and types for the default set of helpers.
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::ops::Range;

use crate::{error::HelperError as Error, parser::ast::Node, render::Render};

/// The result type that helpers should return.
pub type ValueResult = std::result::Result<Option<Value>, Error>;

/// The result type that block helpers should return.
pub type Result = std::result::Result<(), Error>;

/// Trait for helpers.
pub trait Helper: Send + Sync {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
    ) -> ValueResult;
}

/// Trait for block helpers.
pub trait BlockHelper: Send + Sync {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
        block: BlockTemplate<'source>,
    ) -> Result;
}

mod each;
mod r#if;
#[cfg(feature = "json-helper")]
mod json;
#[cfg(feature = "log-helper")]
mod log;
mod lookup;
mod with;

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
    pub fn inverse(&self) -> Option<&'source Node<'source>> {
        match &self.template {
            Node::Block(ref block) => {
                if !block.conditions().is_empty() {
                    for node in block.conditions().iter() {
                        println!("Got block condition {:?}", node);
                        match node {
                            Node::Condition(clause) => {
                                // Got an else clause
                                if clause.call().is_empty() {
                                    return Some(node)
                                } else {
                                    todo!("Evaluate and return 'else if' clauses!");
                                }
                            }
                            _ => {}
                        }
                    }
                }
                None
            }
            _ => None
        }
    }
}

/// Context for the call to a helper.
pub struct Context<'source> {
    name: &'source str,
    arguments: Vec<Value>,
    hash: Map<String, Value>,
}

impl<'source> Context<'source> {
    pub fn new(
        name: &'source str,
        arguments: Vec<Value>,
        hash: Map<String, Value>,
    ) -> Self {
        Self {
            name,
            arguments,
            hash,
        }
    }

    pub fn name(&self) -> &'source str {
        self.name
    }

    pub fn arguments(&self) -> &Vec<Value> {
        &self.arguments
    }

    pub fn hash(&self) -> &Map<String, Value> {
        &self.hash
    }

    pub fn into(self) -> (String, Vec<Value>, Map<String, Value>) {
        (self.name.to_string(), self.arguments, self.hash)
    }

    // TODO: move out of Context
    pub fn assert_arity(&self, range: Range<usize>) -> Result {
        if range.start == range.end {
            if self.arguments.len() != range.start {
                return Err(Error::ArityExact(
                    self.name().to_owned(),
                    range.start,
                ));
            }
        } else {
            if self.arguments.len() < range.start
                || self.arguments.len() > range.end
            {
                return Err(Error::ArityRange(
                    self.name().to_owned(),
                    range.start,
                    range.end,
                ));
            }
        }
        Ok(())
    }
}

/// Registry of helpers.
pub struct HelperRegistry<'reg> {
    helpers: HashMap<&'reg str, Box<dyn Helper + 'reg>>,
    block_helpers: HashMap<&'reg str, Box<dyn BlockHelper + 'reg>>,
}

impl<'reg> HelperRegistry<'reg> {
    pub fn new() -> Self {
        let mut reg = Self {
            helpers: Default::default(),
            block_helpers: Default::default(),
        };
        reg.builtins();
        reg
    }

    fn builtins(&mut self) {
        #[cfg(feature = "log-helper")]
        self.register_helper("log", Box::new(log::LogHelper {}));

        self.register_helper("lookup", Box::new(lookup::LookupHelper {}));

        self.register_block_helper("with", Box::new(with::WithHelper {}));
        self.register_block_helper("each", Box::new(each::EachHelper {}));
        self.register_block_helper("if", Box::new(r#if::IfHelper {}));
        //self.register_block_helper("unless", Box::new(UnlessHelper {}));

        #[cfg(feature = "json-helper")]
        self.register_helper("json", Box::new(json::JsonHelper {}));
    }

    pub fn register_helper(
        &mut self,
        name: &'reg str,
        helper: Box<dyn Helper + 'reg>,
    ) {
        self.helpers.insert(name, helper);
    }

    pub fn register_block_helper(
        &mut self,
        name: &'reg str,
        helper: Box<dyn BlockHelper + 'reg>,
    ) {
        self.block_helpers.insert(name, helper);
    }

    //pub fn helpers(&self) -> &HashMap<&'reg str, Box<dyn Helper + 'reg>> {
    //&self.helpers
    //}

    pub fn get(&self, name: &str) -> Option<&Box<dyn Helper + 'reg>> {
        self.helpers.get(name)
    }

    pub fn get_block(
        &self,
        name: &str,
    ) -> Option<&Box<dyn BlockHelper + 'reg>> {
        self.block_helpers.get(name)
    }
}
