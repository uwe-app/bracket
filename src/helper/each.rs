use crate::{
    helper::{BlockHelper, BlockTemplate, Context, Error, Result},
    render::{Render, Scope},
};

use serde_json::{Number, Value};

static FIRST: &str = "first";
static LAST: &str = "last";
static KEY: &str = "key";
static INDEX: &str = "index";

pub(crate) struct EachHelper;

impl BlockHelper for EachHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
        block: BlockTemplate<'source>,
    ) -> Result {
        ctx.assert_arity(1..1)?;

        //let name = ctx.name().to_string();
        let (name, mut args) = ctx.into();
        let target = args.swap_remove(0);

        rc.push_scope(Scope::new());
        match target {
            Value::Object(t) => {
                let mut it = t.into_iter().enumerate();
                let mut next_value = it.next();
                while let Some((index, (key, value))) = next_value {
                    next_value = it.next();
                    if let Some(ref mut scope) = rc.scope_mut() {
                        scope.set_local(FIRST, Value::Bool(index == 0));
                        scope
                            .set_local(LAST, Value::Bool(next_value.is_none()));
                        scope.set_local(
                            INDEX,
                            Value::Number(Number::from(index)),
                        );
                        scope.set_local(KEY, Value::String(key.to_owned()));
                        scope.set_base_value(value);
                    }
                    rc.template(block.template())?;
                }
            }
            Value::Array(t) => {
                let len = t.len();
                for (index, value) in t.into_iter().enumerate() {
                    if let Some(ref mut scope) = rc.scope_mut() {
                        scope.set_local(FIRST, Value::Bool(index == 0));
                        scope.set_local(LAST, Value::Bool(index == len - 1));
                        scope.set_local(
                            INDEX,
                            Value::Number(Number::from(index)),
                        );
                        scope.set_base_value(value);
                    }
                    rc.template(block.template())?;
                }
            }
            _ => return Err(Error::IterableExpected(name, 1)),
        }
        rc.pop_scope();

        Ok(())
    }
}
