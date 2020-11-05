//! Block helper for conditionals.
use crate::{
    helper::{Assertion, BlockHelper, BlockTemplate, Context, BlockResult},
    render::Render,
};

pub struct IfHelper;

impl BlockHelper for IfHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
        block: BlockTemplate<'source>,
    ) -> BlockResult {
        rc.arity(&ctx, 1..1)?;

        if rc.is_truthy(ctx.arguments().get(0).unwrap()) {
            rc.template(block.template())?;
        } else {
            let inverse = block.inverse(rc)?;
            if let Some(node) = inverse {
                rc.template(node)?;
            }
        }
        Ok(())
    }
}
