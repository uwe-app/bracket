use serde_json::{Map, Value};

/// Context for the call to a helper.
pub struct Context<'ctx> {
    name: String,
    arguments: Vec<Value>,
    hash: Map<String, Value>,
    phantom: std::marker::PhantomData<&'ctx str>,
}

impl<'ctx> Context<'ctx> {
    pub fn new(
        name: String,
        arguments: Vec<Value>,
        hash: Map<String, Value>,
    ) -> Self {
        Self {
            name,
            arguments,
            hash,
            phantom: std::marker::PhantomData,
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

