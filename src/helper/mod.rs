//! Helper trait and types for the default set of helpers.
use dyn_clone::DynClone;
use serde_json::Value;
use std::collections::HashMap;

use crate::{
    error::HelperError,
    parser::ast::Node,
    render::{Context, Render},
};

/// Result type returned when invoking helpers.
pub type HelperResult<T> = std::result::Result<T, HelperError>;

/// The result type that helper implementations should return.
pub type HelperValue = HelperResult<Option<Value>>;

/// Trait for helpers.
pub trait Helper: Send + Sync + DynClone {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        template: Option<&'render Node<'render>>,
    ) -> HelperValue;
}

dyn_clone::clone_trait_object!(Helper);

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
#[cfg(feature = "comparison-helper")]
pub mod comparison;
#[cfg(feature = "conditional-helper")]
pub mod unless;
#[cfg(feature = "with-helper")]
pub mod with;

/// Registry of helpers.
#[derive(Clone, Default)]
pub struct HelperRegistry<'reg> {
    helpers: HashMap<&'reg str, Box<dyn Helper + 'reg>>,
}

impl<'reg> HelperRegistry<'reg> {
    pub fn new() -> Self {
        let mut reg = Self {
            helpers: Default::default(),
        };
        reg.builtins();
        reg
    }

    fn builtins(&mut self) {
        #[cfg(feature = "conditional-helper")]
        self.register_helper("if", Box::new(r#if::IfHelper {}));
        #[cfg(feature = "conditional-helper")]
        self.register_helper("unless", Box::new(unless::UnlessHelper {}));

        #[cfg(feature = "comparison-helper")]
        self.register_helper("eq", Box::new(comparison::Equal{}));
        #[cfg(feature = "comparison-helper")]
        self.register_helper("ne", Box::new(comparison::NotEqual{}));

        #[cfg(feature = "comparison-helper")]
        self.register_helper("gt", Box::new(comparison::GreaterThan {}));
        #[cfg(feature = "comparison-helper")]
        self.register_helper("gte", Box::new(comparison::GreaterThanEqual {}));
        #[cfg(feature = "comparison-helper")]
        self.register_helper("lt", Box::new(comparison::LessThan {}));
        #[cfg(feature = "comparison-helper")]
        self.register_helper("lte", Box::new(comparison::LessThanEqual {}));

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
        self.register_helper("with", Box::new(with::WithHelper {}));
        #[cfg(feature = "each-helper")]
        self.register_helper("each", Box::new(each::EachHelper {}));

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

    pub fn unregister_helper(&mut self, name: &'reg str) {
        self.helpers.remove(name);
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn Helper + 'reg>> {
        self.helpers.get(name)
    }
}
