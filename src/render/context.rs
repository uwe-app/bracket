//! Context information for the call to a helper.
use std::fmt;
use std::ops::Range;

use serde_json::{Map, Value};

use crate::{
    error::HelperError, helper::HelperResult, json, parser::ast::Call,
};

/// JSON types used for type assertions.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Type {
    /// The `null` JSON type.
    Null,
    /// The `boolean` JSON type.
    Bool,
    /// The `number` JSON type.
    Number,
    /// The `string` JSON type.
    String,
    /// The `object` JSON type.
    Object,
    /// The `array` JSON type.
    Array,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", {
            match *self {
                Self::Null => "null",
                Self::Bool => "boolean",
                Self::Number => "number",
                Self::String => "string",
                Self::Object => "object",
                Self::Array => "array",
            }
        })
    }
}

impl From<&Value> for Type {
    fn from(value: &Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(_) => Self::Bool,
            Value::Number(_) => Self::Number,
            Value::String(_) => Self::String,
            Value::Object(_) => Self::Object,
            Value::Array(_) => Self::Array,
        }
    }
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
    text: Option<&'call str>,
}

impl<'call> Context<'call> {
    pub(crate) fn new(
        call: &'call Call<'call>,
        name: String,
        arguments: Vec<Value>,
        parameters: Map<String, Value>,
        text: Option<&'call str>,
    ) -> Self {
        Self {
            call,
            name,
            arguments,
            parameters,
            text,
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
    pub fn try_hash(&self, name: &str, kinds: &[Type]) -> HelperResult<&Value> {
        let value = self.parameters.get(name).or(Some(&Value::Null)).unwrap();
        // TODO: print ErrorInfo code snippet
        self.assert(value, kinds)?;
        Ok(value)
    }

    /// Get the text for this context.
    ///
    /// Only available for raw block helpers.
    pub fn text(&self) -> &Option<&'call str> {
        &self.text
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
        for kind in kinds {
            if !self.assert_type(value, kind) {
                return Err(HelperError::TypeAssert(
                    self.name().to_string(),
                    kind.to_string(),
                    Type::from(value).to_string(),
                ));
            }
        }

        Ok(())
    }

    fn assert_type(&self, value: &Value, kind: &Type) -> bool {
        match value {
            Value::Null => kind == &Type::Null,
            Value::Bool(_) => kind == &Type::Bool,
            Value::String(_) => kind == &Type::String,
            Value::Number(_) => kind == &Type::Number,
            Value::Object(_) => kind == &Type::Object,
            Value::Array(_) => kind == &Type::Array,
        }
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
