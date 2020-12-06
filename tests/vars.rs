use bracket::{Registry, Result};
use serde_json::json;

static NAME: &str = "vars.rs";

#[test]
fn vars_simple() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{foo}}";
    let data = json!({"foo": "<bar>"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("&lt;bar&gt;", &result);
    Ok(())
}

#[test]
fn vars_unescaped() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{{foo}}}";
    let data = json!({"foo": "<bar>"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("<bar>", &result);
    Ok(())
}

#[test]
fn vars_raw() -> Result<()> {
    let registry = Registry::new();
    let value = r"\{{foo}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("{{foo}}", &result);
    Ok(())
}

#[test]
fn vars_this() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{this}}";
    let data = json!("bar");
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn vars_this_dot_slash() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{./}}";
    let data = json!("bar");
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn vars_explicit_this_path() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{this.foo}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn vars_root() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{@root.foo}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn vars_path() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{foo.bar.baz}}";
    let data = json!({"foo": {"bar": {"baz": "qux"}}});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("qux", &result);
    Ok(())
}

#[test]
fn vars_parent() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#with baz}}{{../foo}}{{/with}}";
    let data = json!({"foo": "bar", "baz": "qux"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn vars_local_index() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#each this}}{{@index}}{{/each}}";
    let data = json!(["foo"]);
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("0", &result);
    Ok(())
}

#[test]
fn vars_local_key() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#each this}}{{@key}}{{/each}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("foo", &result);
    Ok(())
}

#[test]
fn vars_arr_local_last() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#each this}}{{@last}}{{/each}}";
    let data = json!(["foo"]);
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("true", &result);
    Ok(())
}

#[test]
fn vars_map_local_last() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#each this}}{{@last}}{{/each}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("true", &result);
    Ok(())
}

#[test]
fn vars_arr_local_first() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#each this}}{{@first}}{{/each}}";
    let data = json!(["foo"]);
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("true", &result);
    Ok(())
}

#[test]
fn vars_map_local_first() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#each this}}{{@first}}{{/each}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("true", &result);
    Ok(())
}

#[test]
fn vars_array_access() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{this.[1]}}";
    let data = json!(["foo", "bar", "baz"]);
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn vars_scope() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#each list}}{{title}}{{/each}}";
    let data = json!({"title": "foo", "list": [{"title": "bar"}]});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn vars_scope_parent() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#with item}}{{#each list}}{{title}}{{/each}}{{/with}}";
    let data = json!({"title": "foo", "item": {"title": "bar"}, "list": [1]});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn vars_scope_explicit_this() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#with item}}{{this.title}}{{/with}}";
    let data = json!({"title": "foo", "item": {"title": "bar"}});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

#[test]
fn vars_scope_explicit_this_no_inherit() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#with item}}{{this.title}}{{/with}}";
    let data = json!({"title": "foo", "item": {}});
    let result = registry.once(NAME, value, &data)?;
    // NOTE: due to the use of explicit `this` it should not 
    // NOTE: resolve in the parent scopes
    assert_eq!("", &result);
    Ok(())
}
