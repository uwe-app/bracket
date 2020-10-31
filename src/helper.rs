//! Helper trait and types for the default set of helpers.

use crate::{
    error::RenderError,
    render::{Render},
};

use serde_json::{Value, to_string, to_string_pretty};

/// The result that helper functions should return.
pub type Result = std::result::Result<Option<Value>, RenderError>;

/// Trait for helpers.
pub trait Helper: Send + Sync {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
    ) -> Result;
}

pub(crate) struct LookupHelper;

impl Helper for LookupHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
    ) -> Result {
        Ok(None)
    }
}

pub(crate) struct WithHelper;

impl Helper for WithHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
    ) -> Result {
        let args = rc.arguments();
        let scope = args
            .get(0)
            .ok_or_else(|| {
                RenderError::from("Arity error for `with`, argument expected")
            })?;

        //if let Some(ref ctx) = rc.context() {
            //let tpl = ctx.template();
            //rc.render(tpl);
        //}

        /*
        if let Some(template) = rc.template() {
            ////println!("WITH HELPER WAS CALLED");
            //template.foo();

        }
        */

        let block = rc.push_scope();
        block.set_base_value(&scope);
        //rc.render();
        rc.pop_scope();

        Ok(None)
    }
}

pub(crate) struct EachHelper;

impl Helper for EachHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
    ) -> Result {
        Ok(None)
    }
}

pub(crate) struct IfHelper;

impl Helper for IfHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
    ) -> Result {
        Ok(None)
    }
}

pub(crate) struct UnlessHelper;

impl Helper for UnlessHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
    ) -> Result {
        Ok(None)
    }
}

// Extended, non-standard helpers

pub(crate) struct JsonHelper;

impl Helper for JsonHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
    ) -> Result {
        let args = rc.arguments();

        let target = args
            .get(0)
            .ok_or_else(|| {
                RenderError::from("Arity error for `json`, argument expected")
            })?;

        let compact = rc.is_truthy(args
            .get(0)
            .unwrap_or(&&Value::Bool(false)));

        println!("JSON HELPER WAS CALLED");

        if compact {
            if let Ok(s) = to_string(target) {
                rc.out().write(s.as_bytes())?;
            }
        } else {
            if let Ok(s) = to_string_pretty(target) {
                rc.out().write(s.as_bytes())?;
            }
        }

        Ok(None)
    }
}
