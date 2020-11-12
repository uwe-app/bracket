//! Helper that returns a JSON string.
use crate::{
    error::HelperError,
    helper::{Helper, HelperValue},
    parser::ast::Node,
    render::{Context, Render},
};

use serde_json::{to_string, to_string_pretty, Value};

/// Convert to a JSON string,
///
/// Accepts a single argument which is converted to a JSON string and returned.
///
/// The optional hash parameter `pretty` when *truthy* will pretty print the value.
#[derive(Clone)]
pub struct Json;

impl Helper for Json {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        ctx.arity(1..1)?;

        let target = ctx.get(0).unwrap();
        let pretty =
            ctx.is_truthy(ctx.hash("pretty").unwrap_or(&Value::Bool(false)));
        let value = if pretty {
            Value::String(to_string_pretty(&target).map_err(HelperError::from)?)
        } else {
            Value::String(to_string(&target).map_err(HelperError::from)?)
        };

        Ok(Some(value))
    }
}
