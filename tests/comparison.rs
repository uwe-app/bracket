use bracket::{Registry, Result};
use serde_json::json;

const NAME: &str = "comparison.rs";

#[test]
fn cmp_eq() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#if (eq 1 1)}}bar{{/if}}";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn cmp_ne() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#if (ne 1 2)}}bar{{/if}}";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn cmp_gt() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#if (gt 2 1)}}bar{{/if}}";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn cmp_gte() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#if (gte 2 2)}}bar{{/if}}";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn cmp_lt() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#if (lt 1 2)}}bar{{/if}}";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn cmp_lte() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#if (lte 2 2)}}bar{{/if}}";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}
