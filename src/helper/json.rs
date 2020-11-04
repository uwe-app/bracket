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
pub struct JsonHelper;

impl Helper for JsonHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
    ) -> ValueResult {
        rc.arity(&ctx, 1..2)?;

        let mut args: Vec<Value> = ctx.into();
        let target = args.swap_remove(0);

        let pretty = rc.is_truthy(args.get(0).unwrap_or(&Value::Bool(false)));
        let value = if pretty {
            Value::String(to_string_pretty(&target).map_err(Error::from)?)
        } else {
            Value::String(to_string(&target).map_err(Error::from)?)
        };

        Ok(Some(value))
    }
}
