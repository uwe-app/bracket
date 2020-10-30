use hbs::{
    helper::{self, BlockHelper, Helper},
    render::*,
    Registry, Result,
};
use serde_json::Value;

pub(crate) struct MockHelper;

impl Helper for MockHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
    ) -> helper::Result {
        Ok(Value::Null)
    }
}

pub(crate) struct MockBlockHelper;

impl BlockHelper for MockBlockHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
    ) -> helper::BlockResult {
        Ok(())
    }
}

#[test]
fn helper_register() -> Result<'static, ()> {
    let mut registry = Registry::new();
    registry.register_helper("mock", Box::new(MockHelper {}));
    registry.register_block_helper("block", Box::new(MockBlockHelper {}));
    Ok(())
}
