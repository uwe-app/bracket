//! Helper to print log messages.
use crate::{
    error::HelperError as Error,
    helper::{Helper, Result, Context},
    render::Render
};

use log::*;

pub(crate) struct LogHelper;

impl Helper for LogHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &Context<'source>,
    ) -> Result {

        let message = ctx
            .arguments()
            .get(0)
            .ok_or_else(|| Error::ArityExact(ctx.name().to_string(), 1))?
            .as_str()
            .ok_or_else(|| Error::ArgumentTypeString(ctx.name().to_string(), 1))?
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

        Ok(None)
    }
}
