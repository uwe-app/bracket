//! Helper to print log messages.
use crate::{
    error::RenderError,
    helper::{Helper, Result},
    render::{Render, Context}
};

use log::*;

pub(crate) struct LogHelper;

impl Helper for LogHelper {
    fn call<'reg, 'source, 'render>(
        &'source self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &Context<'source>,
    ) -> Result<'source> {

        let message = ctx
            .arguments()
            .get(0)
            .ok_or_else(|| {
                RenderError::from("Arity error for `log`, string message expected")
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

        Ok(None)
    }
}
