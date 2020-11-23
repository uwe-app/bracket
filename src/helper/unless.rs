//! Block helper for negated conditional.
use crate::{
    helper::{Helper, HelperValue},
    parser::ast::Node,
    render::{Context, Render},
};

/// Render an inner block when the conditional is **not** truthy.
///
/// If any `else` or `else if` conditionals are present they will
/// be rendered when necessary.
pub struct Unless;

impl Helper for Unless {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        ctx.arity(1..1)?;

        if let Some(template) = template {
            if !ctx.is_truthy(ctx.get(0).unwrap()) {
                rc.template(template)?;
            } else if let Some(node) = rc.inverse(template)? {
                rc.template(node)?;
            }
        }

        Ok(None)
    }
}
