//! Context information for the call to a helper.
use serde_json::{Map, Value};

use crate::parser::ast::Call;

/// Context for the call to a helper.
pub struct Context<'call> {
    call: &'call Call<'call>,
    name: String,
    arguments: Vec<Value>,
    hash: Map<String, Value>,
}

impl<'call> Context<'call> {
    pub fn new(
        call: &'call Call<'call>,
        name: String,
        arguments: Vec<Value>,
        hash: Map<String, Value>,
    ) -> Self {
        Self {
            call,
            name,
            arguments,
            hash,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> &Vec<Value> {
        &self.arguments
    }

    pub fn hash(&self) -> &Map<String, Value> {
        &self.hash
    }
}
