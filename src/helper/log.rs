//! Helper to print log messages.
use crate::{
    helper::{Helper, HelperValue},
    json,
    parser::ast::Node,
    render::{Context, Render},
};

use log::*;

/// Helper that prints a log message.
///
/// Multiple arguments are accepted and concatenated using a
/// space before being sent to the log output.
///
/// Values are coerced to strings before concatenation with
/// special handling for `Value::String` so that it is not quoted.
///
/// Use the `level` hash parameter to set the log level to one of:
///
/// * trace
/// * debug
/// * info
/// * warn
/// * error
///
pub struct Log;

impl Helper for Log {
    fn call<'render, 'call>(
        &self,
        _rc: &mut Render<'render>,
        ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        ctx.arity(1..usize::MAX)?;

        let message = ctx
            .arguments()
            .iter()
            .map(|v| json::unquote(v))
            .collect::<Vec<String>>()
            .join(" ");

        let level = ctx
            .param("level")
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
