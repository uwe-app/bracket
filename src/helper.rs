//! Helper trait and types for the default set of helpers.

use std::collections::HashMap;
use serde_json::{Value, to_string, to_string_pretty};

use crate::{
    output::Output,
    error::RenderError,
    render::{Render, Context},
    parser::ast::Node,
};

/// The result that helper functions should return.
pub type Result = std::result::Result<Option<Value>, RenderError>;

pub type HelperOutput<'render> = std::boxed::Box<&'render mut (dyn Output + 'render)>;

//pub type HelperArguments<'a> = &'a mut Vec<&'a Value>;

/// Trait for helpers.
pub trait Helper: Send + Sync {
    fn call<'reg, 'source, 'render>(
        &self,
        ctx: &Context<'source>,
        out: &mut HelperOutput<'render>,
    ) -> Result;
}

/// Trait for block helpers.
pub trait BlockHelper: Send + Sync {
    fn call<'reg, 'source, 'render>(
        &self,
        ctx: &Context<'source>,
        out: &mut HelperOutput<'render>,
    ) -> Result;
}

//pub(crate) struct LookupHelper;

//impl Helper for LookupHelper {
    //fn call<'reg, 'source, 'render>(
        //&self,
        //rc: &mut Render<'reg, 'source, 'render>,
        //arguments: &mut Vec<&Value>,
        //hash: &mut HashMap<String, &'source Value>,
        //template: &'source Node<'source>,
    //) -> Result {
        //Ok(None)
    //}
//}

pub(crate) struct WithHelper;

impl BlockHelper for WithHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        ctx: &Context<'source>,
        out: &mut HelperOutput<'render>,
        //rc: &mut Render<'reg, 'source, 'render>,
        //arguments: &mut Vec<&Value>,
        //hash: &mut HashMap<String, &'source Value>,
        //template: &mut dyn FnMut() -> std::result::Result<(), RenderError>,
        //template: &'source Node<'source>,
    ) -> Result {

        let scope = ctx.arguments()
            .get(0)
            .ok_or_else(|| {
                RenderError::from("Arity error for `with`, argument expected")
            })?;


        //template()?;

        //let node = template.unwrap();
        //node.goo();

        //if let Some(ref node) = template {
            //node.goo();

            //let block = rc.push_scope();
            //block.set_base_value(scope.clone());
            //node.goo();
            //rc.render(template)?;
            //rc.pop_scope();
        //}

        Ok(None)
    }
}

/*
pub(crate) struct EachHelper;

impl Helper for EachHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        arguments: &mut Vec<&Value>,
        hash: &mut HashMap<String, &'source Value>,
        template: &'source Node<'source>,
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
        template: &'source Node<'source>,
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
        template: &'source Node<'source>,
    ) -> Result {
        Ok(None)
    }
}
*/

// Extended, non-standard helpers

pub(crate) struct JsonHelper;

impl Helper for JsonHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        ctx: &Context<'source>,
        out: &mut HelperOutput<'render>,
    ) -> Result {

        let target = ctx
            .arguments()
            .get(0)
            .ok_or_else(|| {
                RenderError::from("Arity error for `json`, argument expected")
            })?;

        let compact = ctx.is_truthy(
            ctx
            .arguments()
            .get(0)
            .unwrap_or(&&Value::Bool(false))
        );

        println!("JSON HELPER WAS CALLED");

        if compact {
            if let Ok(s) = to_string(target) {
                out.write(s.as_bytes())?;
            }
        } else {
            if let Ok(s) = to_string_pretty(target) {
                out.write(s.as_bytes())?;
            }
        }

        Ok(None)
    }
}
