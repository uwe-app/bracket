//! Block helper for negated conditional.
use crate::{
    helper::{Helper, ValueResult},
    render::{Context, Render},
};

#[derive(Clone)]
pub struct UnlessHelper;

impl Helper for UnlessHelper {
    fn call<'reg, 'source, 'render, 'call>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &Context<'call>,
    ) -> ValueResult {
        ctx.arity(1..1)?;

        if let Some(template) = ctx.template() {
            if !rc.is_truthy(ctx.arguments().get(0).unwrap()) {
                rc.template(template)?;
            } else if let Some(node) = ctx.inverse(rc)? {
                rc.template(node)?;
            }
        }

        Ok(None)
    }
}
