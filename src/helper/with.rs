use crate::{
    helper::{BlockHelper, BlockTemplate, Context, Result},
    render::{Render, Scope},
};

use serde_json::Value;

pub(crate) struct WithHelper;

impl BlockHelper for WithHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
        block: BlockTemplate<'source>,
    ) -> Result {
        ctx.assert_arity(1..1)?;

        let mut args: Vec<Value> = ctx.into();
        let target = args.swap_remove(0);
        rc.push_scope(Scope::new());
        if let Some(ref mut scope) = rc.scope_mut() {
            scope.set_base_value(target);
        }
        rc.template(block.template())?;
        rc.pop_scope();
        Ok(())
    }
}
