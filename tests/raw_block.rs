use bracket::{
    helper::*,
    parser::ast::Node,
    render::{Context, Render},
    Registry, Result,
};
use serde_json::json;

const NAME: &str = "raw_block.rs";

#[derive(Clone)]
pub struct RawBlockHelper;

impl Helper for RawBlockHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        if let Some(text) = ctx.text() {
            rc.write(text)?;
        }
        Ok(None)
    }
}

#[test]
fn raw_block_helper() -> Result<()> {
    let mut registry = Registry::new();
    registry
        .helpers_mut()
        .insert("raw-helper", Box::new(RawBlockHelper {}));
    let value = r"{{{{raw-helper}}}}foo{{{{/raw-helper}}}}";
    let expected = r"foo";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!(expected, result);
    Ok(())
}

#[test]
fn raw_block() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{{{raw}}}}foo {{bar}} baz{{{{/raw}}}}";
    let expected = r"foo {{bar}} baz";
    let data = json!({});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!(expected, result);
    Ok(())
}

#[test]
fn raw_block_multiline() -> Result<()> {
    let registry = Registry::new();
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
    let result = registry.once(NAME, value, &data)?;
    assert_eq!(expected, result);
    Ok(())
}
