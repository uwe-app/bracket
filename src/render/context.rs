//! Context information for the call to a helper.
use serde_json::{Map, Value};

/// Context for the call to a helper.
pub struct Context<'reg, 'source, 'render, 'call> {
    name: String,
    arguments: Vec<Value>,
    hash: Map<String, Value>,

    reg: std::marker::PhantomData<&'reg str>,
    source: std::marker::PhantomData<&'source str>,
    render: std::marker::PhantomData<&'render str>,
    call: std::marker::PhantomData<&'call str>,
}

impl<'reg, 'source, 'render, 'call> Context<'reg, 'source, 'render, 'call> {
    pub fn new(
        name: String,
        arguments: Vec<Value>,
        hash: Map<String, Value>,
    ) -> Self {
        Self {
            name,
            arguments,
            hash,

            reg: std::marker::PhantomData,
            source: std::marker::PhantomData,
            render: std::marker::PhantomData,
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
