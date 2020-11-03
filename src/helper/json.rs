use crate::{
    render::Render,
    helper::{Helper, Context, Error, ValueResult},
};

use serde_json::{to_string, to_string_pretty, Value};

pub(crate) struct JsonHelper;

impl Helper for JsonHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
    ) -> ValueResult {
        ctx.assert_arity(1..2)?;

        let mut args = ctx.into_arguments();
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
