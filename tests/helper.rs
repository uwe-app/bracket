use hbs::{
    helper::{self, Helper},
    render::*,
    Registry,
};
use serde_json::Value;

pub(crate) struct MockHelper;

impl Helper for MockHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
    ) -> helper::Result {
        Ok(Some(Value::Null))
    }
}

#[test]
fn helper_register() {
    let mut registry = Registry::new();
    registry.register_helper("mock", Box::new(MockHelper {}));
}
