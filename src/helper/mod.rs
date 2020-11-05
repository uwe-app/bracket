//! Helper trait and types for the default set of helpers.
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::ops::Range;

use crate::{error::HelperError as Error, parser::ast::Node, render::Render};

/// The result type that helpers should return.
pub type ValueResult = std::result::Result<Option<Value>, Error>;

/// The result type that block helpers should return.
pub type BlockResult = std::result::Result<(), Error>;

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
    ) -> BlockResult;
}

pub mod each;
pub mod r#if;
#[cfg(feature = "json-helper")]
pub mod json;
#[cfg(feature = "log-helper")]
pub mod log;
pub mod lookup;
pub mod with;

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
    ) -> Result<Option<&'source Node<'source>>, Error> {
        let mut alt: Option<&'source Node<'source>> = None;
        let mut branch: Option<&'source Node<'source>> = None;
        match &self.template {
            Node::Block(ref block) => {
                if !block.conditions().is_empty() {
                    for node in block.conditions().iter() {
                        match node {
                            Node::Condition(clause) => {
                                // Got an else clause, last oone wins!
                                if clause.call().is_empty() {
                                    alt = Some(node);
                                } else {
                                    todo!("Evaluate and return 'else if' clauses!");
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
}

impl Into<Vec<Value>> for Context<'_> {
    fn into(self) -> Vec<Value> {
        self.arguments
    }
}

impl Into<String> for Context<'_> {
    fn into(self) -> String {
        self.name.to_string()
    }
}

impl Into<(String, Vec<Value>)> for Context<'_> {
    fn into(self) -> (String, Vec<Value>) {
        (self.name.to_string(), self.arguments)
    }
}

impl Into<(String, Vec<Value>, Map<String, Value>)> for Context<'_> {
    fn into(self) -> (String, Vec<Value>, Map<String, Value>) {
        (self.name.to_string(), self.arguments, self.hash)
    }
}

/// Trait for types that provide helper assertions.
pub trait Assertion {
    /// Assert that the context arguments are in the given arity range.
    fn arity(&self, context: &Context<'_>, range: Range<usize>) -> BlockResult;
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
