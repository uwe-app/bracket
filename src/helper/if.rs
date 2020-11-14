//! Helpers for conditional statements.
use crate::{
    helper::{Helper, HelperValue},
    parser::ast::Node,
    render::{Context, Render},
};

use serde_json::Value;

/// Evaluates an argument as *truthy*.
///
/// For block execution if the value is *truthy* the inner template
/// is rendered otherwise each conditional is evaluated and
/// the first one which returns a *truthy* value is rendered.
///
/// When executed in a statement this helper returns whether it's
/// argument is *truthy*.
///
#[derive(Clone)]
pub struct If;

impl Helper for If {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        ctx.arity(1..1)?;

        if let Some(template) = template {
            if ctx.is_truthy(ctx.get(0).unwrap()) {
                rc.template(template)?;
            } else if let Some(node) = rc.inverse(template)? {
                rc.template(node)?;
            }
            Ok(None)
        } else {
            Ok(Some(Value::Bool(ctx.is_truthy(ctx.get(0).unwrap()))))
        }
    }
}
