//! Abstract syntax tree node types.
use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

use serde_json::Value;

use crate::parser::{
    iter::{BlockIter, NodeIter},
    trim::TrimHint,
};

static WHITESPACE: &str = "~";
static ROOT: &str = "@root";
//pub static LEVEL: &str = "@level";

/// Trait for nodes that reference a slice of the
/// source template.
pub trait Slice<'source>: fmt::Display + fmt::Debug {
    /// Get a string slice of the full span for this node.
    fn as_str(&self) -> &'source str;

    /// Get the underlying template source.
    fn source(&self) -> &'source str;
}

/// Trait for elements that expect to be closed.
pub trait Element<'source> {
    /// Get the string for the open tag.
    fn open(&self) -> &'source str;

    /// Get the string for the close tag.
    ///
    /// If no close span has been set which can happen if the
    /// element has no end tag this should return the empty string.
    fn close(&self) -> &'source str;

    /// Get the span for the open tag.
    fn open_span(&self) -> &Range<usize>;

    /// Get the span for the close tag.
    fn close_span(&self) -> &Option<Range<usize>>;

    /// Determine if this element has been closed.
    fn is_closed(&self) -> bool;

    /// Mark this element as correctly terminated.
    fn exit(&mut self, close: Range<usize>);
}

/// Enumeration of the different kinds of nodes.
#[derive(Eq, PartialEq)]
pub enum Node<'source> {
    Document(Document<'source>),
    Text(Text<'source>),
    Statement(Call<'source>),
    Block(Block<'source>),
    Condition(Block<'source>),
    RawStatement(TextBlock<'source>),
    RawComment(TextBlock<'source>),
    Comment(TextBlock<'source>),
}

impl<'source> Node<'source> {
    pub fn trim(&self) -> TrimHint {
        TrimHint {
            before: self.trim_before(),
            after: self.trim_after(),
        }
    }

    fn trim_before(&self) -> bool {
        match *self {
            Self::Document(_)
            | Self::Text(_)
            | Self::RawStatement(_)
            | Self::RawComment(_)
            | Self::Comment(_) => false,
            Self::Statement(ref n) => n.trim_before(),
            Self::Block(ref n) => n.trim_before(),
            Self::Condition(ref n) => n.trim_before(),
        }
    }

    fn trim_after(&self) -> bool {
        match *self {
            Self::Document(_)
            | Self::Text(_)
            | Self::RawStatement(_)
            | Self::RawComment(_)
            | Self::Comment(_) => false,
            Self::Statement(ref n) => n.trim_after(),
            Self::Block(ref n) => n.trim_after(),
            Self::Condition(ref n) => n.trim_after(),
        }
    }

    /// Iterate leaf nodes.
    pub fn iter<'a>(&'a self) -> NodeIter<'a> {
        NodeIter::new(self)
    }

    /// Iterate descendants of documents and blocks.
    pub fn block_iter<'a>(&'a self) -> BlockIter<'a> {
        BlockIter::new(self)
    }
}

impl<'source> Slice<'source> for Node<'source> {
    fn as_str(&self) -> &'source str {
        match *self {
            Self::Document(ref n) => n.as_str(),
            Self::Text(ref n) => n.as_str(),
            Self::Statement(ref n) => n.as_str(),
            Self::Block(ref n) => n.as_str(),
            Self::Condition(ref n) => n.as_str(),
            Self::RawStatement(ref n)
            | Self::RawComment(ref n)
            | Self::Comment(ref n) => n.as_str(),
        }
    }

    fn source(&self) -> &'source str {
        match *self {
            Self::Document(ref n) => n.source(),
            Self::Text(ref n) => n.source(),
            Self::RawStatement(ref n) => n.source(),
            Self::RawComment(ref n) => n.source(),
            Self::Comment(ref n) => n.source(),
            Self::Statement(ref n) => n.source(),
            Self::Block(ref n) => n.source(),
            Self::Condition(ref n) => n.source(),
        }
    }
}

