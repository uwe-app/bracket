//! Helper trait and types for the default set of helpers.
//!
//! The [Helper Trait](self::Helper) should be implemented
//! for custom helpers which can then be added to a registry.
//!
//! Helper `call()` functions accept three arguments:
//!
//! * [rc](crate::render::Render) The active renderer.
//! * [ctx](crate::render::context::Context) Helper arguments and hash parameters.
//! * [template](crate::parser::ast::Node) Inner template when called as a block.
//!
//! The renderer can be used to render inner templates when a helper
//! is called as a block and provides functions for writing to the output destination.
//!
//! The context is used to access the arguments and hash parameters and may also
//! be used for type assertions using the
//! [try_get()](crate::render::context::Context#method.try_get) and
//! [try_hash()](crate::render::context::Context#method.try_hash) methods. The
//! [arity()](crate::render::context::Context#method.arity) method can be used to
//! assert on argument length.
//!
//! When a helper is called as a block the optional template node will be `Some`.
//! Raw helpers can access the inner text using [text()](crate::render::context::Context#method.text).
//!
//! To determine how a helper was invoked requires checking for an inner template
//! or raw text; if neither is available it is a statement:
//!
//! ```ignore
//! if let Some(node) = template {
//!     // Helper was invoked as a block `{{#helper}}...{{/helper}}`
//! } else if let Some(text) = ctx.text() {
//!     // Helper was invoked as a raw block `{{{{helper}}}}...{{{{/helper}}}}`
//! } else {
//!     // Helper was invoked as a statement `{{helper}}`
//! }
//! ```
//!
//! ## Return Values
//!
//! The signature for helper return values is [HelperValue](HelperValue) which requires
//! that the `call()` function returns an optional [Value](serde_json::Value).
//!
//! A return value is useful when a helper is invoked as a statement; when invoked as
//! a block return `Ok(None)`.
//!
//! If a statement helper is used for side-effects (such as the [Log](log::Log) helper) then
//! return `Ok(None)`.
//!
//! ## Local Helpers
//!
//! Local helpers are defined on [rc](crate::render::Render) using [register_local_helper()](crate::render::Render#method.register_local_helper) and live for the lifetime of the parent helper call.
//!
//! ## Clone
//!
//! Helpers must implement `Clone` so that we can support local helpers.

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

/// Result type that helper implementations should return.
pub type HelperValue = HelperResult<Option<Value>>;

/// Trait for helpers.
pub trait Helper: Send + Sync {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        template: Option<&'render Node<'render>>,
    ) -> HelperValue;
}

/// Trait for local helpers which must implement `Clone`.
pub trait LocalHelper: Helper + DynClone {}

dyn_clone::clone_trait_object!(LocalHelper);

#[cfg(feature = "comparison-helper")]
pub mod comparison;
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

/// Collection of helpers.
#[derive(Default)]
pub struct HelperRegistry<'reg> {
    helpers: HashMap<&'reg str, Box<dyn Helper + 'reg>>,
}

impl<'reg> HelperRegistry<'reg> {
    /// Create a collection of helpers.
    ///
    /// Helpers configured using the compiler feature flags are
    /// automatically added to this collection.
    ///
    /// If you need a helper collection without the builtin helpers
    /// use `Default::default()`.
    pub fn new() -> Self {
        let mut reg = Self {
            helpers: Default::default(),
        };
        reg.builtins();
        reg
    }

    fn builtins(&mut self) {
        #[cfg(feature = "conditional-helper")]
        self.insert("if", Box::new(r#if::If {}));
        #[cfg(feature = "conditional-helper")]
        self.insert("unless", Box::new(unless::Unless {}));

        #[cfg(feature = "comparison-helper")]
        self.insert("eq", Box::new(comparison::Equal {}));
        #[cfg(feature = "comparison-helper")]
        self.insert("ne", Box::new(comparison::NotEqual {}));
        #[cfg(feature = "comparison-helper")]
        self.insert("gt", Box::new(comparison::GreaterThan {}));
        #[cfg(feature = "comparison-helper")]
        self.insert("gte", Box::new(comparison::GreaterThanEqual {}));
        #[cfg(feature = "comparison-helper")]
        self.insert("lt", Box::new(comparison::LessThan {}));
        #[cfg(feature = "comparison-helper")]
        self.insert("lte", Box::new(comparison::LessThanEqual {}));

        #[cfg(feature = "log-helper")]
        self.insert("log", Box::new(log::Log {}));
        #[cfg(feature = "lookup-helper")]
        self.insert("lookup", Box::new(lookup::Lookup {}));

        #[cfg(feature = "logical-helper")]
        self.insert("and", Box::new(logical::And {}));
        #[cfg(feature = "logical-helper")]
        self.insert("or", Box::new(logical::Or {}));
        #[cfg(feature = "logical-helper")]
        self.insert("not", Box::new(logical::Not {}));

        #[cfg(feature = "with-helper")]
        self.insert("with", Box::new(with::With {}));
        #[cfg(feature = "each-helper")]
        self.insert("each", Box::new(each::Each {}));

        #[cfg(feature = "json-helper")]
        self.insert("json", Box::new(json::Json {}));
    }

    /// Insert a helper into this collection.
    pub fn insert(&mut self, name: &'reg str, helper: Box<dyn Helper + 'reg>) {
        self.helpers.insert(name, helper);
    }

    /// Remove a helper from this collection.
    pub fn remove(&mut self, name: &'reg str) {
        self.helpers.remove(name);
    }

    /// Get a helper from this collection.
    pub fn get(&self, name: &str) -> Option<&Box<dyn Helper + 'reg>> {
        self.helpers.get(name)
    }
}
