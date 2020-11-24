use bracket::{
    Registry, Result,
};
use serde_json::json;

static NAME: &str = "partial.rs";

#[test]
fn partial_statement() -> Result<()> {
    let mut registry = Registry::new();
    registry.insert("foo", "{{bar}}".to_string());
    registry.build()?;

    let value = r"{{ > foo }}";
    let data = json!({"bar": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("qux", &result);
    Ok(())
}

#[test]
fn partial_sub_expr() -> Result<()> {
    let mut registry = Registry::new();
    registry.insert("bar", "{{baz}}".to_string());
    registry.build()?;

    let value = r"{{ > (foo) }}";
    let data = json!({"foo": "bar", "baz": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("qux", &result);
    Ok(())
}

#[test]
fn partial_block() -> Result<()> {
    let mut registry = Registry::new();
    registry.insert("foo", "{{> @partial-block}}".to_string());
    registry.build()?;

    let value = r"{{#>foo}}{{bar}}{{/foo}}";
    let data = json!({"bar": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("qux", &result);
    Ok(())
}
