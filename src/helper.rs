//! Helper trait and types for the default set of helpers.
use serde_json::{to_string, to_string_pretty, Value};
use std::collections::HashMap;
use std::ops::Range;

use crate::{
    error::HelperError as Error, json, log::LogHelper, render::Render,
};

/// The result that helper functions should return.
pub type Result<'source> = std::result::Result<Option<Value>, Error>;
pub type AssertResult = std::result::Result<(), Error>;

/// Context for the call to a helper.
pub struct Context<'source> {
    name: &'source str,
    arguments: Vec<Value>,
    hash: HashMap<String, Value>,
}

impl<'source> Context<'source> {
    pub fn new(
        name: &'source str,
        arguments: Vec<Value>,
        hash: HashMap<String, Value>,
    ) -> Self {
        Self {
            name,
            arguments,
            hash,
        }
    }

    pub fn name(&self) -> &'source str {
        self.name
    }

    pub fn arguments(&self) -> &Vec<Value> {
        &self.arguments
    }

    pub fn hash(&self) -> &HashMap<String, Value> {
        &self.hash
    }

    pub fn is_truthy(&self, value: &Value) -> bool {
        json::is_truthy(value)
    }

    pub fn assert_arity(&self, range: Range<usize>) -> AssertResult {
        if range.start == range.end {
            println!("Asserting on arity... {}", self.arguments.len());
            if self.arguments.len() != range.start {
                println!("Returning arity error");
                return Err(Error::ArityExact(
                    self.name().to_owned(),
                    range.start,
                ));
            }
        }
        Ok(())
    }
}

/// Trait for helpers.
pub trait Helper: Send + Sync {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &Context<'source>,
    ) -> Result;
}

/// Trait for block helpers.
pub trait BlockHelper: Send + Sync {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &Context<'source>,
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
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &Context<'source>,
    ) -> Result {
        ctx.assert_arity(1..1)?;

        let scope = ctx
            .arguments()
            .get(0)
            .ok_or_else(|| Error::ArityExact(ctx.name().to_string(), 1))?;

        println!("With is setting the scope {:?}", scope);

        let block = rc.push_scope();
        block.set_base_value(scope.clone());
        rc.render_inner()?;
        rc.pop_scope();

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
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &Context<'source>,
    ) -> Result {
        let target = ctx
            .arguments()
            .get(0)
            .ok_or_else(|| Error::ArityExact(ctx.name().to_string(), 1))?;

        let compact = ctx
            .is_truthy(ctx.arguments().get(0).unwrap_or(&&Value::Bool(false)));

        if compact {
            if let Ok(s) = to_string(target) {
                rc.out().write(s.as_bytes()).map_err(Error::from)?;
            }
        } else {
            if let Ok(s) = to_string_pretty(target) {
                rc.out().write(s.as_bytes()).map_err(Error::from)?;
            }
        }

        Ok(None)
    }
}

/// Registry of helpers.
pub struct HelperRegistry<'reg> {
    helpers: HashMap<&'reg str, Box<dyn Helper + 'reg>>,
    block_helpers: HashMap<&'reg str, Box<dyn BlockHelper + 'reg>>,
}

impl<'reg> HelperRegistry<'reg> {
    pub fn new() -> Self {
        let mut reg = Self {
            helpers: Default::default(),
            block_helpers: Default::default(),
        };
        reg.builtins();
        reg
    }

    fn builtins(&mut self) {
        self.register_helper("log", Box::new(LogHelper {}));
        self.register_helper("json", Box::new(JsonHelper {}));
        //self.register_helper("lookup", Box::new(LookupHelper {}));

        self.register_block_helper("with", Box::new(WithHelper {}));
        //self.register_helper("each", Box::new(EachHelper {}));
        //self.register_helper("if", Box::new(IfHelper {}));
        //self.register_helper("unless", Box::new(UnlessHelper {}));
    }

    pub fn register_helper(
        &mut self,
        name: &'reg str,
        helper: Box<dyn Helper + 'reg>,
    ) {
        self.helpers.insert(name, helper);
    }

    pub fn register_block_helper(
        &mut self,
        name: &'reg str,
        helper: Box<dyn BlockHelper + 'reg>,
    ) {
        self.block_helpers.insert(name, helper);
    }

    //pub fn helpers(&self) -> &HashMap<&'reg str, Box<dyn Helper + 'reg>> {
    //&self.helpers
    //}

    pub fn get(&self, name: &str) -> Option<&Box<dyn Helper + 'reg>> {
        self.helpers.get(name)
    }

    pub fn get_block(
        &self,
        name: &str,
    ) -> Option<&Box<dyn BlockHelper + 'reg>> {
        self.block_helpers.get(name)
    }
}
