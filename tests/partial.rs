use std::convert::TryFrom;

use bracket::{
    template::{Loader, Templates},
    Registry, Result,
};
use serde_json::json;

static NAME: &str = "partial.rs";

#[test]
fn partial_statement() -> Result<()> {
    let mut loader = Loader::new();
    loader.insert("foo", "{{bar}}".to_string());
    let templates = Templates::try_from(&loader)?;

    let registry = Registry::from(templates);
    let value = r"{{ > foo }}";
    let data = json!({"bar": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("qux", &result);
    Ok(())
}

#[test]
fn partial_sub_expr() -> Result<()> {
    let mut loader = Loader::new();
    loader.insert("bar", "{{baz}}".to_string());
    let templates = Templates::try_from(&loader)?;

    let registry = Registry::from(templates);
    let value = r"{{ > (foo) }}";
    let data = json!({"foo": "bar", "baz": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("qux", &result);
    Ok(())
}

#[test]
fn partial_block() -> Result<()> {
    let mut loader = Loader::new();
    loader.insert("foo", "{{@partial-block}}".to_string());
    let templates = Templates::try_from(&loader)?;

    let registry = Registry::from(templates);
    let value = r"{{#>foo}}{{bar}}{{/foo}}";
    let data = json!({"bar": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("qux", &result);
    Ok(())
}
