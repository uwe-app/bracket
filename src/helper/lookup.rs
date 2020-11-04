//! Helper to lookup a field of an array or object.
use crate::{
    helper::{Assertion, Context, Error, Helper, ValueResult},
    render::Render,
};

pub struct LookupHelper;

impl Helper for LookupHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
    ) -> ValueResult {
        rc.arity(&ctx, 2..2)?;

        let name = ctx.name();
        let (name, mut args) = ctx.into();
        let target = args.swap_remove(0);

        let field = args
            .get(0)
            .ok_or_else(|| Error::ArityExact(name.to_string(), 2))?
            .as_str()
            .ok_or_else(|| Error::ArgumentTypeString(name.to_string(), 1))?;

        let result = rc.field(&target, field).cloned();
        if result.is_none() {
            Err(Error::Message(format!(
                "Helper '{}' failed to resolve field '{}'",
                name, field
            )))
        } else {
            Ok(result)
        }
    }
}
