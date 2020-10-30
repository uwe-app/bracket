use crate::{error::RenderError, helper::{Helper, Result}, render::Render};

use log::*;

pub(crate) struct LogHelper;

impl Helper for LogHelper {
    fn call<'reg, 'render>(
        &self,
        rc: &mut Render<'reg, 'render>,
    ) -> Result {
        let message = rc
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

        let level = rc
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

