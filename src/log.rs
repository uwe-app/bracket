//! Helper to print log messages.
use std::collections::HashMap;
use crate::{error::RenderError, helper::{Helper, Result}, render::Render, parser::ast::Node};

use serde_json::Value;

use log::*;

pub(crate) struct LogHelper;

impl Helper for LogHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        arguments: &mut Vec<&Value>,
        hash: &mut HashMap<String, &'source Value>,
        template: &mut Option<&'source Node<'source>>,
    ) -> Result {
        
        //let args = rc.arguments();
        //let hash = rc.hash();

        let message = arguments
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

