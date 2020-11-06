//! Block helper for negated conditional.
use crate::{
    helper::{Assertion, BlockHelper, BlockResult, BlockTemplate, Context},
    render::Render,
};

pub struct UnlessHelper;

impl BlockHelper for UnlessHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
        block: BlockTemplate<'source>,
    ) -> BlockResult {
        rc.arity(&ctx, 1..1)?;

        if !rc.is_truthy(ctx.arguments().get(0).unwrap()) {
            rc.template(block.template())?;
        } else if let Some(node) = block.inverse(rc)? {
            rc.template(node)?;
        }

        Ok(())
    }
}