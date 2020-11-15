use bracket::{Registry, Result};
use serde_json::json;

static NAME: &str = "defaults.rs";

#[test]
fn defaults_statement() -> Result<()> {
    let registry = Registry::new();
    let value = r"foo{{qux}}bar";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("foobar", &result);
    Ok(())
}

#[test]
fn defaults_block() -> Result<()> {
    let registry = Registry::new();
    let value = r"foo{{#qux}}baz{{/qux}}bar";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("foobar", &result);
    Ok(())
}

#[test]
fn defaults_statement_strict() -> Result<()> {
    let mut registry = Registry::new();
    registry.set_strict(true);
    let value = r"foo{{qux}}bar";
    let data = json!({});
    if let Ok(_) = registry.once(NAME, value, &data) {
        panic!("Expecting missing variable error in strict mode.");
    }
    Ok(())
}

#[test]
fn defaults_block_strict() -> Result<()> {
    let mut registry = Registry::new();
    registry.set_strict(true);
    let value = r"foo{{#qux}}baz{{/qux}}bar";
    let data = json!({});
    if let Ok(_) = registry.once(NAME, value, &data) {
        panic!("Expecting missing helper error in strict mode.");
    }
    Ok(())
}
