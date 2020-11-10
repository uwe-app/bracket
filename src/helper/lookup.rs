//! Helper to lookup a field of an array or object.
use crate::{
    error::HelperError,
    helper::{Helper, HelperValue},
    parser::ast::Node,
    render::{Context, Render},
};

#[derive(Clone)]
pub struct LookupHelper;

impl Helper for LookupHelper {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
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

        let result = ctx.field(&target, field).cloned();
        if result.is_none() {
            Err(HelperError::LookupField(
                name.to_string(),
                field.to_string(),
            ))
        } else {
            Ok(result)
        }
    }
}
