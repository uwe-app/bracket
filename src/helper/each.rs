//! Block helper that iterates arrays and objects.
use crate::{
    //error::HelperError,
    helper::{Helper, HelperValue},
    parser::ast::Node,
    render::{Context, Render, Scope},
};

use serde_json::{Number, Value};

const FIRST: &str = "first";
const LAST: &str = "last";
const KEY: &str = "key";
const INDEX: &str = "index";

/// Iterate an array or object.
///
/// Accepts a single argument of the target to iterate, if the
/// target is not an array or object this will return an error.
///
/// Each iteration sets a new scope with the local variables:
///
/// * `@first`: If this is the first iteration `true`.
/// * `@last`: If this is the last iteration `true`.
///
/// Note that these variables are set even for objects where iteration order
/// is not guaranteed which can be useful.
///
/// For objects the `@key` variable contains the name of the field; for
/// arrays the `@index` variable contains the current zero-based index.
///
pub struct Each;

impl Helper for Each {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        ctx.arity(1..1)?;

        if let Some(template) = template {
            //let name = ctx.name();
            let args = ctx.arguments();
            let target = args.get(0).unwrap();

            rc.push_scope(Scope::new());
            match target {
                Value::Object(t) => {
                    let mut it = t.into_iter().enumerate();
                    let mut next_value = it.next();
                    while let Some((index, (key, value))) = next_value {
                        next_value = it.next();
                        if let Some(ref mut scope) = rc.scope_mut() {
                            scope.set_local(FIRST, Value::Bool(index == 0));
                            scope.set_local(
                                LAST,
                                Value::Bool(next_value.is_none()),
                            );
                            scope.set_local(
                                INDEX,
                                Value::Number(Number::from(index)),
                            );
                            scope.set_local(KEY, Value::String(key.to_owned()));
                            scope.set_base_value(value.clone());
                        }
                        rc.template(template)?;
                    }
                }
                Value::Array(t) => {
                    let len = t.len();
                    for (index, value) in t.into_iter().enumerate() {
                        if let Some(ref mut scope) = rc.scope_mut() {
                            scope.set_local(FIRST, Value::Bool(index == 0));
                            scope
                                .set_local(LAST, Value::Bool(index == len - 1));
                            scope.set_local(
                                INDEX,
                                Value::Number(Number::from(index)),
                            );
                            scope.set_base_value(value.clone());
                        }
                        rc.template(template)?;
                    }
                }
                _ => {
                    //return Err(HelperError::IterableExpected(
                    //name.to_string(),
                    //0,
                    //))
                }
            }
            rc.pop_scope();
        }

        Ok(None)
    }
}
