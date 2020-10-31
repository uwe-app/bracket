//! Helper trait and types for the default set of helpers.

use std::collections::HashMap;
use serde_json::{Value, to_string, to_string_pretty};

use crate::{
    error::RenderError,
    render::{Render},
    parser::ast::Node,
};

/// The result that helper functions should return.
pub type Result = std::result::Result<Option<Value>, RenderError>;

/// Trait for helpers.
pub trait Helper: Send + Sync {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        arguments: &mut Vec<&Value>,
        hash: &mut HashMap<String, &'source Value>,
        template: &mut Option<&'source Node<'source>>,
    ) -> Result;
}

pub(crate) struct LookupHelper;

impl Helper for LookupHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        arguments: &mut Vec<&Value>,
        hash: &mut HashMap<String, &'source Value>,
        template: &mut Option<&'source Node<'source>>,
    ) -> Result {
        Ok(None)
    }
}

pub(crate) struct WithHelper;

impl Helper for WithHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        arguments: &mut Vec<&Value>,
        hash: &mut HashMap<String, &'source Value>,
        template: &mut Option<&'source Node<'source>>,
    ) -> Result {

        let scope = arguments
            .get(0)
            .ok_or_else(|| {
                RenderError::from("Arity error for `with`, argument expected")
            })?;

        //let node = template.unwrap();
        //node.goo();

        if let Some(node) = template.take() {
            //node.goo();

            let block = rc.push_scope();
            //block.set_base_value(scope.clone());
            //node.goo();
            //rc.render(node)?;
            rc.pop_scope();
        }

        Ok(None)
    }
}

pub(crate) struct EachHelper;

impl Helper for EachHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        arguments: &mut Vec<&Value>,
        hash: &mut HashMap<String, &'source Value>,
        template: &mut Option<&'source Node<'source>>,
    ) -> Result {
        Ok(None)
    }
}

pub(crate) struct IfHelper;

impl Helper for IfHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        arguments: &mut Vec<&Value>,
        hash: &mut HashMap<String, &'source Value>,
        template: &mut Option<&'source Node<'source>>,
    ) -> Result {
        Ok(None)
    }
}

pub(crate) struct UnlessHelper;

impl Helper for UnlessHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        arguments: &mut Vec<&Value>,
        hash: &mut HashMap<String, &'source Value>,
        template: &mut Option<&'source Node<'source>>,
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
        arguments: &mut Vec<&Value>,
        hash: &mut HashMap<String, &'source Value>,
        template: &mut Option<&'source Node<'source>>,
    ) -> Result {

        let target = arguments
            .get(0)
            .ok_or_else(|| {
                RenderError::from("Arity error for `json`, argument expected")
            })?;

        let compact = rc.is_truthy(arguments
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
