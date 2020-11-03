//! Helper trait and types for the default set of helpers.
use serde_json::{to_string, to_string_pretty, Map, Number, Value};
use std::collections::HashMap;
use std::ops::Range;

use crate::{
    error::HelperError as Error,
    parser::ast::Node,
    render::{Render, Scope},
};

#[cfg(feature = "log-helper")]
use crate::log::LogHelper;

pub static FIRST: &str = "first";
pub static LAST: &str = "last";
pub static KEY: &str = "key";
pub static INDEX: &str = "index";

/// The result that helper functions should return.
pub type ValueResult = std::result::Result<Option<Value>, Error>;
pub type Result = std::result::Result<(), Error>;

#[derive(Debug)]
pub struct BlockTemplate<'source> {
    template: &'source Node<'source>,
    inverse: Option<&'source Node<'source>>,
    // TODO: chained if else blocks - when to invoke the Call?
}

impl<'source> BlockTemplate<'source> {
    pub fn new(
        template: &'source Node<'source>,
        inverse: Option<&'source Node<'source>>,
    ) -> Self {
        Self { template, inverse }
    }

    pub fn template(&self) -> &'source Node<'source> {
        self.template
    }

    pub fn inverse(&self) -> &Option<&'source Node<'source>> {
        &self.inverse
    }
}

/// Context for the call to a helper.
pub struct Context<'source> {
    name: &'source str,
    arguments: Vec<Value>,
    hash: Map<String, Value>,
}

impl<'source> Context<'source> {
    pub fn new(
        name: &'source str,
        arguments: Vec<Value>,
        hash: Map<String, Value>,
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

    pub fn into_arguments(self) -> Vec<Value> {
        self.arguments
    }

    pub fn hash(&self) -> &Map<String, Value> {
        &self.hash
    }

    pub fn into_hash(self) -> Map<String, Value> {
        self.hash
    }

    pub fn into(self) -> (String, Vec<Value>, Map<String, Value>) {
        (self.name.to_string(), self.arguments, self.hash)
    }

    pub fn assert_arity(&self, range: Range<usize>) -> Result {
        if range.start == range.end {
            if self.arguments.len() != range.start {
                return Err(Error::ArityExact(
                    self.name().to_owned(),
                    range.start,
                ));
            }
        } else {
            if self.arguments.len() < range.start
                || self.arguments.len() > range.end
            {
                return Err(Error::ArityRange(
                    self.name().to_owned(),
                    range.start,
                    range.end,
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
        ctx: Context<'source>,
    ) -> ValueResult;
}

/// Trait for block helpers.
pub trait BlockHelper: Send + Sync {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
        block: BlockTemplate<'source>,
        //template: &'source Node<'source>,
    ) -> Result;
}

pub(crate) struct WithHelper;

impl BlockHelper for WithHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
        //template: &'source Node<'source>,
        block: BlockTemplate<'source>,
    ) -> Result {
        ctx.assert_arity(1..1)?;

        let mut args = ctx.into_arguments();
        let target = args.swap_remove(0);
        rc.push_scope(Scope::new());
        if let Some(ref mut scope) = rc.scope_mut() {
            scope.set_base_value(target);
        }
        rc.template(block.template())?;
        rc.pop_scope();
        Ok(())
    }
}

pub(crate) struct EachHelper;

impl BlockHelper for EachHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
        //template: &'source Node<'source>,
        block: BlockTemplate<'source>,
    ) -> Result {
        ctx.assert_arity(1..1)?;

        let name = ctx.name().to_string();
        let mut args = ctx.into_arguments();
        let target = args.swap_remove(0);

        rc.push_scope(Scope::new());
        match target {
            Value::Object(t) => {
                let mut it = t.into_iter().enumerate();
                let mut next_value = it.next();
                while let Some((index, (key, value))) = next_value {
                    next_value = it.next();
                    if let Some(ref mut scope) = rc.scope_mut() {
                        scope.set_local(FIRST, Value::Bool(index == 0));
                        scope
                            .set_local(LAST, Value::Bool(next_value.is_none()));
                        scope.set_local(
                            INDEX,
                            Value::Number(Number::from(index)),
                        );
                        scope.set_local(KEY, Value::String(key.to_owned()));
                        scope.set_base_value(value);
                    }
                    rc.template(block.template())?;
                }
            }
            Value::Array(t) => {
                let len = t.len();
                for (index, value) in t.into_iter().enumerate() {
                    if let Some(ref mut scope) = rc.scope_mut() {
                        scope.set_local(FIRST, Value::Bool(index == 0));
                        scope.set_local(LAST, Value::Bool(index == len - 1));
                        scope.set_local(
                            INDEX,
                            Value::Number(Number::from(index)),
                        );
                        scope.set_base_value(value);
                    }
                    rc.template(block.template())?;
                }
            }
            _ => return Err(Error::IterableExpected(name, 1)),
        }
        rc.pop_scope();

        Ok(())
    }
}

pub(crate) struct IfHelper;

impl BlockHelper for IfHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
        //template: &'source Node<'source>,
        block: BlockTemplate<'source>,
    ) -> Result {
        ctx.assert_arity(1..1)?;

        if rc.is_truthy(ctx.arguments().get(0).unwrap()) {
            rc.template(block.template())?;
        }
        // TODO: inverse and chained statements!
        Ok(())
    }
}

/*
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

pub(crate) struct LookupHelper;

impl Helper for LookupHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
    ) -> ValueResult {
        ctx.assert_arity(2..2)?;

        let name = ctx.name();
        let mut args = ctx.into_arguments();
        let target = args.swap_remove(0);

        let field = args
            .get(0)
            .ok_or_else(|| Error::ArityExact(name.to_string(), 2))?
            .as_str()
            .ok_or_else(|| Error::ArgumentTypeString(name.to_string(), 1))?;

        let result = rc.field(&target, field).cloned();

        Ok(result)
    }
}

// Extended, non-standard helpers
#[cfg(feature = "json-helper")]
pub(crate) struct JsonHelper;

impl Helper for JsonHelper {
    fn call<'reg, 'source, 'render>(
        &self,
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: Context<'source>,
    ) -> ValueResult {
        ctx.assert_arity(1..2)?;

        let mut args = ctx.into_arguments();
        let target = args.swap_remove(0);

        let compact = rc.is_truthy(args.get(0).unwrap_or(&Value::Bool(false)));

        if compact {
            if let Ok(s) = to_string(&target) {
                rc.out().write(s.as_bytes()).map_err(Error::from)?;
            }
        } else {
            if let Ok(s) = to_string_pretty(&target) {
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
        #[cfg(feature = "log-helper")]
        self.register_helper("log", Box::new(LogHelper {}));

        self.register_helper("lookup", Box::new(LookupHelper {}));

        self.register_block_helper("with", Box::new(WithHelper {}));
        self.register_block_helper("each", Box::new(EachHelper {}));
        self.register_block_helper("if", Box::new(IfHelper {}));
        //self.register_block_helper("unless", Box::new(UnlessHelper {}));

        #[cfg(feature = "json-helper")]
        self.register_helper("json", Box::new(JsonHelper {}));
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