impl fmt::Display for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Document(ref n) => n.fmt(f),
            Self::Text(ref n) => n.fmt(f),
            Self::Statement(ref n) => n.fmt(f),
            Self::Block(ref n) => n.fmt(f),
            Self::Condition(ref n) => n.fmt(f),
            Self::RawStatement(ref n)
            | Self::RawComment(ref n)
            | Self::Comment(ref n) => n.fmt(f),
        }
    }
}

impl fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Document(ref n) => fmt::Debug::fmt(n, f),
            Self::Text(ref n) => fmt::Debug::fmt(n, f),
            Self::Block(ref n) => fmt::Debug::fmt(n, f),
            Self::Condition(ref n) => fmt::Debug::fmt(n, f),
            Self::Statement(ref n) => fmt::Debug::fmt(n, f),
            Self::RawStatement(ref n)
            | Self::RawComment(ref n)
            | Self::Comment(ref n) => fmt::Debug::fmt(n, f),
        }
    }
}

/// Text nodes refer to a consecutive range of bytes.
#[derive(Eq, PartialEq)]
pub struct Text<'source>(pub &'source str, pub Range<usize>);

impl<'source> Slice<'source> for Text<'source> {
    fn as_str(&self) -> &'source str {
        &self.0[self.1.start..self.1.end]
    }

    fn source(&self) -> &'source str {
        self.0
    }
}

impl fmt::Display for Text<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for Text<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Text")
            .field("source", &self.as_str())
            .field("range", &self.1)
            .finish()
    }
}

/// Text blocks encapsulate a text node with start and end
/// ranges; used primarily for comments.
#[derive(Eq, PartialEq)]
pub struct TextBlock<'source> {
    source: &'source str,
    text: Text<'source>,
    open: Range<usize>,
    close: Range<usize>,
}

impl<'source> TextBlock<'source> {
    pub fn new(
        source: &'source str,
        text: Text<'source>,
        open: Range<usize>,
        close: Range<usize>,
    ) -> Self {
        Self {
            source,
            text,
            open,
            close,
        }
    }
}

impl<'source> Slice<'source> for TextBlock<'source> {
    fn as_str(&self) -> &'source str {
        &self.source[self.open.start..self.close.end]
    }

    fn source(&self) -> &'source str {
        self.source
    }
}

impl<'source> Into<Text<'source>> for TextBlock<'source> {
    fn into(self) -> Text<'source> {
        self.text
    }
}

impl fmt::Display for TextBlock<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for TextBlock<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextBlock")
            .field("source", &self.as_str())
            .field("open", &self.open)
            .field("close", &self.close)
            .finish()
    }
}

/// Indicates the kind of path component.
#[derive(Debug, Eq, PartialEq)]
pub enum ComponentType {
    Parent,
    ThisKeyword,
    ThisDotSlash,
    Identifier,
    LocalIdentifier,
    Delimiter,
    ArrayAccess,
}

/// Components form part of a path.
#[derive(Eq, PartialEq)]
pub struct Component<'source>(
    pub &'source str,
    pub ComponentType,
    pub Range<usize>,
);

impl<'source> Component<'source> {
    /// Determine if this is the special `@root` component.
    pub fn is_root(&self) -> bool {
        self.as_str() == ROOT
    }

    /// Get the kind of this component.
    pub fn kind(&self) -> &ComponentType {
        &self.1
    }

    /// The span for this component.
    pub fn span(&self) -> &Range<usize> {
        &self.2
    }

    /// Determine if this component is a local identifier; begins
    /// with an `@` symbol.
    pub fn is_local(&self) -> bool {
        &ComponentType::LocalIdentifier == self.kind()
    }

    /// Determine if this component is an identifier.
    pub fn is_identifier(&self) -> bool {
        &ComponentType::Identifier == self.kind()
    }

    /// Determine if this component uses an explicit this reference;
    /// the reference may be the keyword `this` or `./`.
    pub fn is_explicit(&self) -> bool {
        &ComponentType::ThisKeyword == self.kind()
            || self.is_explicit_dot_slash()
    }

    /// Determine if this component uses and explicit dot slash (`./`)
    /// reference.
    ///
    /// This is used by the path parser to determine if the next expected
    /// token should be a path delimiter or identifier.
    pub fn is_explicit_dot_slash(&self) -> bool {
        &ComponentType::ThisDotSlash == self.kind()
    }
}

