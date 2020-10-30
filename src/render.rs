use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::{
    error::RenderError,
    helper::Helper,
    json,
    output::Output,
    parser::ast::{Call, CallTarget, Node, ParameterValue, Path},
    registry::Registry,
};

#[derive(Debug)]
pub enum EvalResult<'render> {
    Json(Option<&'render Value>),
}

#[derive(Debug)]
pub struct Scope<'scope> {
    locals: HashMap<String, Value>,
    value: Option<&'scope Value>,
    phantom: PhantomData<&'scope Value>,
}

impl<'scope> Scope<'scope> {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            phantom: PhantomData,
            value: None,
        }
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

#[derive(Debug)]
pub struct LazyValue<'scope> {
    value: &'scope ParameterValue<'scope>,
}

impl<'scope> From<&'scope ParameterValue<'scope>> for LazyValue<'scope> {
    fn from(value: &'scope ParameterValue<'scope>) -> Self {
        Self { value }
    }
}

pub struct Context<'render> {
    arguments: Vec<LazyValue<'render>>,
    hash: HashMap<String, LazyValue<'render>>,
}

impl<'render> Context<'render> {
    pub fn new(
        /* root: &'render Value, */
        call: &'render Call<'render>,
    ) -> Self {
        let arguments = call.arguments().iter().map(LazyValue::from).collect();
        let hash = call
            .hash()
            .iter()
            .map(|(k, v)| (k.to_string(), LazyValue::from(v)))
            .collect::<HashMap<_, _>>();
        Self { arguments, hash }
    }

    pub fn arguments(&self) -> &Vec<LazyValue<'render>> {
        return &self.arguments;
    }
}

//impl<'scope> From<&'scope Call<'scope>> for Context<'scope> {
//fn from(call: &'scope Call<'scope>) -> Self {
//let arguments = call.arguments().iter().map(LazyValue::from).collect();
//let hash = call.hash().iter().map(|(k, v)| {
//(k.to_string(), LazyValue::from(v))
//}).collect::<HashMap<_,_>>();
//Self {
//arguments,
//hash,
//}
//}
//}

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

    fn write_str(
        &mut self,
        s: &str,
        escape: bool,
    ) -> Result<usize, RenderError> {
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

    fn resolve(
        &self,
        path: &Path,
        scope: Option<&Scope<'render>>,
    ) -> Option<&Value> {
        let root = &self.root;

        // Handle explicit `@root` reference
        if path.is_root() {
            let parts = path
                .components()
                .iter()
                .skip(1)
                .map(|c| c.as_str())
                .collect();
            return json::find_parts(parts, root);
        } else if path.is_simple() {
            let name = path.as_str();
            if let Some(scope) = scope {
                //println!("Look up in current scope...");
            } else {
                //println!("Look up in root scope...");
                let parts =
                    path.components().iter().map(|c| c.as_str()).collect();
                return json::find_parts(parts, root);
            }
        }
        None
    }

    pub fn lookup(&self, path: &Path) -> Option<&Value> {
        return self.resolve(path, self.scope());
    }

    pub fn invoke(
        &mut self,
        call: &'render Call,
        name: &str,
        helper: &'reg Box<dyn Helper + 'reg>,
    ) -> Result<Option<Value>, RenderError> {
        let ctx = Context::new(call);
        helper.call(self, &ctx);
        Ok(None)
    }

    pub fn evaluate(
        &mut self,
        call: &'render Call,
    ) -> Result<EvalResult, RenderError> {
        if call.is_partial() {
            //println!("Got partial call");
        } else {
            println!("Evaluating a call {:?}", call);
            match call.target() {
                CallTarget::Path(ref path) => {
                    if path.is_simple() {
                        if let Some(helper) =
                            self.registry.get_helper(path.as_str())
                        {
                            //println!("Found a helper for the simple path!");
                            self.invoke(call, path.as_str(), helper)?;
                        } else {
                            println!("Evaluate as a variable...");
                            return Ok(EvalResult::Json(self.lookup(path)));
                        }
                    } else {
                        return Ok(EvalResult::Json(self.lookup(path)));
                    }
                }
                _ => todo!("Handle sub expressions"),
            }
        }
        Ok(EvalResult::Json(None))
    }

    pub fn render(
        &mut self,
        node: &'render Node<'render>,
    ) -> Result<(), RenderError> {
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
                let result = self.evaluate(call)?;
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
            _ => todo!("Render other node types"),
        }

        Ok(())
    }
}
