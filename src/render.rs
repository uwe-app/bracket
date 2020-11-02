//! Render a template to output using the data.
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::{
    error::{HelperError, RenderError},
    escape::EscapeFn,
    helper::{BlockHelper, Context, Helper, HelperRegistry, Result as HelperResult},
    json,
    output::Output,
    parser::ast::{Block, Call, CallTarget, Node, ParameterValue, Path},
    template::{Template, Templates},
    RenderResult,
};

type HelperValue = Option<Value>;

#[derive(Debug)]
pub struct Scope {
    value: Option<Value>,
    locals: HashMap<String, Value>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            value: None,
        }
    }

    pub fn new_locals(locals: HashMap<String, Value>) -> Self {
        Self {
            locals,
            value: None,
        }
    }

    pub fn set_local(&mut self, name: &str, value: Value) {
        self.locals.insert(format!("@{}", name), value);
    }

    pub fn local(&self, name: &str) -> Option<&Value> {
        self.locals.get(name)
    }

    pub fn set_base_value(&mut self, value: Value) {
        self.value = Some(value);
    }

    pub fn base_value(&self) -> &Option<Value> {
        &self.value
    }
}

pub struct Render<'reg, 'source, 'render> {
    escape: &'reg EscapeFn,
    helpers: &'reg HelperRegistry<'reg>,
    templates: &'source Templates<'source>,
    source: &'source str,
    root: Value,
    writer: Box<&'render mut dyn Output>,
    scopes: Vec<Scope>,
    trim_start: bool,
    trim_end: bool,
    prev_node: Option<&'source Node<'source>>,
    next_node: Option<&'source Node<'source>>,
}

impl<'reg, 'source, 'render> Render<'reg, 'source, 'render> {
    pub fn new<T>(
        escape: &'reg EscapeFn,
        helpers: &'reg HelperRegistry<'reg>,
        templates: &'source Templates<'source>,
        source: &'source str,
        data: &T,
        writer: Box<&'render mut dyn Output>,
    ) -> Result<Self, RenderError<'source>>
    where
        T: Serialize,
    {
        let root = serde_json::to_value(data).map_err(RenderError::from)?;
        let scopes: Vec<Scope> = Vec::new();

        Ok(Self {
            escape,
            helpers,
            templates,
            source,
            root,
            writer,
            scopes,
            trim_start: false,
            trim_end: false,
            prev_node: None,
            next_node: None,
        })
    }

