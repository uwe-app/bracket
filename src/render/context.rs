//! Context information for the call to a helper.
use serde_json::{Map, Value};

/// Context for the call to a helper.
pub struct Context<'call> {
    name: String,
    arguments: Vec<Value>,
    hash: Map<String, Value>,

    call: std::marker::PhantomData<&'call str>,
}

impl<'call> Context<'call> {
    pub fn new(
        name: String,
        arguments: Vec<Value>,
        hash: Map<String, Value>,
    ) -> Self {
        Self {
            name,
            arguments,
            hash,

            call: std::marker::PhantomData,
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
