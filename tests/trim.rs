use bracket::{
    error::{Error, SyntaxError},
    Registry, Result,
};
use serde_json::json;

static NAME: &str = "trim.rs";

#[test]
fn trim_statement() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"
{{~foo~}}
";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn trim_before_block_start() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"
{{~#if true}}{{foo}}{{/if}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn trim_after_block_start() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#if true~}}
{{foo}}{{/if}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn trim_before_block_end() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#if true}}{{foo}}
{{~/if}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn trim_after_block_end() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#if true}}{{foo}}{{/if~}}
";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}
