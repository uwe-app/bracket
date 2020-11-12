//! Helpers for conditional statements.
use crate::{
    helper::{Helper, HelperValue},
    parser::ast::Node,
    render::{Context, Render},
};

use serde_json::Value;

/// Evaluates arguments as *truthy*.
///
/// For block execution if the value is *truthy* the inner template
/// is rendered otherwise each conditional is evaluated and
/// the first one which returns a *truthy* value is rendered.
///
/// When executed in a statement this helper will accept any number
/// of arguments and return `true` if all the arguments are *truthy*.
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
        if let Some(template) = template {
            ctx.arity(1..1)?;

            if ctx.is_truthy(ctx.get(0).unwrap()) {
                rc.template(template)?;
            } else if let Some(node) = rc.inverse(template)? {
                rc.template(node)?;
            }

            Ok(None)
        } else {
            ctx.arity(1..usize::MAX)?;

            let args = ctx.arguments();
            let mut result = Value::Bool(true);
            for val in args {
                if !ctx.is_truthy(&val) {
                    result = Value::Bool(false);
                    break;
                }
            }
            Ok(Some(result))
        }
    }
}
