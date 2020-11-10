//! Scopes are used by helpers to define local variables.
use serde_json::{Map, Value};

#[derive(Debug)]
pub struct Scope<'scope> {
    value: Option<Value>,
    locals: Value,
    phantom: std::marker::PhantomData<&'scope str>,
}

impl<'scope> Scope<'scope> {
    pub fn new() -> Self {
        Self {
            locals: Value::Object(Map::new()),
            value: None,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn new_locals(locals: Map<String, Value>) -> Self {
        Self {
            locals: Value::Object(locals),
            value: None,
            phantom: std::marker::PhantomData,
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
