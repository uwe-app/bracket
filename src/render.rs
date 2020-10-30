use std::collections::HashMap;
use std::marker::PhantomData;
use serde::Serialize;
use serde_json::Value;

use crate::{
    error::RenderError,
    output::Output,
    parser::ast::{Call, CallTarget, Node, Path},
    registry::Registry,
};

mod json {
    use serde_json::Value;

    pub fn stringify(value: &Value) -> String {
        match value {
            Value::String(ref s) => {
                s.to_owned()
            }
            _ => todo!("Stringify other json types"),
        } 
    }

    // Look up path parts in an object.
    pub fn find_parts<'a, 'b>(parts: Vec<&'a str>, doc: &'b Value) -> Option<&'b Value> {
        let mut parent = None;
        match doc {
            Value::Object(ref _map) => {
                let mut current: Option<&Value> = Some(doc);
                for (i, part) in parts.iter().enumerate() {
                    if i == parts.len() - 1 {
                        if let Some(target) = current {
                            return find_field(&part, target);
                        }
                    } else {
                        if let Some(target) = current {
                            parent = find_field(part, target);
                            if parent.is_none() {
                                break;
                            }
                            current = parent;
                        } else {
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
        None
    }

    // Look up a field in an array or object.
    pub fn find_field<'b, S: AsRef<str>>(field: S, parent: &'b Value) -> Option<&'b Value> {
        match parent {
            Value::Object(ref map) => {
                if let Some(val) = map.get(field.as_ref()) {
                    return Some(val);
                }
            }
            Value::Array(ref list) => {
                if let Ok(index) = field.as_ref().parse::<usize>() {
                    if !list.is_empty() && index < list.len() {
                        return Some(&list[index]);
                    }
                }
            }
            _ => {}
        }
        None
    }

    pub fn is_truthy(val: &Value) -> bool {
        match val {
            Value::Object(ref _map) => return true,
            Value::Array(ref _list) => return true,
            Value::String(ref s) => return s.len() > 0,
            Value::Bool(ref b) => return *b,
            Value::Number(ref n) => {
                if n.is_i64() {
                    return n.as_i64().unwrap() != 0;
                } else if n.is_u64() {
                    return n.as_u64().unwrap() != 0;
                } else if n.is_f64() {
                    return n.as_f64().unwrap() != 0.0;
                }
            }
            _ => {}
        }
        false
    }
}

#[derive(Debug)]
pub enum EvalResult<'render> {
    Json(Option<&'render Value>),
}

#[derive(Debug)]
pub struct Scope<'scope> {
    locals: HashMap<String, Value>,
    value: Option<&'scope Value>,
    phantom: PhantomData<& 'scope Value>,
}

impl<'scope> Scope<'scope> {
    pub fn new() -> Self {
        Self {locals: HashMap::new(), phantom: PhantomData, value: None} 
    }

    pub fn set_local(&mut self, name: &str, value: Value) {
        self.locals.insert(format!("@{}", name), value);
    }

    pub fn set_base_value(&mut self, value: &'scope Value) {
        self.value = Some(value);
    }

    pub fn base_value(&self) -> &Option<&'scope Value> {
        &self.value
    }
}

pub struct Render<'reg, 'render> {
    registry: &'reg Registry<'reg>,
    root: Value,
    writer: Box<&'render mut dyn Output>,
    scopes: Vec<Scope<'render>>,
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

    fn write_str(&mut self, s: &str, escape: bool) -> Result<usize, RenderError> {
        if escape {
            let handler = self.registry.escape(); 
            let escaped = handler(s);
            Ok(self.writer.write_str(&escaped).map_err(RenderError::from)?)
        } else {
            Ok(self.writer.write_str(s).map_err(RenderError::from)?)
        }

    }

    pub fn push_scope(&mut self, scope: Scope<'render>) -> &mut Scope<'render> {
        self.scopes.push(scope);
        self.scopes.last_mut().unwrap()
    }

    pub fn pop_scope(&mut self) -> Option<Scope<'render>> {
        self.scopes.pop()
    }

    pub fn scope(&self) -> Option<&Scope<'render>> {
        self.scopes.last()
    }

    pub fn scope_mut(&mut self) -> Option<&mut Scope<'render>> {
        self.scopes.last_mut()
    }

    fn resolve(&self, path: &Path, scope: Option<&Scope<'render>>) -> Option<&Value> {
        let root = &self.root;
        if path.is_simple() {
            let name = path.as_str(); 
            println!("Lookup variable ... {}", name);
            if let Some(scope) = scope {
                println!("Look up in current scope...");
            } else {
                println!("Look up in root scope...");

                let parts = path.components()
                    .iter()
                    .map(|c| c.as_str())
                    .collect();

                let value = json::find_parts(parts, root);

                println!("Got value {:?}", value);
                return value
            }
        }
        None 
    }

    pub fn lookup(&self, path: &Path) -> Option<&Value> {
        return self.resolve(path, self.scope());
    }

    pub fn evaluate(&mut self, call: &Call) -> EvalResult {
        if call.is_partial() {
            println!("Got partial call");
        } else {
            println!("Evaluating a call {:?}", call);

            match call.target() {
                CallTarget::Path(ref path) => {
                    if path.is_simple() {
                        println!("Got simple path {}", path.as_str());
                        if let Some(helper) = self.registry.get_helper(path.as_str()) {
                            println!("Found a helper for the simple path!");
                        } else {
                            println!("Evaluate as a variable...");
                            return EvalResult::Json(self.lookup(path))
                        }
                    } else {
                        return EvalResult::Json(self.lookup(path))
                    }

                }
                _ => todo!("Handle sub expressions")
            }

        }
        EvalResult::Json(None)
    }

    pub fn render(&mut self, node: &'render Node<'render>) -> Result<(), RenderError> {
        match node {
            Node::Text(ref n) => {
                self.write_str(n.as_str(), false)?;
            }
            Node::RawBlock(ref n) => {
                self.write_str(n.between(), false)?;
            }
            Node::RawStatement(ref n) => {
                let raw = &n.as_str()[1..];
                self.write_str(raw, false)?;
            }
            Node::RawComment(_) => {}
            Node::Comment(_) => {}
            Node::Document(ref doc) => {
                for node in doc.nodes().iter() {
                    self.render(node)?;
                }
            }
            Node::Statement(ref call) => {
                let result = self.evaluate(call);
                match result {
                    EvalResult::Json(maybe_json) => {
                        println!("Got maybe json {:?}", maybe_json);
                        if let Some(value) = maybe_json {
                            let val = json::stringify(value);
                            println!("Got a json string result {}", val);
                            self.write_str(&val, call.is_escaped())?;
                        } else {
                            //todo!("Error on missing varaible.");
                        }
                    }
                }
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
