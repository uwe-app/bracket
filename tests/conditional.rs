use bracket::{
    error::{Error, SyntaxError},
    Registry, Result,
};
use serde_json::json;

static NAME: &str = "conditional.rs";

#[test]
fn if_block() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#if true}}{{foo}}{{/if}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn if_else_block() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#if false}}WRONG{{else}}{{foo}}{{/if}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn if_else_if_block() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#if false}}WRONG{{else if true}}{{foo}}{{/if}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn if_else_if_else_block() -> Result<()> {
    let mut registry = Registry::new();
    let value =
        r"{{#if false}}WRONG{{else if false}}WRONG{{else}}{{foo}}{{/if}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn unless_block() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#unless false}}{{foo}}{{/unless}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn unless_else_block() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#unless true}}WRONG{{else}}{{foo}}{{/unless}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn if_and_block() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#if (and true true)}}{{foo}}{{/if}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn if_or_block() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#if (or false true)}}{{foo}}{{/if}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn if_not_block() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#if (not false)}}{{foo}}{{/if}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}
