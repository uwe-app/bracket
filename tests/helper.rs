use hbs::{
    helper::{Result, Helper},
    render::*,
    Registry,
};
use serde_json::Value;

pub(crate) struct MockHelper;

impl Helper for MockHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
    ) -> Result {
        Ok(Some(Value::Null))
    }
}

#[test]
fn helper_register() {
    let mut registry = Registry::new();
    registry.register_helper("mock", Box::new(MockHelper {}));
}
