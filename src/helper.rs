use crate::{error::RenderError, render::RenderContext};

use serde_json::Value;

pub type Result = std::result::Result<Value, RenderError>;
pub type BlockResult = std::result::Result<(), RenderError>;

pub trait Helper: Send + Sync {
    fn call<'reg, 'render>(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result;
}

pub trait BlockHelper: Send + Sync {
    fn call<'reg, 'render>(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> BlockResult;
}

pub(crate) struct LogHelper;

impl Helper for LogHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result {
        Ok(Value::Null)
    }
}

pub(crate) struct LookupHelper;

impl Helper for LookupHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result {
        Ok(Value::Null)
    }
}

pub(crate) struct WithHelper;

impl BlockHelper for WithHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> BlockResult {
        Ok(())
    }
}

pub(crate) struct EachHelper;

impl BlockHelper for EachHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> BlockResult {
        Ok(())
    }
}

pub(crate) struct IfHelper;

impl BlockHelper for IfHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> BlockResult {
        Ok(())
    }
}

pub(crate) struct UnlessHelper;

impl BlockHelper for UnlessHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> BlockResult {
        Ok(())
    }
}
