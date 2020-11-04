use crate::{
    helper::{Context, Error, Helper, ValueResult},
    render::Render,
};

pub(crate) struct LookupHelper;

impl Helper for LookupHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
    ) -> ValueResult {
        ctx.assert_arity(2..2)?;

        let name = ctx.name();
        let mut args = ctx.into_arguments();
        let target = args.swap_remove(0);

        let field = args
            .get(0)
            .ok_or_else(|| Error::ArityExact(name.to_string(), 2))?
            .as_str()
            .ok_or_else(|| Error::ArgumentTypeString(name.to_string(), 1))?;

        let result = rc.field(&target, field).cloned();

        // TODO: error if no field was found!

        Ok(result)
    }
}