impl<'source> Slice<'source> for Component<'source> {
    fn as_str(&self) -> &'source str {
        &self.0[self.span().start..self.span().end]
    }

    fn source(&self) -> &'source str {
        self.0
    }
}

impl fmt::Display for Component<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for Component<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Component")
            .field("source", &self.as_str())
            .field("kind", &self.1)
            .field("span", &self.2)
            .finish()
    }
}

#[derive(Eq, PartialEq)]
pub struct Path<'source> {
    source: &'source str,
    components: Vec<Component<'source>>,
    parents: u8,
    explicit: bool,
    root: bool,
}

impl<'source> Path<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            source,
            components: Vec::new(),
            parents: 0,
            explicit: false,
            root: false,
        }
    }

    pub fn add_component(&mut self, part: Component<'source>) {
        self.components.push(part);
    }

    pub fn components(&self) -> &Vec<Component<'source>> {
        &self.components
    }

    pub fn parents(&self) -> u8 {
        self.parents
    }

    pub fn set_parents(&mut self, parents: u8) {
        self.parents = parents;
    }

    pub fn set_root(&mut self, root: bool) {
        self.root = root;
    }

    pub fn is_root(&self) -> bool {
        self.root
    }

    pub fn set_explicit(&mut self, explicit: bool) {
        self.explicit = explicit;
    }

    pub fn is_explicit(&self) -> bool {
        self.explicit
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    pub fn is_local(&self) -> bool {
        return !self.components.is_empty()
            && self.components.first().unwrap().is_local();
    }

    pub fn is_simple(&self) -> bool {
        return self.components.len() == 1
            && self.components.first().unwrap().1 == ComponentType::Identifier;
    }
}

impl<'source> Slice<'source> for Path<'source> {
    fn as_str(&self) -> &'source str {
        if !self.components.is_empty() {
            let first = self.components.first().unwrap();
            let last = self.components.last().unwrap();
            &self.source[first.span().start..last.span().end]
        } else {
            ""
        }
    }

    fn source(&self) -> &'source str {
        self.source
    }
}

impl fmt::Display for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Path")
            .field("source", &self.as_str())
            .field("components", &self.components)
            .field("parents", &self.parents)
            .field("explicit", &self.explicit)
            .field("root", &self.root)
            .finish()
    }
}

/// Parameter values can be used as arguments or hash values.
#[derive(Debug, Eq, PartialEq)]
pub enum ParameterValue<'source> {
    /// A parameter that should resolve to a runtime variable.
    Path(Path<'source>),
    /// A literal JSON value.
    Json(Value),
    /// A sub-expression to be invoked at runtime to determine the value.
    SubExpr(Call<'source>),
}

/// Call targets represent either a helper call, partial render or variable path.
///
/// To support dynamic partials call targets may also be sub-expressions.
#[derive(Debug, Eq, PartialEq)]
pub enum CallTarget<'source> {
    Path(Path<'source>),
    SubExpr(Box<Call<'source>>),
}

impl<'source> CallTarget<'source> {
    pub fn as_str(&self) -> &'source str {
        match *self {
            Self::Path(ref path) => path.as_str(),
            Self::SubExpr(ref call) => call.as_str(),
        }
    }

    // FIXME!
    pub fn is_empty(&self) -> bool {
        match *self {
            Self::Path(ref path) => path.is_empty(),
            Self::SubExpr(ref call) => {
                //println!("Checking empty on sub expression...");
                call.is_empty()
            }
        }
    }
}

impl Default for CallTarget<'_> {
    fn default() -> Self {
        CallTarget::Path(Path::new(""))
    }
}

/// Call is a variable interpolation, helper invocation or partial
/// render; they have zero or more arguments and optional hash parameters.
///
/// The partial flag is used to indicate that this call should be
/// rendered as a partial.
#[derive(Default, Eq, PartialEq)]
pub struct Call<'source> {
    // Raw source input.
    source: &'source str,
    partial: bool,
    conditional: bool,
    open: Range<usize>,
    close: Option<Range<usize>>,
    target: CallTarget<'source>,
    arguments: Vec<ParameterValue<'source>>,
    hash: HashMap<&'source str, ParameterValue<'source>>,
}

