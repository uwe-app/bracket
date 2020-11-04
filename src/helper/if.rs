use crate::{
    helper::{BlockHelper, BlockTemplate, Context, Result},
    render::Render,
};

pub(crate) struct IfHelper;

impl BlockHelper for IfHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
        block: BlockTemplate<'source>,
    ) -> Result {
        ctx.assert_arity(1..1)?;

        if rc.is_truthy(ctx.arguments().get(0).unwrap()) {
            rc.template(block.template())?;
        } else {
            let inverse = block.inverse();
            if let Some(node) = inverse {
                rc.template(node)?;
            }
        }
        Ok(())
    }
}
