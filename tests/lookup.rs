use bracket::{Registry, Result};
use serde_json::json;

static NAME: &str = "lookup.rs";

#[test]
fn lookup_map() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{lookup map 'foo'}}";
    let data = json!({"map": {"foo": "bar"}});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn lookup_array() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{lookup list '1'}}";
    let data = json!({"list": ["foo", "bar", "qux"]});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn lookup_deep() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{lookup foo.bar.qux 'baz'}}";
    let data = json!({"foo": {"bar": {"qux": {"baz": "bar"}}}});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}
