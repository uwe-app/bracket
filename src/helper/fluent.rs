//! Helper for fluent language lookup.
use crate::{
    helper::{Helper, HelperValue},
    parser::ast::Node,
    render::{Context, Render},
};

use serde_json::Value;

use fluent_templates::Loader;

/// Lookup a language string in the underlying loader.
pub struct Fluent {
    loader: Box<dyn Loader + Send + Sync>,
}

impl Helper for Fluent {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        ctx.arity(1..usize::MAX)?;

        Ok(None)
    }
}