impl<'source> Call<'source> {
    /// Create an open call.
    ///
    /// If it is correctly terminated the parser will call `exit()` to terminate
    /// the call statement.
    pub fn new(source: &'source str, open: Range<usize>) -> Self {
        Self {
            source,
            partial: false,
            conditional: false,
            open,
            close: None,
            target: CallTarget::Path(Path::new(source)),
            arguments: Vec::new(),
            hash: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.target.is_empty()
    }

    pub fn target(&self) -> &CallTarget<'source> {
        &self.target
    }

    pub fn has_target(&self) -> bool {
        self.target.as_str() != ""
    }

    pub fn set_target(&mut self, target: CallTarget<'source>) {
        self.target = target;
    }

    pub fn add_argument(&mut self, arg: ParameterValue<'source>) {
        self.arguments.push(arg);
    }

    pub fn arguments(&self) -> &Vec<ParameterValue<'source>> {
        &self.arguments
    }

    pub fn add_hash(
        &mut self,
        key: &'source str,
        val: ParameterValue<'source>,
    ) {
        self.hash.insert(key, val);
    }

    pub fn hash(&self) -> &HashMap<&'source str, ParameterValue<'source>> {
        &self.hash
    }

    /// The full range for this call; if the call is not closed
    /// only the open span is returned.
    pub fn span(&self) -> Range<usize> {
        if let Some(ref close) = self.close {
            self.open.start..close.end
        } else {
            self.open.clone()
        }
    }

    fn trim_before(&self) -> bool {
        self.open().ends_with(WHITESPACE)
    }

    fn trim_after(&self) -> bool {
        self.close().starts_with(WHITESPACE)
    }

    pub fn is_partial(&self) -> bool {
        self.partial
    }

    pub fn set_partial(&mut self, partial: bool) {
        self.partial = partial;
    }

    pub fn is_conditional(&self) -> bool {
        self.conditional
    }

    pub fn set_conditional(&mut self, conditional: bool) {
        self.conditional = conditional;
    }

    pub fn is_escaped(&self) -> bool {
        // FIXME: ensure this is not `true` for raw blocks!
        !self.open().starts_with("{{{")
    }
}

impl<'source> Slice<'source> for Call<'source> {
    fn as_str(&self) -> &'source str {
        //if let Some(ref close) = self.close {
        //return &self.source[self.open.end..close.start];
        //}

        if let Some(ref close) = self.close {
            return &self.source[self.open.start..close.end];
        }
        &self.source[self.open.start..self.open.end]
    }

    fn source(&self) -> &'source str {
        self.source
    }
}

impl<'source> Element<'source> for Call<'source> {
    fn open(&self) -> &'source str {
        &self.source[self.open.start..self.open.end]
    }

    fn close(&self) -> &'source str {
        if let Some(ref close) = self.close {
            return &self.source[close.start..close.end];
        }
        ""
    }

    fn open_span(&self) -> &Range<usize> {
        &self.open
    }

    fn close_span(&self) -> &Option<Range<usize>> {
        &self.close
    }

    fn is_closed(&self) -> bool {
        self.close.is_some()
    }

    fn exit(&mut self, close: Range<usize>) {
        self.close = Some(close);
    }
}

impl fmt::Display for Call<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for Call<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Call")
            .field("source", &self.as_str())
            .field("partial", &self.partial)
            .field("open", &self.open)
            .field("close", &self.close)
            .field("target", &self.target)
            .field("arguments", &self.arguments)
            .field("hash", &self.hash)
            .finish()
    }
}

/// Documents are abstract nodes that encapsulate a collection
/// of child nodes; they are used as the root node of a compiled template.
#[derive(Eq, PartialEq)]
pub struct Document<'source>(pub &'source str, pub Vec<Node<'source>>);

impl<'source> Document<'source> {
    pub fn nodes(&self) -> &Vec<Node<'source>> {
        &self.1
    }

    pub fn nodes_mut(&mut self) -> &mut Vec<Node<'source>> {
        &mut self.1
    }
}

