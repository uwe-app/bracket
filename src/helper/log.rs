//! Helper to print log messages.
use crate::{
    json,
    helper::{Helper, ValueResult},
    render::{Context, Render},
};

use log::*;

/// Helper that prints a log message.
///
/// Use the `level` hash parameter to set the log level to one of:
///
/// * trace
/// * debug
/// * info
/// * warn
/// * error
///
#[derive(Clone)]
pub struct LogHelper;

impl Helper for LogHelper {
    fn call<'reg, 'source, 'render, 'call>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &mut Context<'source, 'call>,
    ) -> ValueResult {
        ctx.arity(1..usize::MAX)?;

        let args = ctx.arguments();
        let hash = ctx.hash();

        let message = args
            .iter()
            .map(|v| json::unquote(v))
            .collect::<Vec<String>>()
            .join(" ");

        let level = hash
            .get("level")
            .map(|v| v.as_str())
            .unwrap_or(Some("info"))
            .unwrap();

        let lines = message.split("\n");
        for line in lines {
            match level {
                "error" => error!("{}", line),
                "debug" => debug!("{}", line),
                "warn" => warn!("{}", line),
                "trace" => trace!("{}", line),
                _ => info!("{}", line),
            }
        }

        Ok(None)
    }
}
