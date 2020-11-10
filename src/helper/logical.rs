//! Helpers for conditional statements.
use crate::{
    helper::{Helper, ValueResult},
    parser::ast::Node,
    render::{Context, Render},
};

use serde_json::Value;

/// Perform a logical AND on two arguments.
#[derive(Clone)]
pub struct AndHelper;

impl Helper for AndHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> ValueResult {
        ctx.arity(2..2)?;

        let args = ctx.arguments();
        Ok(Some(Value::Bool(
            rc.is_truthy(args.get(0).unwrap())
                && rc.is_truthy(args.get(1).unwrap()),
        )))
    }
}

/// Perform a logical OR on two arguments.
#[derive(Clone)]
pub struct OrHelper;

impl Helper for OrHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> ValueResult {
        ctx.arity(2..2)?;

        let args = ctx.arguments();
        Ok(Some(Value::Bool(
            rc.is_truthy(args.get(0).unwrap())
                || rc.is_truthy(args.get(1).unwrap()),
        )))
    }
}

/// Perform a logical NOT on an argument.
#[derive(Clone)]
pub struct NotHelper;

impl Helper for NotHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> ValueResult {
        ctx.arity(1..1)?;

        let args = ctx.arguments();
        Ok(Some(Value::Bool(!rc.is_truthy(args.get(0).unwrap()))))
    }
}
