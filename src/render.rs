//! Render a template to output using the data.
use serde::Serialize;
use serde_json::{Map, Value};
use std::ops::Range;

use crate::{
    error::{HelperError, RenderError},
    escape::EscapeFn,
    helper::{Assertion, BlockHelper, BlockResult, BlockTemplate, Context, Helper, HelperRegistry},
    json,
    output::Output,
    parser::{
        ast::{Block, Call, CallTarget, Node, ParameterValue, Path},
        trim::{TrimHint, TrimState},
    },
    template::Templates,
    RenderResult,
};

static PARTIAL_BLOCK: &str = "@partial-block";

type HelperValue = Option<Value>;

// Used to determine how to find and invoke helpers.
enum HelperType {
    Value,
    Block,
    Raw,
}

#[derive(Debug)]
pub struct Scope<'scope> {
    value: Option<Value>,
    locals: Value,
    partial_block: Option<&'scope Node<'scope>>,
}

impl<'scope> Scope<'scope> {
    pub fn new() -> Self {
        Self {
            locals: Value::Object(Map::new()),
            value: None,
            partial_block: None,
        }
    }

    pub fn new_locals(locals: Map<String, Value>) -> Self {
        Self {
            locals: Value::Object(locals),
            value: None,
            partial_block: None,
        }
    }

    pub fn as_value(&self) -> &Value {
        &self.locals
    }

    pub fn set_local(&mut self, name: &str, value: Value) {
        self.locals
            .as_object_mut()
            .unwrap()
            .insert(format!("@{}", name), value);
    }

    pub fn local(&self, name: &str) -> Option<&Value> {
        self.locals.as_object().unwrap().get(name)
    }

    pub fn set_base_value(&mut self, value: Value) {
        self.value = Some(value);
    }

    pub fn base_value(&self) -> &Option<Value> {
        &self.value
    }

    pub fn set_partial_block(&mut self, block: Option<&'scope Node<'scope>>) {
        self.partial_block = block;
    }

    pub fn partial_block_mut(&mut self) -> &mut Option<&'scope Node<'scope>> {
        &mut self.partial_block
    }
}

pub struct Render<'reg, 'source, 'render> {
    escape: &'reg EscapeFn,
    helpers: &'reg HelperRegistry<'reg>,
    templates: &'source Templates<'source>,
    source: &'source str,
    root: Value,
    writer: Box<&'render mut dyn Output>,
    scopes: Vec<Scope<'source>>,
    local_helpers: Option<&'render HelperRegistry<'render>>,
    trim: TrimState,
    hint: Option<TrimHint>,
    end_tag_hint: Option<TrimHint>,
}

