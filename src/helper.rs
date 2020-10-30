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
        let message = ctx
            .arguments()
            .get(0)
            .ok_or_else(|| {
                RenderError::from("Arity error for `log` argument expected")
            })?
            .as_str()
            .ok_or_else(|| {
                RenderError::from(
                    "Type error for `log` helper, string expected",
                )
            })?
            .to_string();

        let level = ctx
            .hash()
            .get("level")
            .map(|v| v.as_str())
            .unwrap_or(Some("info"))
            .unwrap();

        match level {
            "error" => error!("{}", message),
            "debug" => debug!("{}", message),
            "warn" => warn!("{}", message),
            "trace" => trace!("{}", message),
            _ => info!("{}", message),
        }

        //println!("Log level {:?}", &level);

        // TODO: log levels!
        //info!("{}", message);

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
