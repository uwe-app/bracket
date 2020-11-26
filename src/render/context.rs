//! Context information for the call to a helper.
use std::ops::Range;

use serde_json::{Map, Value};

use crate::{
    error::HelperError,
    helper::HelperResult,
    json,
    parser::ast::{Call, Node, Slice},
    render::assert::{assert, Type},
};

/// Represents a value to use when a variable lookup fails.
///
/// The underlying value is guaranteed to be `Value::String` and
/// encapsulates the raw value which can be either a path or sub-expression.
#[derive(Debug, Eq, PartialEq)]
pub enum MissingValue {
    /// Stores the raw value for a missing argument.
    Argument(usize, Value),
    /// Stores the raw value for a missing parameter.
    Parameter(String, Value),
}

/// Property represents a key/value pair.
///
/// This is used so that `blockHelperMissing` handlers have access
/// to the resolved property.
#[derive(Debug)]
pub struct Property {
    /// The path to the property.
    pub name: String,
    /// The resolved property value.
    pub value: Value,
}

/// Context for the call to a helper exposes immutable access to
/// the arguments and hash parameters.
///
/// It also provides some useful functions for asserting on argument
/// arity and type.
pub struct Context<'call> {
    // TODO: use call to generate context specific errors!
    call: &'call Call<'call>,
    name: String,
    arguments: Vec<Value>,
    parameters: Map<String, Value>,
    text: Option<&'call str>,
    property: Option<Property>,
    missing: Vec<MissingValue>,
}

impl<'call> Context<'call> {
    pub(crate) fn new(
        call: &'call Call<'call>,
        name: String,
        arguments: Vec<Value>,
        parameters: Map<String, Value>,
        text: Option<&'call str>,
        property: Option<Property>,
        missing: Vec<MissingValue>,
    ) -> Self {
        Self {
            call,
            name,
            arguments,
            parameters,
            text,
            property,
            missing,
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
    pub fn param(&self, name: &str) -> Option<&Value> {
        self.parameters.get(name)
    }

    /// Get an argument at an index and use a fallback string
    /// value when the argument is missing.
    pub fn get_fallback(&self, index: usize) -> Option<&Value> {
        let value = self.arguments.get(index);
        if let Some(&Value::Null) = value {
            if let Some(value) = self.missing(index) {
                return Some(value);
            }
        }
        value
    }

    /// Get a hash parameter for the name and use a fallback string
    /// value when the parameter is missing.
    pub fn param_fallback(&self, name: &str) -> Option<&Value> {
        let value = self.parameters.get(name);
        if let Some(&Value::Null) = value {
            if let Some(value) = self.missing_param(name) {
                return Some(value);
            }
        }
        value
    }

    /// Get the value for a missing argument.
    ///
    /// When the value for an argument is missing it is coerced to
    /// `Value::Null`; this function allows a helper to distinguish
    /// between a literal null value and a null resulting from a missing
    /// value.
    pub fn missing(&self, index: usize) -> Option<&Value> {
        for m in self.missing.iter() {
            if let MissingValue::Argument(ref i, ref value) = m {
                if i == &index {
                    return Some(value);
                }
            }
        }
        None
    }

    /// Get the value for a missing parameter.
    ///
    /// When the value for a parameter is missing it is coerced to
    /// `Value::Null`; this function allows a helper to distinguish
    /// between a literal null value and a null resulting from a missing
    /// value.
    pub fn missing_param(&self, name: &str) -> Option<&Value> {
        for m in self.missing.iter() {
            if let MissingValue::Parameter(ref key, ref value) = m {
                if key == name {
                    return Some(value);
                }
            }
        }
        None
    }

    /// Get the call syntax tree element.
    pub fn call(&self) -> &'call Call<'call> {
        self.call
    }

    /// Get the raw string value for an argument at an index.
    pub fn raw(&self, index: usize) -> Option<&str> {
        self.call.arguments().get(index).map(|v| v.as_str())
    }

    /// Get the raw string value for a hash parameter with the given name.
    pub fn raw_param(&self, name: &str) -> Option<&str> {
        self.call.parameters().get(name).map(|v| v.as_str())
    }

    /// Get an argument at an index and assert that the value
    /// is one of the given types.
    ///
    /// If no argument exists at the given index the value is
    /// treated as null and type assertion is performed on the
    /// null value.
    pub fn try_get(
        &self,
        index: usize,
        kinds: &[Type],
    ) -> HelperResult<&Value> {
        let value = self.arguments.get(index).or(Some(&Value::Null)).unwrap();
        // TODO: print ErrorInfo code snippet
        self.assert(value, kinds)?;
        Ok(value)
    }

    /// Get a hash parameter for the name and assert that the value
    /// is one of the given types.
    ///
    /// If no parameter exists for the given name the value is
    /// treated as null and type assertion is performed on the
    /// null value.
    pub fn try_param(
        &self,
        name: &str,
        kinds: &[Type],
    ) -> HelperResult<&Value> {
        let value = self.parameters.get(name).or(Some(&Value::Null)).unwrap();
        // TODO: print ErrorInfo code snippet
        self.assert(value, kinds)?;
        Ok(value)
    }

    /// Get the text for this context.
    ///
    /// Only available when invoked as a raw block.
    pub fn text(&self) -> &Option<&'call str> {
        &self.text
    }

    /// Get a resolved property.
    ///
    /// Only available to `blockHelperMissing` handlers.
    pub fn property(&self) -> &Option<Property> {
        &self.property
    }

    /// Assert that the call arguments have a valid arity.
    ///
    /// If the range start and end are equal than an exact number
    /// of arguments are expected and a more concise error message
    /// is used. Range ends are exclusive so 1..1 and 1..2 are the
    /// same test they will just generate different error messages.
    pub fn arity(&self, range: Range<usize>) -> HelperResult<()> {
        if range.start == range.end {
            if self.arguments.len() != range.start {
                return Err(HelperError::ArityExact(
                    self.name.clone(),
                    range.start,
                ));
            }
        } else {
            if self.arguments.len() < range.start
                || self.arguments.len() >= range.end
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

    /// Assert on the type of a value.
    pub fn assert(&self, value: &Value, kinds: &[Type]) -> HelperResult<()> {
        let (result, kind) = assert(value, kinds);
        if !result {
            return Err(HelperError::TypeAssert(
                self.name().to_string(),
                kind.unwrap(),
                Type::from(value).to_string(),
            ));
        }
        Ok(())
    }

    /// Map an optional template to a result.
    ///
    /// If the template is `None` this will yield an error; use this
    /// to assert when an inner block template is required.
    pub fn assert_block<'a>(
        &self,
        template: Option<&'a Node<'a>>,
    ) -> HelperResult<&'a Node<'a>> {
        if let Some(node) = template {
            return Ok(node);
        }
        Err(HelperError::BlockTemplate(self.name().to_string()))
    }

    /// Lookup a field of a value.
    ///
    /// If the target value is not an object or array then this
    /// will yield `None`.
    pub fn lookup<'a, S: AsRef<str>>(
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