impl<'source> Slice<'source> for Document<'source> {
    fn as_str(&self) -> &'source str {
        self.0
    }
    fn source(&self) -> &'source str {
        self.0
    }
}

impl fmt::Display for Document<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for Document<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Document").field("nodes", &self.1).finish()
    }
}

/*
#[derive(Eq, PartialEq)]
pub struct Condition<'source> {
    // Raw source input.
    source: &'source str,
    call: Call<'source>,
    nodes: Vec<Node<'source>>,
    close: Option<Range<usize>>,
}

impl<'source> Condition<'source> {
    pub fn new(source: &'source str, call: Call<'source>) -> Self {
        Self {
            source,
            call,
            nodes: Vec::new(),
            close: None,
        }
    }

    pub fn call(&self) -> &Call<'source> {
        &self.call
    }

    pub fn nodes(&self) -> &Vec<Node<'source>> {
        &self.nodes
    }

    pub fn close(&self) -> &'source str {
        if let Some(ref close) = self.close {
            &self.source[close.start..close.end]
        } else {
            ""
        }
    }

    pub fn trim_close(&self) -> TrimHint {
        TrimHint {
            before: self.trim_before_close(),
            after: self.trim_after_close(),
        }
    }

    fn trim_before(&self) -> bool {
        self.call.trim_before()
    }

    fn trim_after(&self) -> bool {
        self.call.trim_after()
    }

    fn trim_before_close(&self) -> bool {
        let close = self.close();
        close.len() > 2 && WHITESPACE == &close[2..3]
    }

    fn trim_after_close(&self) -> bool {
        let close = self.close();
        let index = close.len() - 3;
        close.len() > 2 && WHITESPACE == &close[index..index + 1]
    }
}

impl<'source> Slice<'source> for Condition<'source> {
    fn as_str(&self) -> &'source str {
        let open = &self.call.open;
        let close = self.close.as_ref().unwrap_or(open);
        &self.source[open.start..close.end]
    }

    fn source(&self) -> &'source str {
        self.source
    }
}

impl fmt::Display for Condition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for Condition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Condition")
            .field("close", &self.close)
            .field("call", &self.call)
            .field("nodes", &self.nodes)
            .finish()
    }
}
*/

/// Block encapsulates an inner template.
///
/// These nodes are rendered indirectly via registered helpers
/// that should call back in to the renderer.
///
/// When a block has the raw flag set it should only contain a
/// single `Text` child node.
#[derive(Eq, PartialEq)]
pub struct Block<'source> {
    source: &'source str,
    nodes: Vec<Node<'source>>,
    raw: bool,
    open: Range<usize>,
    close: Option<Range<usize>>,
    call: Call<'source>,
    conditionals: Vec<Node<'source>>,
}

impl<'source> Block<'source> {
    /// Create a new block.
    pub fn new(source: &'source str, open: Range<usize>, raw: bool) -> Self {
        Self {
            source,
            nodes: Vec::new(),
            raw,
            open,
            close: None,
            call: Default::default(),
            conditionals: Vec::new(),
        }
    }

