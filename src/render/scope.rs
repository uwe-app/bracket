//! Scopes define the evaluation context for variable paths.
use serde_json::{Map, Value};

/// A scope encapsulates a base value (lookup object) used when
/// resolving variable paths and a collection of local variables
/// which are prefixed using the `@` symbol.
///
/// Helpers can create scopes and push and pop them from the scope
/// stack to create new variable evaluation contexts.
#[derive(Debug, Clone)]
pub struct Scope {
    value: Option<Value>,
    locals: Value,
}

impl Scope {
    /// Create a new scope.
    pub fn new() -> Self {
        Self {
            locals: Value::Object(Map::new()),
            value: None,
        }
    }

    /// Get the underlying locals value.
    pub fn locals(&self) -> &Value {
        &self.locals
    }

    /// Set a named local variable.
    ///
    /// The name does not need an `@` prefix it is automatically
    /// prepended to the key.
    pub fn set_local(&mut self, name: &str, value: Value) {
        self.locals
            .as_object_mut()
            .unwrap()
            .insert(format!("@{}", name), value);
    }

    /// Get a named local.
    ///
    /// Locals should have the `@` prefix.
    pub fn local(&self, name: &str) -> Option<&Value> {
        self.locals.as_object().unwrap().get(name)
    }

    /// Set the base value for the scope.
    ///
    /// When the renderer resolves variables if they
    /// can be resolved using this value they are preferred
    /// over the root object.
    pub fn set_base_value(&mut self, value: Value) {
        self.value = Some(value);
    }

    /// Get the base value for this scope.
    pub fn base_value(&self) -> &Option<Value> {
        &self.value
    }
}

/// Create a Scope from a locals map.
impl From<Map<String, Value>> for Scope {
    fn from(map: Map<String, Value>) -> Self {
        let mut scope = Scope::new();
        scope.locals = Value::Object(map);
        scope
    }
}
