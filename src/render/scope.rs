//! Scopes define the evaluation context for variable paths.
use serde_json::{Map, Value};

#[derive(Debug)]
pub struct Scope {
    value: Option<Value>,
    locals: Value,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            locals: Value::Object(Map::new()),
            value: None,
        }
    }

    pub fn new_locals(locals: Map<String, Value>) -> Self {
        Self {
            locals: Value::Object(locals),
            value: None,
        }
    }

    pub fn as_value(&self) -> &Value {
        &self.locals
    }

    pub fn set_local(&mut self, name: &str, value: Value) {
        self.locals
            .as_object_mut()
            .unwrap()
            .insert(format!("@{}", name), value);
    }

    pub fn local(&self, name: &str) -> Option<&Value> {
        self.locals.as_object().unwrap().get(name)
    }

    pub fn set_base_value(&mut self, value: Value) {
        self.value = Some(value);
    }

    pub fn base_value(&self) -> &Option<Value> {
        &self.value
    }
}
