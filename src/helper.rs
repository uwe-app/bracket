use crate::{
    error::RenderError,
    render::{Render, Scope},
};

use serde_json::Value;

pub type Result = std::result::Result<Option<Value>, RenderError>;

pub trait Helper: Send + Sync {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
    ) -> Result;
}

pub(crate) struct LookupHelper;

impl Helper for LookupHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
    ) -> Result {
        Ok(None)
    }
}

pub(crate) struct WithHelper;

impl Helper for WithHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
    ) -> Result {
        let args = rc.arguments();
        let scope = args
            .get(0)
            .ok_or_else(|| {
                RenderError::from("Arity error for `with`, argument expected")
            })?;

        let mut block = Scope::new();
        block.set_base_value(scope);

        rc.push_scope(block);

        rc.pop_scope();

        Ok(None)
    }
}

pub(crate) struct EachHelper;

impl Helper for EachHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
    ) -> Result {
        Ok(None)
    }
}

pub(crate) struct IfHelper;

impl Helper for IfHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
    ) -> Result {
        Ok(None)
    }
}

pub(crate) struct UnlessHelper;

impl Helper for UnlessHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
    ) -> Result {
        Ok(None)
    }
}
