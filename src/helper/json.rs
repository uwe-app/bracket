//! Helper that returns a JSON string.
use crate::{
    error::HelperError,
    helper::{Helper, ValueResult},
    parser::ast::Node,
    render::{Context, Render},
};

use serde_json::{to_string, to_string_pretty, Value};

/// The first argument is converted to a JSON string and returned.
///
/// Accepts an optional second argument which when *truthy* will
/// pretty print the value.
#[derive(Clone)]
pub struct JsonHelper;

impl Helper for JsonHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> ValueResult {
        ctx.arity(1..2)?;

        let args = ctx.arguments();
        let target = args.get(0).unwrap();

        let pretty = rc.is_truthy(args.get(0).unwrap_or(&Value::Bool(false)));
        let value = if pretty {
            Value::String(to_string_pretty(&target).map_err(HelperError::from)?)
        } else {
            Value::String(to_string(&target).map_err(HelperError::from)?)
        };

        Ok(Some(value))
    }
}
