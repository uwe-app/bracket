//! Utility functions for type assertions.
use std::fmt;
use serde_json::Value;

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

/// Assert on the type of a value.
///
/// The type of the value must be one of the given types.
///
/// If the type assertion fails the returned value contains a string 
/// of the current type that caused the failure.
pub fn assert(value: &Value, kinds: &[Type]) -> (bool, Option<String>) {
    for kind in kinds {
        if !assert_type(value, kind) {
            return (false, Some(kind.to_string()))
        }
    }
    (true, None)
}

fn assert_type(value: &Value, kind: &Type) -> bool {
    match value {
        Value::Null => kind == &Type::Null,
        Value::Bool(_) => kind == &Type::Bool,
        Value::String(_) => kind == &Type::String,
        Value::Number(_) => kind == &Type::Number,
        Value::Object(_) => kind == &Type::Object,
        Value::Array(_) => kind == &Type::Array,
    }
}
