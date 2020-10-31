use hbs::{
    helper::{Result, Helper},
    parser::ast::Node,
    render::*,
    Registry,
};
use serde_json::Value;

pub(crate) struct MockHelper;

impl Helper for MockHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        template: Option<&'source Node<'source>>,
    ) -> Result {
        Ok(Some(Value::Null))
    }
}

#[test]
fn helper_register() {
    let mut registry = Registry::new();
    registry.register_helper("mock", Box::new(MockHelper {}));
}
