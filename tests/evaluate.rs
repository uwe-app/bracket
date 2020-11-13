use bracket::{
    helper::*,
    parser::ast::Node,
    render::{Context, Render},
    Registry, Result,
};
use serde_json::{json, Value};

static NAME: &str = "evaluate.rs";

#[derive(Clone)]
pub struct EvalHelper;

impl Helper for EvalHelper {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        _ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        Ok(rc.evaluate("@root.foo")?.cloned())
    }
}

#[test]
fn helper_evaluate_path() -> Result<()> {
    let mut registry = Registry::new();
    registry.helpers_mut().insert("eval", Box::new(EvalHelper {}));
    let value = r"{{eval}}";
    let data = json!({"foo": "bar"});
    let result = registry.once(NAME, value, &data)?;
    assert_eq!("bar", &result);
    Ok(())
}

