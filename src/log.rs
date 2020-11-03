//! Helper to print log messages.
use crate::{
    error::HelperError as Error,
    helper::{Context, Helper, ValueResult},
    render::Render,
};

use log::*;

pub(crate) struct LogHelper;

impl Helper for LogHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
    ) -> ValueResult {
        ctx.assert_arity(1..usize::MAX)?;

        let (name, args, hash) = ctx.into();
        let message = args
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(" ");

        let level = hash
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
