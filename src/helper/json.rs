//! Helper that returns a JSON string.
use crate::{
    helper::{Assertion, Context, Error, Helper, ValueResult},
    render::Render,
};

use serde_json::{to_string, to_string_pretty, Value};

/// The first argument is converted to a JSON string and returned.
///
/// Accepts an optional second argument which when *truthy* will
/// pretty print the value.
#[derive(Clone)]
pub struct JsonHelper;

impl Helper for JsonHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &mut Context<'source>,
    ) -> ValueResult {
        rc.arity(&ctx, 1..2)?;

        let args = ctx.arguments();
        let target = args.get(0).unwrap();

        let pretty = rc.is_truthy(args.get(0).unwrap_or(&Value::Bool(false)));
        let value = if pretty {
            Value::String(to_string_pretty(&target).map_err(Error::from)?)
        } else {
            Value::String(to_string(&target).map_err(Error::from)?)
        };

        Ok(Some(value))
    }
}
