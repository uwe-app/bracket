//! Block helper that sets the scope.
use crate::{
    helper::{Helper, ValueResult, BlockTemplate},
    render::{Context, Render, Scope},
};

use serde_json::Value;

#[derive(Clone)]
pub struct WithHelper;

impl Helper for WithHelper {
    fn call<'reg, 'source, 'render, 'call>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &mut Context<'source, 'call>,
    ) -> ValueResult {
        ctx.arity(1..1)?;

        if let Some(template) = ctx.template() {
            let args = ctx.arguments();
            let target = args.get(0).unwrap();
            rc.push_scope(Scope::new());
            if let Some(ref mut scope) = rc.scope_mut() {
                scope.set_base_value(target.clone());
            }
            rc.template(template)?;
            rc.pop_scope();
        }

        Ok(None)
    }
}
