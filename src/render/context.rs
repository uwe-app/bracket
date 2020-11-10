//! Context information for the call to a helper.
use serde_json::{Map, Value};
use std::ops::Range;

use crate::{json, error::HelperError, helper::HelperResult, parser::ast::Call};

/// Enumerate JSON types for type assertions.
pub enum Type {
    Null,
    Bool,
    Number,
    String,
    Object,
    Array,
}

/// Context for the call to a helper exposes immutable access to
/// the arguments and hash parameters for the helper.
///
/// It also provides some useful functions for asserting on argument
/// arity and type.
pub struct Context<'call> {
    // TODO: use call to generate context specific errors!
    call: &'call Call<'call>,

    name: String,
    arguments: Vec<Value>,
    parameters: Map<String, Value>,
}

impl<'call> Context<'call> {
    pub fn new(
        call: &'call Call<'call>,
        name: String,
        arguments: Vec<Value>,
        parameters: Map<String, Value>,
    ) -> Self {
        Self {
            call,
            name,
            arguments,
            parameters,
        }
    }

    /// Get the name for the call.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the list of arguments.
    pub fn arguments(&self) -> &Vec<Value> {
        &self.arguments
    }

    /// Get the map of hash parameters.
    pub fn parameters(&self) -> &Map<String, Value> {
        &self.parameters
    }

    /// Get an argument at an index.
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.arguments.get(index) 
    }

    /// Get a hash parameter for the name.
    pub fn hash(&self, name: &str) -> Option<&Value> {
        self.parameters.get(name) 
    }

    /// Assert that the call arguments have a valid arity.
    ///
    /// If the range start and end are equal than an exact number 
    /// of arguments are expected and a more concise error message 
    /// is used. Range ends are exclusive so 1..1 and 1..2 are the 
    /// same test they will just generate different error messages.
    pub fn arity(&self, range: Range<usize>) -> HelperResult<()> {
        if range.start == range.end {
            if self.arguments().len() != range.start {
                return Err(HelperError::ArityExact(
                    self.name.clone(),
                    range.start,
                ));
            }
        } else {
            if self.arguments().len() < range.start
                || self.arguments().len() >= range.end
            {
                return Err(HelperError::ArityRange(
                    self.name.clone(),
                    range.start,
                    range.end,
                ));
            }
        }
        Ok(())
    }

    /// Lookup a field of a value.
    ///
    /// If the target value is not an object or array then this
    /// will yield `None`.
    pub fn field<'a, S: AsRef<str>>(
        &self,
        target: &'a Value,
        field: S,
    ) -> Option<&'a Value> {
        json::find_field(target, field)
    }

    /// Determine if a value is truthy.
    pub fn is_truthy(&self, value: &Value) -> bool {
        json::is_truthy(value)
    }
}
