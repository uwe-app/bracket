use bracket::{Registry, Result};
use serde_json::json;

const NAME: &str = "each.rs";

#[test]
fn each_array() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#each foo}}{{this}}{{/each}}";
    let data = json!({"foo": ["b", "a", "r"]});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn each_array_index() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#each foo}}{{@index}}{{/each}}";
    let data = json!({"foo": ["b", "a", "r"]});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("012", &result);
    Ok(())
}

#[test]
fn each_map() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#each foo}}{{this}}{{/each}}";
    let data = json!({"foo": {"bar": "baz", "buz": "qux"}});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bazqux", &result);
    Ok(())
}

#[test]
fn each_map_key() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#each foo}}{{@key}}{{/each}}";
    let data = json!({"foo": {"bar": "baz", "buz": "qux"}});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("barbuz", &result);
    Ok(())
}
