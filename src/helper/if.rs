//! Helpers for conditional statements.
use crate::{
    helper::{
        Assertion, BlockHelper, BlockResult, BlockTemplate, Context, Helper,
        ValueResult,
    },
    render::Render,
};

use serde_json::Value;

#[derive(Clone)]
pub struct IfHelper;

impl Helper for IfHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &mut Context<'source>,
    ) -> ValueResult {
        rc.arity(&ctx, 1..usize::MAX)?;

        let args = ctx.arguments();

        let mut result = Value::Bool(true);
        for val in args {
            if !rc.is_truthy(&val) {
                result = Value::Bool(false);
                break;
            }
        }
        Ok(Some(result))
    }
}

#[derive(Clone)]
pub struct IfBlockHelper;

impl BlockHelper for IfBlockHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &mut Context<'source>,
        block: BlockTemplate<'source>,
    ) -> BlockResult {
        rc.arity(&ctx, 1..1)?;

        if rc.is_truthy(ctx.arguments().get(0).unwrap()) {
            rc.template(block.template())?;
        } else if let Some(node) = block.inverse(rc)? {
            rc.template(node)?;
        }
        Ok(())
    }
}