impl<'reg, 'source, 'render> Render<'reg, 'source, 'render> {
    pub fn new<T>(
        escape: &'reg EscapeFn,
        helpers: &'reg HelperRegistry<'reg>,
        templates: &'source Templates<'source>,
        source: &'source str,
        data: &T,
        writer: Box<&'render mut dyn Output>,
    ) -> RenderResult<Self>
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
            local_helpers: None,
            trim: Default::default(),
            hint: None,
            end_tag_hint: None,
        })
    }

    /// Get a mutable reference to the output writer.
    pub fn out(&mut self) -> &mut Box<&'render mut dyn Output> {
        &mut self.writer
    }

    /// Push a scope onto the stack.
    pub fn push_scope(&mut self, scope: Scope<'source>) {
        self.scopes.push(scope);
    }

    /// Remove a scope from the stack.
    pub fn pop_scope(&mut self) -> Option<Scope<'source>> {
        self.scopes.pop()
    }

    /// Get a mutable reference to the current scope.
    pub fn scope_mut(&mut self) -> Option<&mut Scope<'source>> {
        self.scopes.last_mut()
    }

    /// Reference to the root data for the render.
    pub fn root(&self) -> &Value {
        &self.root
    }

    /// Determine if a value is truthy.
    pub fn is_truthy(&self, value: &Value) -> bool {
        json::is_truthy(value)
    }

    /// Render an inner template.
    ///
    /// Block helpers should call this when they want to render an inner template.
    pub fn template(
        &mut self,
        node: &'source Node<'_>,
    ) -> Result<(), HelperError> {

        let mut hint: Option<TrimHint> = None;
        for event in node.block_iter().trim(self.hint) {
            let mut trim = event.trim;

            if event.first {
                let hint = node.trim();
                if hint.after {
                    trim.start = true;
                }
            }

            if event.last {
                match node {
                    Node::Condition(ref block) => {
                        let last_hint = block.trim_close();
                        if last_hint.before {
                            trim.end = true; 
                        }
                        hint = Some(last_hint);
                    }
                    Node::Block(ref block) => {
                        let last_hint = block.trim_close();
                        if last_hint.before {
                            trim.end = true; 
                        }
                        hint = Some(last_hint);
                    }
                    _ => {}
                }
            }
            self.render_from_helper(event.node, trim)?;
        }

        // Store the hint so we can remove leading whitespace
        // after a block end tag
        self.end_tag_hint = hint;

        Ok(())
    }

    /// Lookup a field of a value.
    ///
    /// If the target value is not an object or array then this
    /// will yield None.
    pub fn field<'a, S: AsRef<str>>(
        &self,
        target: &'a Value,
        field: S,
    ) -> Option<&'a Value> {
        json::find_field(target, field)
    }

    /// Infallible variable lookup by path.
    fn lookup<'a>(&'a self, path: &'source Path) -> Option<&'a Value> {
        //println!("Lookup path {:?}", path.as_str());
        //println!("Lookup path {:?}", path);

        // Handle explicit `@root` reference
        if path.is_root() {
            json::find_parts(
                path.components().iter().skip(1).map(|c| c.as_str()),
                &self.root,
            )
        // Handle explicit this
        } else if path.is_explicit() {
            let value = if let Some(scope) = self.scopes.last() {
                if let Some(base) = scope.base_value() {
                    base
                } else {
                    &self.root
                }
            } else {
                &self.root
            };

            // Handle explicit this only
            if path.components().len() == 1 {
                Some(value)
            // Otherwise lookup in this context
            } else {
                json::find_parts(
                    path.components().iter().skip(1).map(|c| c.as_str()),
                    value,
                )
            }
        // Handle local @variable references which must
        // be resolved using the current scope
        } else if path.is_local() {
            if let Some(scope) = self.scopes.last() {
                json::find_parts(
                    path.components().iter().map(|c| c.as_str()),
                    scope.as_value(),
                )
            } else {
                None
            }
        } else if path.parents() > 0 {
            // Combine so that the root object is
            // treated as a scope
            let mut all = vec![&self.root];
            let mut values: Vec<&'a Value> =
                self.scopes.iter().map(|s| s.as_value()).collect();
            all.append(&mut values);

            if all.len() > path.parents() as usize {
                let index: usize = all.len() - (path.parents() as usize + 1);
                if let Some(value) = all.get(index) {
                    json::find_parts(
                        path.components().iter().map(|c| c.as_str()),
                        value,
                    )
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            // Lookup in the current scope
            if let Some(scope) = self.scopes.last() {
                json::find_parts(
                    path.components().iter().map(|c| c.as_str()),
                    scope.as_value(),
                ).or(
                    json::find_parts(
                        path.components().iter().map(|c| c.as_str()),
                        &self.root,
                    )
                )
            // Lookup in the root scope
            } else {
                json::find_parts(
                    path.components().iter().map(|c| c.as_str()),
                    &self.root,
                )
            }
        }
    }

    /// Create the context arguments list.
    fn arguments(
        &mut self,
        call: &'source Call<'_>,
    ) -> RenderResult<Vec<Value>> {
        let mut out: Vec<Value> = Vec::new();
        for p in call.arguments() {
            let arg = match p {
                ParameterValue::Json(val) => val.clone(),
                ParameterValue::Path(ref path) => {
                    self.lookup(path).cloned().unwrap_or(Value::Null)
                }
                ParameterValue::SubExpr(ref call) => {
                    self.statement(call)?.unwrap_or(Value::Null)
                }
            };
            out.push(arg);
        }
        Ok(out)
    }

    /// Create the context hash parameters.
    fn hash(
        &mut self,
        call: &'source Call<'_>,
    ) -> RenderResult<Map<String, Value>> {
        let mut out = Map::new();
        for (k, p) in call.hash() {
            let (key, value) = match p {
                ParameterValue::Json(val) => (k.to_string(), val.clone()),
                ParameterValue::Path(ref path) => {
                    let val = self.lookup(path).cloned().unwrap_or(Value::Null);
                    (k.to_string(), val)
                }
                ParameterValue::SubExpr(ref call) => (
                    k.to_string(),
                    self.statement(call)?.unwrap_or(Value::Null),
                ),
            };
            out.insert(key, value);
        }

        Ok(out)
    }

    /// Local helpers can be used by global helpers to define 
    /// helpers for a render.
    pub fn local_helpers(&self) -> &Option<&'render HelperRegistry<'render>> {
        &self.local_helpers 
    }

    fn invoke(
        &mut self,
        kind: HelperType,
        name: &'source str,
        call: &'source Call<'_>,
        mut content: Option<&'source Node<'source>>,
        ) -> RenderResult<HelperValue> {

        let args = self.arguments(call)?;
        let hash = self.hash(call)?;
        let context = Context::new(name, args, hash);

        let value: Option<Value> = match kind {
            HelperType::Value => {
                if let Some(local_helpers) = self.local_helpers {
                    if let Some(helper) = local_helpers.get(name) {
                        helper.call(self, context)?
                    } else { None }
                } else {
                    if let Some(helper) = self.helpers.get(name) {
                        helper.call(self, context)?
                    } else { None }
                }
            } 
            HelperType::Block => {
                let template = content.take().unwrap();    

                if let Some(local_helpers) = self.local_helpers {
                    if let Some(helper) = local_helpers.get_block(name) {
                        let block = BlockTemplate::new(template);
                        helper.call(self, context, block).map(|_| None)?
                    } else { None }
                } else {
                    if let Some(helper) = self.helpers.get_block(name) {
                        let block = BlockTemplate::new(template);
                        helper.call(self, context, block).map(|_| None)?
                    } else { None }
                }
            }
            HelperType::Raw => {
                todo!("Resolve raw helpers");
            }
        };

        Ok(value)
    }

    // Fallible version of path lookup.
    fn resolve(
        &mut self,
        path: &'source Path<'_>,
    ) -> RenderResult<HelperValue> {
        if let Some(value) = self.lookup(path).cloned().take() {
            return Ok(Some(value));
        } else {
            panic!("Missing variable with path {:?}", path);
        }
    }

    /// Invoke a call and return the result.
    pub fn call(
        &mut self,
        call: &'source Call<'_>,
    ) -> RenderResult<HelperValue> {
        match call.target() {
            CallTarget::Path(ref path) => {
                // Explicit paths should resolve to a lookup
                if path.is_explicit() {
                    Ok(self.lookup(path).cloned())

                // Handle @partial-block variables!
                } else if path.components().len() == 1
                    && path.components().get(0).unwrap().as_str()
                        == PARTIAL_BLOCK
                {
                    if let Some(scope) = self.scopes.last_mut() {
                        if let Some(node) = scope.partial_block_mut().take() {
                            self.template(node)?;
                        }
                        Ok(None)
                    } else {
                        Ok(None)
                    }

                // Simple paths may be helpers
                } else if path.is_simple() {
                    if let Some(_) = self.helpers.get(path.as_str()) {
                        self.invoke(HelperType::Value, path.as_str(), call, None)
                    } else {
                        self.resolve(path)
                    }
                } else {
                    self.resolve(path)
                }
            }
            CallTarget::SubExpr(ref sub) => self.call(sub),
        }
    }

    fn statement(
        &mut self,
        call: &'source Call<'_>,
    ) -> RenderResult<HelperValue> {
        if call.is_partial() {
            self.render_partial(call, None)?;
            Ok(None)
        } else {
            Ok(self.call(call)?)
        }
    }

    fn get_partial_name<'a>(
        &mut self,
        call: &'source Call<'_>,
    ) -> RenderResult<String> {
        match call.target() {
            CallTarget::Path(ref path) => {
                if path.is_simple() {
                    return Ok(path.as_str().to_string());
                } else {
                    panic!("Partials must be simple identifiers");
                }
            }
            CallTarget::SubExpr(ref call) => {
                let result = self.statement(call)?.unwrap_or(Value::Null);
                return Ok(json::stringify(&result));
            }
        }
        Err(RenderError::PartialNameResolve(call.as_str().to_string()))
    }

    fn render_partial(
        &mut self,
        call: &'source Call<'_>,
        partial_block: Option<&'source Node<'source>>,
    ) -> RenderResult<()> {
        let name = self.get_partial_name(call)?;
        let template = self
            .templates
            .get(&name)
            .ok_or_else(|| RenderError::PartialNotFound(name))?;

        let node = template.node();
        let hash = self.hash(call)?;
        let mut scope = Scope::new_locals(hash);
        scope.set_partial_block(partial_block);
        self.scopes.push(scope);
        // WARN: We must iterate the document child nodes
        // WARN: when rendering partials otherwise the
        // WARN: rendering process will halt after the first partial!
        for event in node.block_iter().trim(self.hint) {
            self.render_node(event.node, event.trim)?;
        }
        self.scopes.pop();
        Ok(())
    }

    fn block(
        &mut self,
        node: &'source Node<'_>,
        block: &'source Block<'_>,
    ) -> RenderResult<()> {
        let call = block.call();

        if call.is_partial() {
            self.render_partial(call, Some(node))?;
        } else {
            match call.target() {
                CallTarget::Path(ref path) => {
                    if path.is_simple() {
                        self.invoke(
                            HelperType::Block, path.as_str(), call, Some(node))?;
                    }
                }
                //CallTarget::SubExpr(ref sub) => self.call(sub),
                _ => todo!("Handle block sub expression for cal target"),
            }
        }
        Ok(())
    }

    /// Render and return a helper result wrapping the underlying render error.
    pub(crate) fn render_from_helper(
        &mut self,
        node: &'source Node<'_>,
        trim: TrimState,
    ) -> BlockResult {
        self.render_node(node, trim)
            .map_err(|e| HelperError::Render(Box::new(e)))
    }

    pub(crate) fn render_node(
        &mut self,
        node: &'source Node<'_>,
        trim: TrimState,
    ) -> RenderResult<()> {
        self.trim = trim;
        self.hint = Some(node.trim());

        if let Some(hint) = self.end_tag_hint.take() {
            if hint.after {
                self.trim.start = true;
            }        
        }

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
            Node::Document(_) => {}
            Node::Condition(_) => {}
            Node::Statement(ref call) => {
                if let Some(ref value) = self.statement(call)? {
                    let val = json::stringify(value);
                    self.write_str(&val, call.is_escaped())?;
                }
            }
            Node::Block(ref block) => {
                self.block(node, block)?;
            }
            _ => todo!("Render other node types"),
        }

        Ok(())
    }

    fn write_str(&mut self, s: &str, escape: bool) -> RenderResult<usize> {

        let val = if self.trim.start { s.trim_start() } else { s };
        let val = if self.trim.end { val.trim_end() } else { val };
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
}

impl Assertion for Render<'_, '_, '_> {
    fn arity(&self, ctx: &Context<'_>, range: Range<usize>) -> BlockResult {
        if range.start == range.end {
            if ctx.arguments().len() != range.start {
                return Err(HelperError::ArityExact(
                    ctx.name().to_owned(),
                    range.start,
                ));
            }
        } else {
            if ctx.arguments().len() < range.start
                || ctx.arguments().len() > range.end
            {
                return Err(HelperError::ArityRange(
                    ctx.name().to_owned(),
                    range.start,
                    range.end,
                ));
            }
        }
        Ok(())
    }
}
