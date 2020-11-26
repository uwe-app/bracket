//! Render a template to output using the data.
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::{
    error::{HelperError, RenderError},
    helper::{Helper, HelperResult, LocalHelper},
    json,
    output::{Output, StringOutput},
    parser::{
        ast::{
            Block, Call, CallTarget, Lines, Link, Node, ParameterValue, Path,
            Slice,
        },
        path,
    },
    template::Template,
    trim::{TrimHint, TrimState},
    Registry, RenderResult,
};

static PARTIAL_BLOCK: &str = "@partial-block";
static HELPER_MISSING: &str = "helperMissing";
static BLOCK_HELPER_MISSING: &str = "blockHelperMissing";
static HELPER_LINK: &str = "link";

type HelperValue = Option<Value>;

pub mod assert;
pub mod context;
pub mod scope;

pub use assert::{assert, Type};
pub use context::{Context, Property};
pub use scope::Scope;

/// Maximum stack size for helper calls
static STACK_MAX: usize = 32;

enum HelperTarget<'a> {
    Name(&'a str),
    Helper(&'a Box<dyn Helper + 'a>),
}

/// Call site keeps track of calls so we can
/// detect cyclic calls and therefore prevent a
/// stack overflow by returning a render
/// error when a cycle is detected.
///
/// Note that we must distinguish between helper
/// types otherwise the `if` helper will not work
/// as expected as it returns values and handles
/// block templates.
#[derive(Eq, PartialEq, Hash, Debug, Clone)]
enum CallSite {
    Partial(String),
    Helper(String),
    BlockHelper(String),
}

impl fmt::Display for CallSite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", {
            match *self {
                CallSite::Partial(ref name) => format!("partial#{}", name),
                CallSite::Helper(ref name) => format!("helper#{}", name),
                CallSite::BlockHelper(ref name) => format!("block#{}", name),
            }
        })
    }
}

impl Into<String> for CallSite {
    fn into(self) -> String {
        match self {
            CallSite::Partial(name)
            | CallSite::Helper(name)
            | CallSite::BlockHelper(name) => name,
        }
    }
}

/// Render a template.
pub struct Render<'render> {
    registry: &'render Registry<'render>,
    local_helpers: Rc<RefCell<HashMap<String, Box<dyn LocalHelper + 'render>>>>,
    partials: HashMap<String, &'render Node<'render>>,
    name: &'render str,
    root: Value,
    writer: Box<&'render mut dyn Output>,
    scopes: Vec<Scope>,
    trim: TrimState,
    hint: Option<TrimHint>,
    end_tag_hint: Option<TrimHint>,
    stack: Vec<CallSite>,
}

impl<'render> Render<'render> {
    /// Create a renderer.
    ///
    /// You should not need to create a renderer directly, instead
    /// use the functions provided by the `Registry`.
    pub fn new<T>(
        registry: &'render Registry<'render>,
        name: &'render str,
        data: &T,
        writer: Box<&'render mut dyn Output>,
    ) -> RenderResult<Self>
    where
        T: Serialize,
    {
        let root = serde_json::to_value(data).map_err(RenderError::from)?;
        let scopes: Vec<Scope> = Vec::new();

        Ok(Self {
            registry,
            local_helpers: Rc::new(RefCell::new(HashMap::new())),
            partials: HashMap::new(),
            name,
            root,
            writer,
            scopes,
            trim: Default::default(),
            hint: None,
            end_tag_hint: None,
            stack: Vec::new(),
        })
    }

