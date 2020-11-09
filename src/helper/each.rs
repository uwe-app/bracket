//! Block helper that iterates arrays and objects.
use crate::{
    error::HelperError,
    helper::{Helper, ValueResult},
    render::{Context, Render, Scope},
};

use serde_json::{Number, Value};

static FIRST: &str = "first";
static LAST: &str = "last";
static KEY: &str = "key";
static INDEX: &str = "index";

#[derive(Clone)]
pub struct EachHelper;

impl Helper for EachHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
    ) -> ValueResult {
        ctx.arity(1..1)?;

        if let Some(template) = ctx.template() {
            let name = ctx.name();
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
                    return Err(HelperError::IterableExpected(
                        name.to_string(),
                        1,
                    ))
                }
            }
            rc.pop_scope();
        }

        Ok(None)
    }
}
