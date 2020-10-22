use hbs::{Registry, Result};
use serde_json::json;

#[test]
fn register_template_string() -> Result<()> {
    let mut registry = Registry::new();
    let name = "mock-template";
    let value = r"Some text";
    registry.register_template_string(name, value)?;
    assert_eq!(1, registry.templates().len());
    registry.unregister_template(name);
    assert_eq!(0, registry.templates().len());
    Ok(())
}

#[test]
fn render_text() -> Result<()> {
    let mut registry = Registry::new();
    let name = "mock-template";
    let value = r"Some text";
    let data = json!({});
    registry.register_template_string(name, value)?;
    let result = registry.render(name, &data)?;
    assert_eq!(value, result);
    Ok(())
}
