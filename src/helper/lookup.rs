//! Helper to lookup a field of an array or object.
use crate::{
    error::HelperError,
    helper::{Helper, HelperValue},
    parser::ast::Node,
    render::{Context, Render, Type},
    parser::ast::Slice,
};

/// Lookup a field of an array of object.
///
/// Requires exactly two arguments; the first is the target
/// value and the second is a string field name.
///
/// If the target field could not be found this helper will
/// return an error.
pub struct Lookup;

impl Helper for Lookup {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        ctx.arity(2..2)?;

        let target = ctx.get(0).unwrap();
        let field = ctx.try_get(1, &[Type::String]).unwrap().as_str().unwrap();

        if let Some(result) = ctx.lookup(&target, field).cloned() {
            Ok(Some(result))
        } else {
            Err(HelperError::LookupField(
                ctx.name().to_string(),
                field.to_string(),
            ))
        }
    }
}