    /// Get the call for the block.
    pub fn call(&self) -> &Call<'source> {
        &self.call
    }

    /// Set the call for the block.
    pub fn set_call(&mut self, call: Call<'source>) {
        self.call = call;
    }

    /// The name of this block extracted from the call target.
    ///
    /// This will only be available if the call target is a path
    /// and the path is a simple identifier.
    pub fn name(&self) -> Option<&'source str> {
        match self.call.target() {
            CallTarget::Path(ref path) => {
                if path.is_simple() {
                    let id = path.components().first().unwrap();
                    Some(id.as_str())
                } else {
                    None
                }
            }
            CallTarget::SubExpr(_) => None,
        }
    }

    /// Determine if this block has the raw flag.
    pub fn is_raw(&self) -> bool {
        self.raw
    }

    /// Add a condition to this block.
    pub fn add_condition(&mut self, condition: Block<'source>) {
        self.close_condition(condition.call.open.clone());
        self.conditionals.push(Node::Block(condition));
    }

    /// Get the list of conditional blocks.
    pub fn conditions(&self) -> &Vec<Node<'source>> {
        &self.conditionals
    }

    /// Add a node to this block; if this block has
    /// conditionals then the node is added to the last conditional.
    pub fn push(&mut self, node: Node<'source>) {
        if !self.conditionals.is_empty() {
            let mut last = self.conditionals.last_mut().unwrap();
            match &mut last {
                Node::Block(ref mut condition) => {
                    condition.nodes.push(node);
                }
                _ => {}
            }
        } else {
            self.nodes.push(node);
        }
    }

    /// The collection of nodes for this block.
    ///
    /// For raw blocks this should always be a single `Text` node.
    pub fn nodes(&self) -> &'source Vec<Node> {
        &self.nodes
    }

    /// The trim hint for the close tag.
    pub fn trim_close(&self) -> TrimHint {
        TrimHint {
            before: self.trim_before_close(),
            after: self.trim_after_close(),
        }
    }

    fn close_condition(&mut self, span: Range<usize>) {
        if !self.conditionals.is_empty() {
            if span.start > 0 {
                let close = span.start - 1..span.start;
                let mut last = self.conditionals.last_mut().unwrap();
                match &mut last {
                    Node::Condition(ref mut condition) => {
                        condition.close = Some(close);
                    }
                    _ => {}
                }
            }
        }
    }

    fn trim_before(&self) -> bool {
        let open = self.open();
        if self.is_raw() {
            open.len() > 4 && WHITESPACE == &open[4..5]
        } else {
            open.len() > 2 && WHITESPACE == &open[2..3]
        }
    }

    fn trim_after(&self) -> bool {
        self.call.trim_after()
    }

    fn trim_before_close(&self) -> bool {
        let close = self.close();
        if self.is_raw() {
            close.len() > 4 && WHITESPACE == &close[4..5]
        } else {
            close.len() > 2 && WHITESPACE == &close[2..3]
        }
    }

    fn trim_after_close(&self) -> bool {
        let close = self.close();

        if self.is_raw() {
            if close.len() > 5 {
                let index = close.len() - 5;
                close.len() > 4 && WHITESPACE == &close[index..index + 1]
            } else {
                false
            }
        } else {
            if close.len() > 3 {
                let index = close.len() - 3;
                close.len() > 2 && WHITESPACE == &close[index..index + 1]
            } else {
                false
            }
        }
    }
}

impl<'source> Slice<'source> for Block<'source> {
    fn as_str(&self) -> &'source str {
        let close = self.close.clone().unwrap_or(0..self.source.len());
        &self.source[self.open.start..close.end]
    }

    fn source(&self) -> &'source str {
        self.source
    }
}

impl<'source> Element<'source> for Block<'source> {
    fn open(&self) -> &'source str {
        &self.source[self.open.start..self.open.end]
    }

    fn close(&self) -> &'source str {
        if let Some(ref close) = self.close {
            &self.source[close.start..close.end]
        } else {
            ""
        }
    }

    fn open_span(&self) -> &Range<usize> {
        &self.open
    }

    fn close_span(&self) -> &Option<Range<usize>> {
        &self.close
    }

    fn is_closed(&self) -> bool {
        self.close.is_some()
    }

    fn exit(&mut self, span: Range<usize>) {
        // NOTE: close_condition() sets the span up until the next
        // NOTE: block but when we exit a block node the last conditional
        // NOTE: needs a close matching the end tag so that whitespace
        // NOTE: trim logic is correct.
        if !self.conditionals.is_empty() {
            let mut last = self.conditionals.last_mut().unwrap();
            match &mut last {
                Node::Block(ref mut condition) => {
                    condition.close = Some(span.clone());
                }
                _ => {}
            }
        }

        self.close = Some(span);
    }
}

impl fmt::Display for Block<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in self.nodes() {
            t.fmt(f)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Block<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("open", &self.open)
            .field("close", &self.close)
            .field("call", &self.call)
            .field("nodes", &self.nodes)
            .finish()
    }
}
