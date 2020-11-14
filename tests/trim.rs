use bracket::{Registry, Result};
use serde_json::json;

static NAME: &str = "trim.rs";

#[test]
fn trim_statement() -> Result<()> {
    let registry = Registry::new();
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
    let registry = Registry::new();
    let value = r"
{{~#if true}}{{foo}}{{/if}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn trim_after_block_start() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#if true~}}
{{foo}}{{/if}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn trim_before_block_end() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#if true}}{{foo}}
{{~/if}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn trim_after_block_end() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#if true}}{{foo}}{{/if~}}
";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn trim_condition_after() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#if false}}WRONG{{else~}}
{{foo}}{{/if~}}
";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn trim_condition_if() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#if true ~}}
{{foo}}
{{~else~}}
{{foo}}
{{~/if~}}
";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn trim_condition_else() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#if false ~}}
{{foo}}
{{~else~}}
{{foo}}
{{~/if~}}
";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn trim_raw_block_outside() -> Result<()> {
    let registry = Registry::new();
    let value = r"
{{{{~raw}}}}bar{{{{/raw~}}}}
";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn trim_raw_block_inside() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{{{raw~}}}}
bar
{{{{~/raw}}}}";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}
