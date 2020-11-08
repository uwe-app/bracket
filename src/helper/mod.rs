//! Helper trait and types for the default set of helpers.
use dyn_clone::DynClone;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::ops::Range;

use crate::{
    error::HelperError as Error,
    parser::ast::Node,
    render::{BlockTemplate, Context, Render},
};

/// The result type that helpers should return.
pub type ValueResult = std::result::Result<Option<Value>, Error>;

/// The result type that block helpers should return.
pub type BlockResult = std::result::Result<(), Error>;

/// Trait for helpers.
pub trait Helper: Send + Sync + DynClone {
    fn call<'reg, 'source, 'render, 'call>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &mut Context<'reg, 'source, 'render, 'call>,
    ) -> ValueResult;
}

dyn_clone::clone_trait_object!(Helper);

/// Trait for block helpers.
pub trait BlockHelper: Send + Sync + DynClone {
    fn call<'reg, 'source, 'render, 'call>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &mut Context<'reg, 'source, 'render, 'call>,
        block: BlockTemplate<'source>,
    ) -> BlockResult;
}

dyn_clone::clone_trait_object!(BlockHelper);

/// Trait for types that provide helper assertions.
pub trait Assertion {
    /// Assert that the context arguments are in the given arity range.
    fn arity(&self, context: &Context<'_, '_, '_, '_>, range: Range<usize>) -> BlockResult;
}

#[cfg(feature = "each-helper")]
pub mod each;
#[cfg(feature = "conditional-helper")]
pub mod r#if;
#[cfg(feature = "json-helper")]
pub mod json;
#[cfg(feature = "log-helper")]
pub mod log;
#[cfg(feature = "logical-helper")]
pub mod logical;
#[cfg(feature = "lookup-helper")]
pub mod lookup;
#[cfg(feature = "conditional-helper")]
pub mod unless;
#[cfg(feature = "with-helper")]
pub mod with;

/// Registry of helpers.
#[derive(Clone, Default)]
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
        #[cfg(feature = "conditional-helper")]
        self.register_helper("if", Box::new(r#if::IfHelper {}));
        #[cfg(feature = "conditional-helper")]
        self.register_block_helper("if", Box::new(r#if::IfBlockHelper {}));
        #[cfg(feature = "conditional-helper")]
        self.register_block_helper("unless", Box::new(unless::UnlessHelper {}));

        #[cfg(feature = "log-helper")]
        self.register_helper("log", Box::new(log::LogHelper {}));
        #[cfg(feature = "lookup-helper")]
        self.register_helper("lookup", Box::new(lookup::LookupHelper {}));

        #[cfg(feature = "logical-helper")]
        self.register_helper("and", Box::new(logical::AndHelper {}));
        #[cfg(feature = "logical-helper")]
        self.register_helper("or", Box::new(logical::OrHelper {}));
        #[cfg(feature = "logical-helper")]
        self.register_helper("not", Box::new(logical::NotHelper {}));

        #[cfg(feature = "with-helper")]
        self.register_block_helper("with", Box::new(with::WithHelper {}));
        #[cfg(feature = "each-helper")]
        self.register_block_helper("each", Box::new(each::EachHelper {}));

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
