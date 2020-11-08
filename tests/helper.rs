use std::convert::TryFrom;

use bracket::{
    error::{Error, SyntaxError},
    helper::*,
    render::{BlockTemplate, Context, Render},
    template::{Loader, Templates},
    Registry, Result,
};
use serde_json::{json, Value};

static NAME: &str = "helper.rs";

#[derive(Clone)]
pub struct FooHelper;

impl Helper for FooHelper {
    fn call<'reg, 'source, 'render, 'call>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &mut Context<'reg, 'source, 'render, 'call>,
    ) -> ValueResult {
        Ok(Some(Value::String("bar".to_string())))
    }
}

#[derive(Clone)]
pub struct FooBlockHelper;

impl BlockHelper for FooBlockHelper {
    fn call<'reg, 'source, 'render, 'call>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &mut Context<'reg, 'source, 'render, 'call>,
        block: BlockTemplate<'source>,
    ) -> BlockResult {
        rc.register_helper("foo", Box::new(FooHelper {}));

        rc.template(block.template())?;

        Ok(())
    }
}

#[test]
fn helper_value() -> Result<()> {
    let mut registry = Registry::new();
    registry
        .helpers_mut()
        .register_helper("foo", Box::new(FooHelper {}));
    let value = r"{{foo}}";
    // NOTE: the helper takes precedence over the variable
    let data = json!({"foo": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn helper_explicit_this() -> Result<()> {
    let mut registry = Registry::new();
    registry
        .helpers_mut()
        .register_helper("foo", Box::new(FooHelper {}));
    let value = r"{{this.foo}}";
    // NOTE: explicit this causes the variable to take precedence
    let data = json!({"foo": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("qux", &result);
    Ok(())
}

#[test]
fn helper_explicit_this_dot_slash() -> Result<()> {
    let mut registry = Registry::new();
    registry
        .helpers_mut()
        .register_helper("foo", Box::new(FooHelper {}));
    let value = r"{{./foo}}";
    // NOTE: explicit ./ causes the variable to take precedence
    let data = json!({"foo": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("qux", &result);
    Ok(())
}

#[test]
fn helper_block() -> Result<()> {
    let mut registry = Registry::new();
    registry
        .helpers_mut()
        .register_block_helper("block", Box::new(FooBlockHelper {}));
    let value = r"{{#block}}{{foo}}{{/block}}";
    // NOTE: the helper takes precedence over the variable
    let data = json!({"foo": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}
