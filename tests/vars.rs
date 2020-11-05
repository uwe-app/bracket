use bracket::{
    error::{Error, SyntaxError},
    Registry, Result,
};
use serde_json::json;

static NAME: &str = "vars.rs";

#[test]
fn vars_simple() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{foo}}";
    let data = json!({"foo": "<bar>"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("&lt;bar&gt;", &result);
    Ok(())
}

#[test]
fn vars_unescaped() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{{foo}}}";
    let data = json!({"foo": "<bar>"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("<bar>", &result);
    Ok(())
}

#[test]
fn vars_raw() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"\{{foo}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("{{foo}}", &result);
    Ok(())
}

#[test]
fn vars_this() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{this}}";
    let data = json!("bar");
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn vars_this_dot_slash() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{./}}";
    let data = json!("bar");
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn vars_root() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{@root.foo}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn vars_path() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{foo.bar.baz}}";
    let data = json!({"foo": {"bar": {"baz": "qux"}}});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("qux", &result);
    Ok(())
}

#[test]
fn vars_parent() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#with baz}}{{../foo}}{{/with}}";
    let data = json!({"foo": "bar", "baz": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn vars_local_index() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#each this}}{{@index}}{{/each}}";
    let data = json!(["foo"]);
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("0", &result);
    Ok(())
}

#[test]
fn vars_local_key() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#each this}}{{@key}}{{/each}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("foo", &result);
    Ok(())
}

#[test]
fn vars_arr_local_last() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#each this}}{{@last}}{{/each}}";
    let data = json!(["foo"]);
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("true", &result);
    Ok(())
}

#[test]
fn vars_map_local_last() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#each this}}{{@last}}{{/each}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("true", &result);
    Ok(())
}

#[test]
fn vars_arr_local_first() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#each this}}{{@first}}{{/each}}";
    let data = json!(["foo"]);
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("true", &result);
    Ok(())
}

#[test]
fn vars_map_local_first() -> Result<()> {
    let mut registry = Registry::new();
    let value = r"{{#each this}}{{@first}}{{/each}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("true", &result);
    Ok(())
}
