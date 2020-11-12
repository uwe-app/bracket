//! Helpers for conditional statements.
use crate::{
    helper::{Helper, HelperValue},
    parser::ast::Node,
    render::{Context, Render},
};

use serde_json::Value;

/// Perform a logical AND on two arguments.
#[derive(Clone)]
pub struct And;

impl Helper for And {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        ctx.arity(2..2)?;

        Ok(Some(Value::Bool(
            ctx.is_truthy(ctx.get(0).unwrap())
                && ctx.is_truthy(ctx.get(1).unwrap()),
        )))
    }
}

/// Perform a logical OR on two arguments.
#[derive(Clone)]
pub struct Or;

impl Helper for Or {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        ctx.arity(2..2)?;

        Ok(Some(Value::Bool(
            ctx.is_truthy(ctx.get(0).unwrap())
                || ctx.is_truthy(ctx.get(1).unwrap()),
        )))
    }
}

/// Perform a logical NOT on an argument.
#[derive(Clone)]
pub struct Not;

impl Helper for Not {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        ctx.arity(1..1)?;
        Ok(Some(Value::Bool(!ctx.is_truthy(ctx.get(0).unwrap()))))
    }
}
