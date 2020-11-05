//! Helpers for conditional statements.
use crate::{
    helper::{
        Assertion, Context, Helper, ValueResult,
    },
    render::Render,
};

use serde_json::Value;

/// Perform a logical AND on two arguments.
pub struct AndHelper;

impl Helper for AndHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
    ) -> ValueResult {
        rc.arity(&ctx, 2..2)?;

        let args = ctx.arguments();
        Ok(Some(Value::Bool(
            rc.is_truthy(args.get(0).unwrap())
                && rc.is_truthy(args.get(1).unwrap()),
        )))
    }
}

/// Perform a logical OR on two arguments.
pub struct OrHelper;

impl Helper for OrHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
    ) -> ValueResult {
        rc.arity(&ctx, 2..2)?;

        let args = ctx.arguments();
        Ok(Some(Value::Bool(
            rc.is_truthy(args.get(0).unwrap())
                || rc.is_truthy(args.get(1).unwrap()),
        )))
    }
}

/// Perform a logical NOT on an argument.
pub struct NotHelper;

impl Helper for NotHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
    ) -> ValueResult {
        rc.arity(&ctx, 1..1)?;

        let args = ctx.arguments();
        Ok(Some(Value::Bool(!rc.is_truthy(args.get(0).unwrap()))))
    }
}
