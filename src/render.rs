//! Render a template to output using the data.
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::{
    error::{HelperError, RenderError},
    helper::{BlockHelper, Context, Helper},
    json,
    output::Output,
    parser::ast::{Block, Call, CallTarget, Node, ParameterValue, Path},
    registry::{Registry, Templates},
    template::Template,
    RenderResult,
};

#[derive(Debug)]
pub enum HelperValue<'source> {
    Json(Option<&'source Value>),
}

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

    pub fn set_base_value(&mut self, value: Value) {
        self.value = Some(value);
    }

    pub fn base_value(&self) -> &Option<Value> {
        &self.value
    }
}

pub struct Render<'reg, 'source, 'render> {
    source: &'source str,
    registry: &'reg Registry<'reg>,
    templates: &'source Templates<'source>,
    root: Value,
    writer: Box<&'render mut dyn Output>,
    scopes: Vec<Scope>,
    trim_start: bool,
    trim_end: bool,
    prev_node: Option<&'source Node<'source>>,
    next_node: Option<&'source Node<'source>>,
    block_template_node: Option<&'source Node<'source>>,
}

impl<'reg, 'source, 'render> Render<'reg, 'source, 'render> {
    pub fn new<T>(
        source: &'source str,
        registry: &'reg Registry<'reg>,
        templates: &'source Templates<'source>,
        data: &T,
        writer: Box<&'render mut dyn Output>,
    ) -> Result<Self, RenderError<'source>>
    where
        T: Serialize,
    {
        let root = serde_json::to_value(data).map_err(RenderError::from)?;
        let scopes: Vec<Scope> = Vec::new();

        Ok(Self {
            source,
            registry,
            templates,
            root,
            writer,
            scopes,
            trim_start: false,
            trim_end: false,
            prev_node: None,
            next_node: None,
            block_template_node: None,
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
            let handler = self.registry.escape();
            let escaped = handler(val);
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

    fn lookup(
        path: &Path,
        root: &'source Value,
        scopes: &'source Vec<Scope>,
    ) -> Option<&'source Value> {
        println!("Lookup path {:?}", path.as_str());

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
            println!("Got explicit this!!!");
            let this = if let Some(scope) = scopes.last() {
                if let Some(base) = scope.base_value() {
                    println!(
                        "Got explicit this with a scope base value!!! {:?}",
                        base
                    );
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
            if let Some(scope) = scopes.last() {
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

    fn arguments(&mut self, call: &'source Call<'source>) -> Vec<Value> {
        call.arguments()
            .iter()
            .map(|p| {
                match p {
                    ParameterValue::Json(val) => val.clone(),
                    ParameterValue::Path(ref path) => {
                        Render::lookup(path, &self.root, &self.scopes)
                            .map(|v| v.clone())
                            .unwrap_or(Value::Null)
                    }
                    _ => {
                        // TODO: evaluate sub-expressions
                        Value::Null
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
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &Context<'source>,
        helper: &'reg Box<dyn Helper + 'reg>,
    ) -> RenderResult<'source, Option<Value>> {
        helper.call(rc, ctx)?;
        Ok(None)
    }

    fn invoke_block_helper(
        rc: &mut Render<'reg, 'source, 'render>,
        ctx: &Context<'source>,
        helper: &'reg Box<dyn BlockHelper + 'reg>,
        template: &'source Node<'source>,
    ) -> RenderResult<'source, ()> {
        rc.block_template_node = Some(template);
        helper.call(rc, ctx)?;
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
    ) -> Result<HelperValue<'_>, RenderError<'source>> {
        if call.is_partial() {
            println!("GOT PARTIAL CALL FOR STATEMENT!");
            let name = self.get_partial_name(call).ok_or_else(|| {
                RenderError::PartialNameResolve(call.as_str())
            })?;
            Render::render_partial(self, call, name)?;
        } else {
            //println!("Evaluating a call {:?}", call);
            match call.target() {
                CallTarget::Path(ref path) => {
                    if path.is_simple() {
                        if let Some(helper) =
                            self.registry.get_helper(path.as_str())
                        {
                            let args = self.arguments(call);
                            let hash = Render::hash(call);
                            let ctx = Context::new(path.as_str(), args, hash);

                            // FIXME: return the result from invoking the helper
                            Render::invoke_helper(self, &ctx, helper)?;
                        } else {
                            return Ok(HelperValue::Json(Render::lookup(
                                path,
                                &self.root,
                                &self.scopes,
                            )));
                        }
                    } else {
                        return Ok(HelperValue::Json(Render::lookup(
                            path,
                            &self.root,
                            &self.scopes,
                        )));
                    }
                }
                _ => todo!("Handle sub expressions"),
            }
        }
        Ok(HelperValue::Json(None))
    }

    fn block(
        &mut self,
        node: &'source Node<'source>,
        block: &'source Block<'source>,
    ) -> Result<(), RenderError<'source>> {
        println!("Render a block...");
        let call = block.call();

        if call.is_partial() {
            println!("Got partial call for block!");
        } else {
            println!("Call the block...");
            //println!("Evaluating a call {:?}", call);
            match call.target() {
                CallTarget::Path(ref path) => {
                    if path.is_simple() {
                        if let Some(helper) =
                            self.registry.get_block_helper(path.as_str())
                        {
                            println!(
                                "Found a helper for the block path: {}",
                                path.as_str()
                            );

                            let args = self.arguments(call);
                            let hash = Render::hash(call);
                            let context =
                                Context::new(path.as_str(), args, hash);

                            Render::invoke_block_helper(
                                self, &context, helper, node,
                            )?;
                        }
                    }
                }
                _ => todo!("Handle sub expressions"),
            }
        }
        Ok(())
    }

    pub(crate) fn render_inner(&mut self) -> Result<(), HelperError> {
        //println!("RENDER INNER BLOCK");

        if let Some(node) = self.block_template_node {
            match node {
                Node::Block(ref block) => {
                    for node in block.nodes().iter() {
                        self.render_node(node).map_err(|e| {
                            HelperError::Render(format!("{:?}", e))
                        })?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
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
                let result = self.statement(call)?;
                match result {
                    HelperValue::Json(maybe_json) => {
                        //println!("Got maybe json {:?}", maybe_json);
                        if let Some(value) = maybe_json {
                            let val = json::stringify(value)?;
                            //println!("Got a json string result {}", val);
                            self.write_str(&val, call.is_escaped())?;
                        } else {
                            //todo!("Error on missing varaible.");
                        }
                    }
                }
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
