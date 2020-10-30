use crate::{
    error::RenderError,
    render::{Context, Render},
};

use serde_json::Value;

use log::*;

pub type Result = std::result::Result<Option<Value>, RenderError>;

pub trait Helper: Send + Sync {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
        ctx: &Context<'render>,
    ) -> Result;
}

pub(crate) struct LogHelper;

impl Helper for LogHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
        ctx: &Context<'render>,
    ) -> Result {
        println!("Log helper got arguments {:?}", ctx.arguments());
        info!("THE LOG HELPER WAS CALLED via logging...");
        Ok(None)
    }
}

pub(crate) struct LookupHelper;

impl Helper for LookupHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
        ctx: &Context<'render>,
    ) -> Result {
        Ok(None)
    }
}

pub(crate) struct WithHelper;

impl Helper for WithHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
        ctx: &Context<'render>,
    ) -> Result {
        Ok(None)
    }
}

pub(crate) struct EachHelper;

impl Helper for EachHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
        ctx: &Context<'render>,
    ) -> Result {
        Ok(None)
    }
}

pub(crate) struct IfHelper;

impl Helper for IfHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
        ctx: &Context<'render>,
    ) -> Result {
        Ok(None)
    }
}

pub(crate) struct UnlessHelper;

impl Helper for UnlessHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
        ctx: &Context<'render>,
    ) -> Result {
        Ok(None)
    }
}
