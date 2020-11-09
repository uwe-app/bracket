//! Helpers for conditional statements.
use crate::{
    helper::{Helper, HelperResult, ValueResult},
    render::{Context, Render},
    parser::ast::Node,
};

use serde_json::Value;

#[derive(Clone)]
pub struct IfHelper;

impl Helper for IfHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        template: Option<&'render Node<'render>>,
    ) -> ValueResult {
        if let Some(template) = template {
            ctx.arity(1..1)?;

            if rc.is_truthy(ctx.arguments().get(0).unwrap()) {
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
                if !rc.is_truthy(&val) {
                    result = Value::Bool(false);
                    break;
                }
            }
            Ok(Some(result))
        }
    }
}