    pub fn out(&mut self) -> &mut Box<&'render mut dyn Output> {
        &mut self.writer
    }

    fn write_str(
        &mut self,
        s: &str,
        escape: bool,
    ) -> Result<usize, RenderError<'source>> {
        let val = if self.trim_start { s.trim_start() } else { s };
        let val = if self.trim_end { val.trim_end() } else { val };
        if val.is_empty() {
            return Ok(0);
        }

        if escape {
            let escaped = (self.escape)(val);
            Ok(self.writer.write_str(&escaped).map_err(RenderError::from)?)
        } else {
            Ok(self.writer.write_str(val).map_err(RenderError::from)?)
        }
    }

    pub fn push_scope(&mut self) -> &mut Scope {
        let scope = Scope::new();
        self.scopes.push(scope);
        self.scopes.last_mut().unwrap()
    }

    pub fn pop_scope(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    pub fn scope_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.last_mut()
    }

    pub fn root(&self) -> &Value {
        &self.root
    }

    pub fn scopes(&mut self) -> &mut Vec<Scope> {
        &mut self.scopes
    }

    /// Infallible path lookup.
    fn lookup(&'source self, path: &'source Path) -> Option<&'source Value> {
        //println!("Lookup path {:?}", path.as_str());

        let root: &'source Value = &self.root;
        let scopes: &'source Vec<Scope> = &self.scopes;

        // Handle explicit `@root` reference
        if path.is_root() {
            let parts = path
                .components()
                .iter()
                .skip(1)
                .map(|c| c.as_str())
                .collect();
            return json::find_parts(parts, root);
        // Handle explicit this only
        } else if path.is_explicit() && path.components().len() == 1 {
            let this = if let Some(scope) = scopes.last() {
                if let Some(base) = scope.base_value() {
                    base
                } else {
                    root
                }
            } else {
                root
            };
            return Some(this);
        } else if path.is_simple() {
            let name = path.as_str();
            // Lookup in the current scope
            if let Some(scope) = scopes.last() {
                if let Some(val) = scope.local(name) {
                    return Some(val);
                }
            // Lookup in the root scope
            } else {
                let parts =
                    path.components().iter().map(|c| c.as_str()).collect();
                return json::find_parts(parts, root);
            }
        }
        None
    }

    fn arguments(&mut self, call: &'source Call<'source>) -> Vec<Value> {
        call.arguments()
            .iter()
            .map(|p| {
                match p {
                    ParameterValue::Json(val) => val.clone(),
                    ParameterValue::Path(ref path) => {
                        self.lookup(path)
                            .map(|v| v.clone())
                            .unwrap_or(Value::Null)
                    }
                    _ => {
                        // TODO: evaluate sub-expressions
                        panic!("Evaluate sub expression in argument");
                        //Value::Null
                    }
                }
            })
            .collect()
    }

    fn hash(call: &'source Call<'source>) -> HashMap<String, Value> {
        call.hash()
            .iter()
            .map(|(k, p)| {
                match p {
                    ParameterValue::Json(val) => (k.to_string(), val.clone()),
                    _ => {
                        // TODO: evaluate sub-expressions
                        (k.to_string(), Value::Null)
                    }
                }
            })
            .collect::<HashMap<_, _>>()
    }

    fn invoke_helper(
        &mut self,
        name: &'source str,
        call: &'source Call<'source>,
    ) -> RenderResult<'source, HelperValue> {

        if let Some(helper) = self.helpers.get(name) {
            let args = self.arguments(call);
            let hash = Render::hash(call);
            let context = Context::new(name, args, hash);
            // FIXME: return the result from invoking the helper
            return Ok(helper.call(self, &context)?)
        }

        Ok(None)
    }

    fn invoke_block_helper(
        &mut self,
        name: &'source str,
        call: &'source Call<'source>,
        template: &'source Node<'source>,
    ) -> RenderResult<'source, ()> {
        if let Some(helper) =
            self.helpers.get_block(name)
        {
            let args = self.arguments(call);
            let hash = Render::hash(call);
            let context = Context::new(name, args, hash);
            helper.call(self, &context, template)?;
        }
        Ok(())
    }

    fn render_partial<'a>(
        rc: &'a mut Render<'reg, 'source, 'render>,
        call: &'source Call<'source>,
        name: String,
    ) -> RenderResult<'source, ()> {
        let template = rc
            .templates
            .get(&name)
            .ok_or_else(|| RenderError::PartialNotFound(name.clone()))?;

        let node: &'source Node<'_> = template.node();
        let hash = Render::hash(call);
        let scope = Scope::new_locals(hash);
        rc.scopes.push(scope);
        rc.render_node(node)?;
        rc.scopes.pop();

        Ok(())
    }

    fn get_partial_name(&self, call: &'source Call<'source>) -> Option<String> {
        match call.target() {
            CallTarget::Path(ref path) => {
                if path.is_simple() {
                    return Some(path.as_str().to_owned());
                } else {
                    panic!("Partials must be simple identifiers");
                }
            }
            _ => todo!("Handle sub expressions"),
        }
        None
    }

    fn statement(
        &mut self,
        call: &'source Call<'source>,
    ) -> Result<HelperValue, RenderError<'source>> {

        //println!("Statement {:?}", call.as_str());

        if call.is_partial() {
            let name = self.get_partial_name(call).ok_or_else(|| {
                RenderError::PartialNameResolve(call.as_str())
            })?;
            Render::render_partial(self, call, name)?;
        } else {
            match call.target() {
                CallTarget::Path(ref path) => {
                    // Explicit paths should resolve to a lookup
                    if path.is_explicit() {
                        return Ok(self.lookup(path).cloned());
                    // Simple paths may be helpers
                    } else if path.is_simple() {
                        if let Some(_) = self.helpers.get(path.as_str()) {
                            return self.invoke_helper(path.as_str(), call)
                        } else {
                            if let Some(value) = self.lookup(path).cloned().take() {
                                return Ok(Some(value));
                            } else {
                                panic!("Missing variable with path {:?}", path);
                            }
                            // TODO: helper does not exist so try to resolve a variable
                            // TODO: otherwise fallback to missing variable handling
                        }
                    } else {
                        return Ok(self.lookup(path).cloned());
                    }
                }
                _ => todo!("Handle sub expressions"),
            }
        }
        Ok(None)
    }

    fn block(
        &mut self,
        node: &'source Node<'source>,
        block: &'source Block<'source>,
    ) -> Result<(), RenderError<'source>> {
        let call = block.call();

        if call.is_partial() {
            // TODO: support passing block to the partial
            // TODO: as @partial-block
            println!("Got partial call for block!");
        } else {
            println!("Call the block...");
            //println!("Evaluating a call {:?}", call);
            match call.target() {
                CallTarget::Path(ref path) => {
                    if path.is_simple() {
                        self.invoke_block_helper(path.as_str(), call, node)?;
                        //if let Some(helper) =
                            //self.helpers.get_block(path.as_str())
                        //{
                            //println!(
                                //"Found a helper for the block path: {}",
                                //path.as_str()
                            //);

                            //let args = self.arguments(call);
                            //let hash = Render::hash(call);
                            //let context =
                                //Context::new(path.as_str(), args, hash);

                            //self.invoke_block_helper(
                                //&context, helper, node,
                            //)?;
                        //}
                    }
                }
                _ => todo!("Handle sub expressions"),
            }
        }
        Ok(())
    }

    /// Render an inner template.
    ///
    /// Block helpers should call this when they want to render an inner template.
    pub fn template(
        &mut self,
        node: &'source Node<'source>,
    ) -> Result<(), HelperError> {
        match node {
            Node::Block(ref block) => {
                for node in block.nodes().iter() {
                    self.render_helper(node)?;
                }
            }
            _ => return self.render_helper(node),
        }
        Ok(())
    }

    /// Render and return a helper result wrapping the underlying render error.
    pub(crate) fn render_helper(
        &mut self,
        node: &'source Node<'source>,
    ) -> HelperResult {
        self.render_node(node)
            .map_err(|e| HelperError::Render(format!("{:?}", e)))
    }

    pub(crate) fn render_node(
        &mut self,
        node: &'source Node<'source>,
    ) -> RenderResult<'source, ()> {
        self.trim_start = if let Some(node) = self.prev_node {
            node.trim_after()
        } else {
            false
        };

        self.trim_end = if let Some(node) = self.next_node {
            node.trim_before()
        } else {
            false
        };

        //let trim_after = node.trim_after();
        //println!("Has trim before {}", trim_before);
        //println!("Has trim after {}", trim_after);

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
                let mut it = doc.nodes().iter();
                self.next_node = it.next();
                while let Some(node) = self.next_node {
                    self.next_node = it.next();
                    self.render_node(node)?;
                }
            }
            Node::Statement(ref call) => {
                if let Some(ref value) = self.statement(call)? {
                    let val = json::stringify(value)?;
                    //println!("Got a json string result {}", val);
                    self.write_str(&val, call.is_escaped())?;
                }
                //match result {
                    //HelperValue::Json(maybe_json) => {
                        ////println!("Got maybe json {:?}", maybe_json);
                        //if let Some(value) = maybe_json {
                            //let val = json::stringify(value)?;
                            ////println!("Got a json string result {}", val);
                            //self.write_str(&val, call.is_escaped())?;
                        //} else {
                            ////todo!("Error on missing varaible.");
                        //}
                    //}
                //}
            }
            Node::Block(ref block) => {
                println!("got block to render...");
                self.block(node, block)?;
                // TODO: call partial / helper for blocks
                //for node in block.nodes().iter() {
                //self.render(node)?;
                //}
            }
            _ => todo!("Render other node types"),
        }

        self.prev_node = Some(node);

        Ok(())
    }
}
