//! Scopes are used by helpers to define local variables.
use crate::parser::ast::Node;
use serde_json::{Map, Value};

#[derive(Debug)]
pub struct Scope<'scope> {
    value: Option<Value>,
    locals: Value,
    partial_block: Option<&'scope Node<'scope>>,
}

impl<'scope> Scope<'scope> {
    pub fn new() -> Self {
        Self {
            locals: Value::Object(Map::new()),
            value: None,
            partial_block: None,
        }
    }

    pub fn new_locals(locals: Map<String, Value>) -> Self {
        Self {
            locals: Value::Object(locals),
            value: None,
            partial_block: None,
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

    pub fn set_partial_block(&mut self, block: Option<&'scope Node<'scope>>) {
        self.partial_block = block;
    }

    pub fn partial_block_mut(&mut self) -> &mut Option<&'scope Node<'scope>> {
        &mut self.partial_block
    }
}
