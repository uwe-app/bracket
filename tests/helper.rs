use bracket::{helper::prelude::*, Registry, Result};
use serde_json::{json, Value};

static NAME: &str = "helper.rs";

#[derive(Clone)]
pub struct FooHelper;
impl Helper for FooHelper {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        _ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        Ok(Some(Value::String("bar".to_string())))
    }
}
impl LocalHelper for FooHelper {}

pub struct FooBlockHelper;
impl Helper for FooBlockHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        _ctx: &Context<'call>,
        template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        rc.register_local_helper("foo", Box::new(FooHelper {}));

        if let Some(template) = template {
            rc.template(template)?;
        }

        rc.unregister_local_helper("foo");

        Ok(None)
    }
}

pub struct HelperMissing;
impl Helper for HelperMissing {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        _ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        rc.write("bar")?;
        Ok(None)
    }
}

pub struct BlockHelperMissing;
impl Helper for BlockHelperMissing {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        // Get the property value and write it out
        let prop = ctx.property().as_ref().unwrap();
        let value = prop.value.as_str().unwrap();
        rc.write(value)?;
        Ok(None)
    }
}

#[test]
fn helper_value() -> Result<()> {
    let mut registry = Registry::new();
    registry.helpers_mut().insert("foo", Box::new(FooHelper {}));
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
    registry.helpers_mut().insert("foo", Box::new(FooHelper {}));
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
    registry.helpers_mut().insert("foo", Box::new(FooHelper {}));
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
        .insert("block", Box::new(FooBlockHelper {}));
    let value = r"{{#block}}{{foo}}{{/block}}";
    // NOTE: the helper takes precedence over the variable
    let data = json!({"foo": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn helper_missing() -> Result<()> {
    let mut registry = Registry::new();
    registry.handlers_mut().helper_missing = Some(Box::new(HelperMissing {}));

    let value = r"{{baz}}";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn helper_block_missing() -> Result<()> {
    let mut registry = Registry::new();
    registry.handlers_mut().block_helper_missing = Some(Box::new(BlockHelperMissing {}));
    let value = r"{{#block}}{{foo}}{{/block}}";
    // NOTE: the variable must exist for `blockHelperMissing` to fire
    let data = json!({"block": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}
