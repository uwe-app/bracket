use bracket::{Registry, Result};
use serde_json::json;

const NAME: &str = "partial.rs";

#[test]
fn partial_statement() -> Result<()> {
    let mut registry = Registry::new();
    registry.insert("foo", "{{bar}}".to_string())?;

    let value = r"{{ > foo }}";
    let data = json!({"bar": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("qux", &result);
    Ok(())
}

#[test]
fn partial_sub_expr() -> Result<()> {
    let mut registry = Registry::new();
    registry.insert("bar", "{{baz}}".to_string())?;

    let value = r"{{ > (foo) }}";
    let data = json!({"foo": "bar", "baz": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("qux", &result);
    Ok(())
}

#[test]
fn partial_block() -> Result<()> {
    let mut registry = Registry::new();
    registry.insert("foo", "{{> @partial-block}}".to_string())?;

    let value = r"{{#>foo}}{{bar}}{{/foo}}";
    let data = json!({"bar": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("qux", &result);
    Ok(())
}

#[test]
fn partial_context() -> Result<()> {
    let mut registry = Registry::new();
    registry.insert("foo", "{{bar}}".to_string())?;

    let value = r"{{ > foo ctx }}";
    let data = json!({"bar": "qux", "ctx": {"bar": "baz"}});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("baz", &result);
    Ok(())
}

#[test]
fn partial_context_parameter() -> Result<()> {
    let mut registry = Registry::new();
    registry.insert("foo", "{{bar}}".to_string())?;

    let value = r#"{{ > foo ctx bar="xyz"}}"#;
    let data = json!({"bar": "qux", "ctx": {"bar": "baz"}});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("xyz", &result);
    Ok(())
}
