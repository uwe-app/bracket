use std::convert::TryFrom;

use bracket::{
    error::{Error, SyntaxError},
    helper::*,
    render::{Context, Render},
    template::{Loader, Templates},
    Registry, Result,
};
use serde_json::{json, Value};

static NAME: &str = "helper.rs";

#[derive(Clone)]
pub struct FooHelper;

impl Helper for FooHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
    ) -> ValueResult {
        Ok(Some(Value::String("bar".to_string())))
    }
}

#[derive(Clone)]
pub struct FooBlockHelper;

impl Helper for FooBlockHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
    ) -> ValueResult {
        rc.register_helper("foo", Box::new(FooHelper {}));

        if let Some(template) = ctx.template() {
            rc.template(template)?;
        }

        Ok(None)
    }
}

#[derive(Clone)]
pub struct MissingBlockHelper;

impl Helper for MissingBlockHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
    ) -> ValueResult {
        rc.write("bar")?;
        Ok(None)
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
        .register_helper("block", Box::new(FooBlockHelper {}));
    let value = r"{{#block}}{{foo}}{{/block}}";
    // NOTE: the helper takes precedence over the variable
    let data = json!({"foo": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

/*
#[test]
fn helper_block_missing() -> Result<()> {
    let mut registry = Registry::new();
    registry
        .helpers_mut()
        .register_helper("blockHelperMissing", Box::new(MissingBlockHelper {}));
    let value = r"{{#block}}{{foo}}{{/block}}";
    // NOTE: the helper takes precedence over the variable
    let data = json!({"foo": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}
*/
