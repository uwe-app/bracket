//! Helpers for numerical comparisons.
//!
//! Arguments must be numerical values otherwise a type assertion
//! error is returned.
//!
//! Values are compared as `f64`.
use crate::{
    error::HelperError,
    helper::{Helper, HelperValue},
    parser::ast::Node,
    render::{Context, Render, Type},
};

use serde_json::Value;

fn cmp<'call, F>(ctx: &Context<'call>, cmp: F) -> HelperValue
where
    F: FnOnce(f64, f64) -> bool,
{
    ctx.arity(2..2)?;

    let lhs = ctx.try_get(0, &[Type::Number])?;
    let rhs = ctx.try_get(1, &[Type::Number])?;

    match (lhs, rhs) {
        (Value::Number(lhs), Value::Number(rhs)) => {
            if let (Some(lhs), Some(rhs)) = (lhs.as_f64(), rhs.as_f64()) {
                Ok(Some(Value::Bool(cmp(lhs, rhs))))
            } else {
                Err(HelperError::InvalidNumericalOperand(
                    ctx.name().to_string(),
                ))
            }
        }
        _ => Err(HelperError::InvalidNumericalOperand(ctx.name().to_string())),
    }
}

/// Perform an equality comparison.
pub struct Equal;

impl Helper for Equal {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        cmp(ctx, |lhs: f64, rhs: f64| lhs == rhs)
    }
}

/// Perform a negated equality comparison.
pub struct NotEqual;

impl Helper for NotEqual {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        cmp(ctx, |lhs: f64, rhs: f64| lhs != rhs)
    }
}

/// Perform a numerical greater than comparison.
pub struct GreaterThan;

impl Helper for GreaterThan {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        cmp(ctx, |lhs: f64, rhs: f64| lhs > rhs)
    }
}

/// Perform a numerical greater than or equal comparison.
pub struct GreaterThanEqual;

impl Helper for GreaterThanEqual {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        cmp(ctx, |lhs: f64, rhs: f64| lhs >= rhs)
    }
}

/// Perform a numerical less than comparison.
pub struct LessThan;

impl Helper for LessThan {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        cmp(ctx, |lhs: f64, rhs: f64| lhs < rhs)
    }
}

/// Perform a numerical less than comparison.
pub struct LessThanEqual;

impl Helper for LessThanEqual {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        cmp(ctx, |lhs: f64, rhs: f64| lhs <= rhs)
    }
}
