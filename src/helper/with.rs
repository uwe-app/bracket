//! Block helper that sets the scope.
use crate::{
    helper::{Helper, HelperValue},
    parser::ast::Node,
    render::{Context, Render, Scope},
};

use serde_json::Value;

/// Set the scope for a block to the target argument.
pub struct With;

impl Helper for With {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        ctx.arity(1..1)?;

        if let Some(arg) = ctx.get(0) {
            let is_null = if let Value::Null = arg { true } else { false };
            if !is_null {
                if let Some(template) = template {
                    rc.push_scope(Scope::new());
                    if let Some(ref mut scope) = rc.scope_mut() {
                        scope.set_base_value(ctx.get(0).cloned().unwrap());
                    }
                    rc.template(template)?;
                    rc.pop_scope();
                }
            }
        }

        Ok(None)
    }
}
