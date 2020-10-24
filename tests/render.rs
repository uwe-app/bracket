use hbs::{Registry, Result};
use serde_json::json;

#[test]
fn render_text() -> Result<()> {
    let mut registry = Registry::new();
    let name = "mock-template";
    let value = r"Some text";
    let data = json!({});
    registry.register_template_string(name, value)?;
    let result = registry.render(name, &data)?;
    assert_eq!(value, result);

    // Verify removing templates
    registry.unregister_template(name);
    assert_eq!(0, registry.templates().len());

    Ok(())
}

#[test]
fn render_raw() -> Result<()> {
    let mut registry = Registry::new();
    let name = "mock-template";
    let value = r"{{{{raw}}}}foo {{bar}} baz{{{{/raw}}}}";
    let expected = r"foo {{bar}} baz";
    let data = json!({});
    registry.register_template_string(name, value)?;
    let result = registry.render(name, &data)?;
    assert_eq!(expected, result);
    Ok(())
}

#[test]
fn render_raw_multiline() -> Result<()> {
    let mut registry = Registry::new();
    let name = "mock-template";
    let value = r"some{{{{raw}}}}
foo
{{bar}}
baz{{{{/raw}}}}
text";
    let expected = r"some
foo
{{bar}}
baz
text";
    let data = json!({});
    registry.register_template_string(name, value)?;
    let result = registry.render(name, &data)?;
    assert_eq!(expected, result);
    Ok(())
}

#[test]
fn render_raw_comment() -> Result<()> {
    let mut registry = Registry::new();
    let name = "mock-template";
    let value = r"{{!-- foo {{bar}} baz --}}";
    let expected = r"";
    let data = json!({});
    registry.register_template_string(name, value)?;
    let result = registry.render(name, &data)?;
    assert_eq!(expected, result);
    Ok(())
}

#[test]
fn render_raw_statement() -> Result<()> {
    let mut registry = Registry::new();
    let name = "mock-template";
    let value = r"\{{}}";
    let expected = r"{{expr}}";
    let data = json!({});
    registry.register_template_string(name, value)?;
    let result = registry.render(name, &data)?;
    assert_eq!(expected, result);
    Ok(())
}

