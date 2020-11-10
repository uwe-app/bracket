//! Block helper for negated conditional.
use crate::{
    helper::{Helper, ValueResult},
    parser::ast::Node,
    render::{Context, Render},
};

#[derive(Clone)]
pub struct UnlessHelper;

impl Helper for UnlessHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        template: Option<&'render Node<'render>>,
    ) -> ValueResult {
        ctx.arity(1..1)?;

        if let Some(template) = template {
            if !rc.is_truthy(ctx.arguments().get(0).unwrap()) {
                rc.template(template)?;
            } else if let Some(node) = rc.inverse(template)? {
                rc.template(node)?;
            }
        }

        Ok(None)
    }
}
