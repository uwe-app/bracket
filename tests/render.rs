use bracket::{Registry, Result};
use serde_json::json;

static NAME: &str = "render.rs";

#[test]
fn render_text() -> Result<()> {
    let registry = Registry::new();
    let value = r"Some text";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!(value, result);
    Ok(())
}

#[test]
fn render_html_comment() -> Result<()> {
    let registry = Registry::new();
    let value = r"<!-- foo -->";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!(value, result);
    Ok(())
}

#[test]
fn render_comment() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{! simple comment }}";
    let expected = r"";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!(expected, result);
    Ok(())
}

#[test]
fn render_raw_comment() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{!-- foo {{bar}} baz --}}";
    let expected = r"";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!(expected, result);
    Ok(())
}

#[test]
fn render_raw_statement() -> Result<()> {
    let registry = Registry::new();
    let value = r"\{{expr}}";
    let expected = r"{{expr}}";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!(expected, result);
    Ok(())
}

#[test]
fn render_statement() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{foo}}";
    let expected = r"bar";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!(expected, result);
    Ok(())
}
