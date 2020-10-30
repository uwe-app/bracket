use std::collections::HashMap;
use serde::Serialize;
use serde_json::Value;

use crate::{
    error::RenderError,
    output::Output,
    parser::ast::{Call, Node},
    registry::Registry,
};

#[derive(Debug)]
pub enum EvalResult {
    Json(Value),
}

#[derive(Debug)]
pub struct Scope {
    locals: HashMap<String, Value>,
}

impl Scope {

    pub fn new() -> Self {
        Self {locals: HashMap::new()} 
    }

    pub fn set_local(&mut self, name: &str, value: Value) {
        self.locals.insert(format!("@{}", name), value);
    }
}

pub struct Render<'reg, 'render> {
    registry: &'reg Registry<'reg>,
    root: Value,
    writer: Box<&'render mut dyn Output>,
    scopes: Vec<Scope>,
}

impl<'reg, 'render> Render<'reg, 'render> {
    pub fn new<T: Serialize>(
        registry: &'reg Registry<'reg>,
        data: &T,
        writer: Box<&'render mut dyn Output>,
    ) -> Result<Self, RenderError> {
        let root = serde_json::to_value(data).map_err(RenderError::from)?;
        Ok(Self {
            registry,
            root,
            writer,
            scopes: Vec::new(),
        })
    }

    fn write_str(&mut self, s: &str) -> Result<usize, RenderError> {
        Ok(self.writer.write_str(s).map_err(RenderError::from)?)
    }

    pub fn push_scope(&mut self) -> &mut Scope {
        self.scopes.push(Scope::new());
        self.scopes.last_mut().unwrap()
    }

    pub fn pop_scope(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    pub fn evaluate(&mut self, call: &Call) -> EvalResult {
        if call.is_partial() {
            println!("Got partial call");
        } else {
            println!("Evaluating a call...");
        }
        EvalResult::Json(Value::Null)
    }

    pub fn render(&mut self, node: &'render Node<'render>) -> Result<(), RenderError> {

        //println!("rendering node {:?}", node);

        match node {
            Node::Text(ref n) => {
                self.write_str(n.as_str())?;
            }
            Node::RawBlock(ref n) => {
                self.write_str(n.between())?;
            }
            Node::RawStatement(ref n) => {
                let raw = &n.as_str()[1..];
                self.write_str(raw)?;
            }
            Node::RawComment(_) => {}
            Node::Comment(_) => {}
            Node::Document(ref doc) => {
                self.push_scope();
                for node in doc.nodes().iter() {
                    self.render(node)?;
                }
                self.pop_scope();
            }
            Node::Statement(ref call) => {
                println!("TODO: Evaluate statement in render!");
                match self.evaluate(call) {
                    EvalResult::Json(ref value) => {
                        println!("Got json value...");
                    }
                }
                //self.write_str(n.as_str())?;
                //let value = 
            }
            Node::Block(ref block) => {
                // TODO: call partial / helper for blocks
                for node in block.nodes().iter() {
                    self.render(node)?;
                }
            }
            _ => todo!("Render other node types")
        }

        Ok(())
    }
}