    /// Get a reference to the registry.
    pub fn registry(&self) -> &Registry<'_> {
        self.registry
    }

    /// Render a node by iterating it's children.
    ///
    /// The supplied node should be a document or block node.
    pub fn render(&mut self, node: &'render Node<'render>) -> RenderResult<()> {
        for event in node.into_iter().event(Default::default()) {
            self.render_node(event.node, event.trim)?;
        }
        Ok(())
    }

    /// Get a named template.
    pub fn get_template(&self, name: &str) -> Option<&'render Template> {
        self.registry.get_template(name)
    }

    /// Get a mutable reference to the output destination.
    ///
    /// You should prefer the `write()` and `write_escaped()` functions
    /// when writing strings but if you want to write bytes directly to
    /// the output destination you can use this reference.
    pub fn out(&mut self) -> &mut Box<&'render mut dyn Output> {
        &mut self.writer
    }

    /// Escape a value using the current escape function.
    pub fn escape(&self, val: &str) -> String {
        (self.registry.escape())(val)
    }

    /// Write a string to the output destination.
    pub fn write(&mut self, s: &str) -> HelperResult<usize> {
        self.write_str(s, false)
            .map_err(Box::new)
            .map_err(HelperError::from)
    }

    /// Write a string to the output destination and escape the content
    /// using the current escape function.
    pub fn write_escaped(&mut self, s: &str) -> HelperResult<usize> {
        self.write_str(s, true)
            .map_err(Box::new)
            .map_err(HelperError::from)
    }

    /// Push a scope onto the stack.
    pub fn push_scope(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    /// Remove a scope from the stack.
    pub fn pop_scope(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    /// Get a mutable reference to the current scope.
    pub fn scope_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.last_mut()
    }

    /// Reference to the root data for the render.
    pub fn data(&self) -> &Value {
        &self.root
    }

    /// Evaluate the block conditionals and find
    /// the first node that should be rendered.
    pub fn inverse<'a>(
        &mut self,
        template: &'a Node<'a>,
    ) -> Result<Option<&'a Node<'a>>, HelperError> {
        let mut alt: Option<&'a Node<'_>> = None;
        let mut branch: Option<&'a Node<'_>> = None;

        match template {
            Node::Block(ref block) => {
                if !block.conditions().is_empty() {
                    for node in block.conditions().iter() {
                        match node {
                            Node::Block(clause) => {
                                // Got an else clause, last one wins!
                                if clause.call().is_empty() {
                                    alt = Some(node);
                                } else {
                                    if let Some(value) = self
                                        .call(clause.call())
                                        .map_err(Box::new)?
                                    {
                                        if json::is_truthy(&value) {
                                            branch = Some(node);
                                            break;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(branch.or(alt))
    }

    /// Render an inner template.
    ///
    /// Block helpers should call this when they want to render an inner template.
    pub fn template(
        &mut self,
        node: &'render Node<'render>,
    ) -> Result<(), HelperError> {
        let mut hint: Option<TrimHint> = None;
        for event in node.into_iter().event(self.hint) {
            let mut trim = event.trim;

            if event.first {
                let hint = node.trim();
                if hint.after {
                    trim.start = true;
                }
            }

            if event.last {
                match node {
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

            self.render_node(event.node, trim)
                .map_err(|e| HelperError::Render(Box::new(e)))?;
        }

        // Store the hint so we can remove leading whitespace
        // after a block end tag
        self.end_tag_hint = hint;

        Ok(())
    }

    /// Render a node and buffer the result to a string.
    ///
    /// The call stack and scopes are inherited from this renderer.
    ///
    /// The supplied node should be a document or block node.
    pub fn buffer(
        &self,
        node: &'render Node<'render>,
    ) -> Result<String, HelperError> {
        let mut writer = StringOutput::new();
        let mut rc = Render::new(
            self.registry,
            self.name,
            &self.root,
            Box::new(&mut writer),
        )
        .map_err(Box::new)?;

        // Inherit the stack and scope from this renderer
        rc.stack = self.stack.clone();
        rc.scopes = self.scopes.clone();

        // NOTE: call `template()` not `render()` so trim settings
        // NOTE: on the parent node are respected!
        rc.template(node)?;

        // Must drop the renderer to take ownership of the string buffer
        drop(rc);

        Ok(writer.into())
    }

    /// Evaluate a path and return the resolved value.
    ///
    /// This allows helpers to find variables in the template data
    /// using the familiar path syntax such as `@root.name`.
    ///
    /// Paths are evaluated using the current scope so local variables
    /// in the current scope will be resolved.
    ///
    /// Paths are dynamically evaluated so syntax errors are caught and
    /// returned wrapped as `HelperError`.
    ///
    /// Sub-expressions are not executed.
    pub fn evaluate<'a>(
        &'a self,
        value: &str,
    ) -> HelperResult<Option<&'a Value>> {
        if let Some(path) = path::from_str(value)? {
            return Ok(self.lookup(&path));
        }
        Ok(None)
    }

    /// Evaluate a path and perform a type assertion on the value.
    ///
    /// If no value exists for the given path the value is
    /// treated as null and type assertion is performed on the
    /// null value.
    pub fn try_evaluate<'a>(
        &'a self,
        value: &str,
        kinds: &[Type],
    ) -> HelperResult<&'a Value> {
        let val = self.evaluate(value)?.or(Some(&Value::Null)).unwrap();
        let (result, kind) = assert(val, kinds);
        if !result {
            return Err(HelperError::TypeAssert(
                value.to_string(),
                kind.unwrap(),
                Type::from(val).to_string(),
            ));
        }
        Ok(val)
    }

    /// Infallible variable lookup by path.
    fn lookup<'a>(&'a self, path: &Path<'_>) -> Option<&'a Value> {
        //println!("Lookup path {:?}", path.as_str());
        //println!("Lookup path {:?}", path);

        // Handle explicit `@root` reference
        if path.is_root() {
            json::find_parts(
                path.components().iter().skip(1).map(|c| c.as_value()),
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
                    path.components().iter().skip(1).map(|c| c.as_value()),
                    value,
                )
            }
        // Handle local @variable references which must
        // be resolved using the current scope
        } else if path.is_local() {
            if let Some(scope) = self.scopes.last() {
                json::find_parts(
                    path.components().iter().map(|c| c.as_value()),
                    scope.locals(),
                )
            } else {
                None
            }
        } else if path.parents() > 0 {
            // Combine so that the root object is
            // treated as a scope
            let mut all = vec![&self.root];
            // FIXME: use base_value() here!
            let mut values: Vec<&'a Value> =
                self.scopes.iter().map(|s| s.locals()).collect();
            all.append(&mut values);

            if all.len() > path.parents() as usize {
                let index: usize = all.len() - (path.parents() as usize + 1);
                if let Some(value) = all.get(index) {
                    json::find_parts(
                        path.components().iter().map(|c| c.as_value()),
                        value,
                    )
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            let mut values: Vec<&Value> = self
                .scopes
                .iter()
                .filter(|v| v.base_value().is_some())
                .map(|v| v.base_value().as_ref().unwrap())
                .rev()
                .collect();
            values.push(&self.root);

            for v in values {
                if let Some(res) = json::find_parts(
                    path.components().iter().map(|c| c.as_value()),
                    v,
                ) {
                    return Some(res)
                }
            }
            None
        }
    }

    /// Create the context arguments list.
    fn arguments(&mut self, call: &Call<'_>) -> RenderResult<Vec<Value>> {
        let mut out: Vec<Value> = Vec::new();
        for p in call.arguments() {
            let arg = match p {
                ParameterValue::Json {
                    ref value,
                    span: _,
                    line: _,
                } => value.clone(),
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
    fn hash(&mut self, call: &Call<'_>) -> RenderResult<Map<String, Value>> {
        let mut out = Map::new();
        for (k, p) in call.parameters() {
            let (key, value) = match p {
                ParameterValue::Json {
                    ref value,
                    span: _,
                    line: _,
                } => (k.to_string(), value.clone()),
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

    /// Register a local helper.
    ///
    /// Local helpers are available for the scope of the parent helper.
    pub fn register_local_helper(
        &mut self,
        name: &'render str,
        helper: Box<dyn LocalHelper + 'render>,
    ) {
        let registry = Rc::make_mut(&mut self.local_helpers);
        registry.borrow_mut().insert(name.to_string(), helper);
    }

    /// Remove a local helper.
    ///
    /// Local helpers will be removed once a helper call has finished
    /// but you can call this if you want to be explicit.
    pub fn unregister_local_helper(&mut self, name: &'render str) {
        let registry = Rc::make_mut(&mut self.local_helpers);
        registry.borrow_mut().remove(name);
    }

    fn invoke<'a>(
        &mut self,
        name: &str,
        target: HelperTarget<'a>,
        call: &Call<'_>,
        content: Option<&'render Node<'render>>,
        text: Option<&'render str>,
        property: Option<Property>,
    ) -> RenderResult<HelperValue> {
        let site = if content.is_some() {
            CallSite::BlockHelper(name.to_string())
        } else {
            CallSite::Helper(name.to_string())
        };

        let amount = self.stack.iter().filter(|&n| *n == site).count();
        if amount >= STACK_MAX {
            return Err(RenderError::HelperCycle(site.into()));
        }
        self.stack.push(site);

        let args = self.arguments(call)?;
        let hash = self.hash(call)?;
        let mut context =
            Context::new(call, name.to_owned(), args, hash, text, property);

        let local_helpers = Rc::clone(&self.local_helpers);

        let value: Option<Value> = match target {
            HelperTarget::Name(name) => {
                if let Some(helper) = local_helpers.borrow().get(name) {
                    helper.call(self, &mut context, content)?
                } else if let Some(helper) = self.registry.helpers().get(name) {
                    helper.call(self, &mut context, content)?
                } else {
                    None
                }
            }
            // NOTE: evnet handlers will pass a reference to the helper.
            HelperTarget::Helper(helper) => {
                helper.call(self, &mut context, content)?
            }
        };

        drop(local_helpers);

        self.stack.pop();

        Ok(value)
    }

    fn has_helper(&mut self, name: &str) -> bool {
        self.local_helpers.borrow().get(name).is_some()
            || self.registry.helpers().get(name).is_some()
    }

    // Fallible version of path lookup.
    fn resolve(&mut self, path: &Path<'_>) -> RenderResult<HelperValue> {
        if let Some(value) = self.lookup(path).cloned().take() {
            Ok(Some(value))
        } else {
            if self.registry.strict() {
                Err(RenderError::VariableNotFound(path.as_str().to_string()))
            } else {
                // TODO: call a missing_variable handler?
                Ok(None)
            }
        }
    }

    /// Invoke a call and return the result.
    pub(crate) fn call(
        &mut self,
        call: &Call<'_>,
    ) -> RenderResult<HelperValue> {
        match call.target() {
            CallTarget::Path(ref path) => {
                // Explicit paths should resolve to a lookup
                if path.is_explicit() {
                    Ok(self.lookup(path).cloned())
                // Simple paths may be helpers
                } else if path.is_simple() {
                    if self.has_helper(path.as_str()) {
                        self.invoke(path.as_str(), HelperTarget::Name(path.as_str()), call, None, None, None)
                    } else {
                        let value = self.lookup(path).cloned();
                        if let None = value {
                            if self.has_helper(HELPER_MISSING) {
                                return self.invoke(
                                    HELPER_MISSING,
                                    HelperTarget::Name(HELPER_MISSING),
                                    call,
                                    None,
                                    None,
                                    None,
                                );
                            } else {
                                // TODO: also error if Call has arguments or parameters
                                if self.registry.strict() {
                                    return Err(RenderError::VariableNotFound(
                                        path.as_str().to_string(),
                                    ));
                                }
                            }
                        }
                        Ok(value)
                    }
                } else {
                    self.resolve(path)
                }
            }
            CallTarget::SubExpr(ref sub) => self.call(sub),
        }
    }

    fn statement(&mut self, call: &Call<'_>) -> RenderResult<HelperValue> {
        if call.is_partial() {
            self.render_partial(call, None)?;
            Ok(None)
        } else {
            Ok(self.call(call)?)
        }
    }

    fn get_partial_name<'a>(
        &mut self,
        call: &Call<'_>,
    ) -> RenderResult<String> {
        match call.target() {
            CallTarget::Path(ref path) => {
                if path.as_str() == PARTIAL_BLOCK {
                    return Ok(PARTIAL_BLOCK.to_string());
                } else if path.is_simple() {
                    return Ok(path.as_str().to_string());
                } else {
                    return Err(RenderError::PartialIdentifier(
                        path.as_str().to_string(),
                    ));
                }
            }
            CallTarget::SubExpr(ref call) => {
                let result = self.statement(call)?.unwrap_or(Value::Null);
                return Ok(json::stringify(&result));
            }
        }
    }

    fn render_partial(
        &mut self,
        call: &Call<'_>,
        partial_block: Option<&'render Node<'render>>,
    ) -> RenderResult<()> {
        let name = self.get_partial_name(call)?;

        let site = CallSite::Partial(name.to_string());
        if self.stack.contains(&site) {
            return Err(RenderError::PartialCycle(site.into()));
        }
        self.stack.push(site);

        if let Some(node) = partial_block {
            self.partials.insert(PARTIAL_BLOCK.to_string(), node);
        }

        let node = if let Some(local_partial) = self.partials.get(&name) {
            local_partial
        } else {
            let template = self
                .get_template(&name)
                .ok_or_else(|| RenderError::PartialNotFound(name))?;

            template.node()
        };

        let hash = self.hash(call)?;
        let scope = Scope::from(hash);
        self.scopes.push(scope);
        // WARN: We must iterate the document child nodes
        // WARN: when rendering partials otherwise the
        // WARN: rendering process will halt after the first partial!
        for event in node.into_iter().event(self.hint) {
            self.render_node(event.node, event.trim)?;
        }
        self.scopes.pop();

        self.stack.pop();

        Ok(())
    }

    fn block_helper_missing(
        &mut self,
        node: &'render Node<'render>,
        _block: &'render Block<'render>,
        call: &'render Call<'render>,
        text: Option<&str>,
        raw: bool,
    ) -> RenderResult<()> {
        // Handling a raw block without a corresponding helper
        // so we just write out the content
        if raw {
            if let Some(text) = text {
                self.write_str(text, false)?;
            }
        } else {
            match call.target() {
                CallTarget::Path(ref path) => {
                    if let Some(value) = self.lookup(path).cloned() {
                        if self.has_helper(BLOCK_HELPER_MISSING) {
                            let prop = Property {
                                name: path.as_str().to_string(),
                                value,
                            };
                            self.invoke(
                                BLOCK_HELPER_MISSING,
                                HelperTarget::Name(BLOCK_HELPER_MISSING),
                                call,
                                Some(node),
                                None,
                                Some(prop),
                            )?;
                        } else {
                            // Default behavior is to just render the block
                            self.template(node)?;
                        }
                    } else if self.has_helper(HELPER_MISSING) {
                        self.invoke(HELPER_MISSING, HelperTarget::Name(HELPER_MISSING), call, None, None, None)?;
                    } else {
                        if self.registry.strict() {
                            return Err(RenderError::HelperNotFound(
                                path.as_str().to_string(),
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn block(
        &mut self,
        node: &'render Node<'render>,
        block: &'render Block<'render>,
    ) -> RenderResult<()> {
        let call = block.call();
        let raw = block.is_raw();

        if call.is_partial() {
            self.render_partial(call, Some(node))?;
        } else {
            match call.target() {
                CallTarget::Path(ref path) => {
                    if path.is_simple() {
                        let mut text: Option<&str> = None;

                        if raw {
                            // Raw block nodes should have a single Text child node
                            text = if !block.nodes().is_empty() {
                                Some(block.nodes().get(0).unwrap().as_str())
                            // Empty raw block should be treated as the empty string
                            } else {
                                Some("")
                            };

                            // Store the hint so we can remove leading whitespace
                            // after a raw block end tag
                            match node {
                                Node::Block(ref block) => {
                                    let hint = block.trim_close();

                                    // Trim leading inside a raw block
                                    if node.trim().after {
                                        if let Some(ref content) = text {
                                            text = Some(content.trim_start());
                                        }
                                    }

                                    // Trim trailing inside a raw block
                                    if hint.before {
                                        if let Some(ref content) = text {
                                            text = Some(content.trim_end());
                                        }
                                    }

                                    // Trim after the end tag
                                    self.end_tag_hint = Some(hint);
                                }
                                _ => {}
                            }
                        }

                        if self.has_helper(path.as_str()) {
                            self.invoke(
                                path.as_str(),
                                HelperTarget::Name(path.as_str()),
                                call,
                                Some(node),
                                text,
                                None,
                            )?;
                        } else {
                            return self.block_helper_missing(
                                node, block, call, text, raw,
                            );
                        }
                    } else {
                        return Err(RenderError::BlockIdentifier(
                            path.as_str().to_string(),
                        ));
                    }
                }
                CallTarget::SubExpr(ref _call) => {
                    return Err(RenderError::BlockTargetSubExpr)
                }
            }
        }
        Ok(())
    }

    // Try to call a link helper.
    fn link(&mut self, helper: &Box<dyn Helper + 'render>, link: &'render Link<'render>) -> RenderResult<()> {
        let lines = link.lines();
        let href = Value::String(link.href().to_string());
        let label = Value::String(link.label().to_string());
        let title = Value::String(link.title().to_string());

        // Build a call so that the helper invocation flows
        // through the standard logic.
        let mut call = Call::new(link.source(), 0..0, 0..0);
        call.add_argument(ParameterValue::from((
            href,
            link.href_span().clone(),
            lines.clone(),
        )));
        call.add_argument(ParameterValue::from((
            label,
            link.label_span().clone(),
            lines.clone(),
        )));
        call.add_argument(ParameterValue::from((
            title,
            link.title_span().clone(),
            lines.clone(),
        )));

        self.invoke(HELPER_LINK, HelperTarget::Helper(helper), &call, None, None, None)?;

        Ok(())
    }

    pub(crate) fn render_node(
        &mut self,
        node: &'render Node<'render>,
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
            Node::RawStatement(ref n) => {
                let raw = &n.as_str()[1..];
                self.write_str(raw, false)?;
            }
            Node::Link(ref n) => {
                if cfg!(feature = "links") {
                    if let Some(helper) = &self.registry.handlers().link {
                        self.link(helper, n)?;
                    } else {
                        self.write_str(n.as_str(), false)?;
                    }
                } else {
                    self.write_str(n.as_str(), false)?;
                }
            }
            Node::RawComment(_) => {}
            Node::Comment(_) => {}
            Node::Document(_) => {}
            Node::Statement(ref call) => {
                if let Some(ref value) = self.statement(call)? {
                    let val = json::stringify(value);
                    self.write_str(&val, call.is_escaped())?;
                }
            }
            Node::Block(ref block) => {
                self.block(node, block)?;
            }
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
            let escaped = (self.registry.escape())(val);
            Ok(self.writer.write_str(&escaped).map_err(RenderError::from)?)
        } else {
            Ok(self.writer.write_str(val).map_err(RenderError::from)?)
        }
    }
}
