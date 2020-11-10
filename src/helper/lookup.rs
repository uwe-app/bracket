//! Helper to lookup a field of an array or object.
use crate::{
    error::HelperError,
    helper::{Helper, ValueResult},
    parser::ast::Node,
    render::{Context, Render},
};

#[derive(Clone)]
pub struct LookupHelper;

impl Helper for LookupHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> ValueResult {
        ctx.arity(2..2)?;

        let name = ctx.name();
        let args = ctx.arguments();
        let target = args.get(0).unwrap();

        let field = args
            .get(1)
            .ok_or_else(|| HelperError::ArityExact(name.to_string(), 2))?
            .as_str()
            .ok_or_else(|| {
                HelperError::ArgumentTypeString(name.to_string(), 1)
            })?;

        let result = rc.field(&target, field).cloned();
        if result.is_none() {
            Err(HelperError::Message(format!(
                "Helper '{}' failed to resolve field '{}'",
                name, field
            )))
        } else {
            Ok(result)
        }
    }
}
