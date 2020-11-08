//! Context information for the call to a helper.
use std::ops::Range;
use serde_json::{Map, Value};

use crate::{parser::ast::Call, helper::BlockResult, error::HelperError};

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

    pub fn arity(&self, range: Range<usize>) -> BlockResult {
        if range.start == range.end {
            if self.arguments().len() != range.start {
                return Err(HelperError::ArityExact(
                    self.name.clone(),
                    range.start,
                ));
            }
        } else {
            if self.arguments().len() < range.start
                || self.arguments().len() > range.end
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
}
