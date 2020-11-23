//! Abstract syntax tree node types.
use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

use serde_json::Value;

use crate::{parser::iter::BranchIter, trim::TrimHint};

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

/// Trait for nodes that track line numbers.
///
/// Line numbers begin at index zero.
pub trait Lines {
    /// Reference to the line range for the node.
    fn lines(&self) -> &Range<usize>;

    /// Mutable reference to the line range for the node.
    fn lines_mut(&mut self) -> &mut Range<usize>;

    /// Set the end of the lines range.
    fn lines_end(&mut self, line: &usize) {
        self.lines_mut().end = line.clone() + 1;
    }
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

    /// The full byte range for this element; if the element is not closed
    /// only the open span is returned.
    fn span(&self) -> Range<usize> {
        if let Some(ref close) = self.close_span() {
            self.open_span().start..close.end
        } else {
            self.open_span().clone()
        }
    }
}

/// Nodes form the abstract syntax tree.
///
/// Every node provides access to a [TrimHint](crate::trim::TrimHint) used
/// by the renderer to determine how whitespace should be handled.
#[derive(Eq, PartialEq)]
pub enum Node<'source> {
    /// Document nodes encapsulate a collection of children.
    Document(Document<'source>),
    /// Text nodes are a byte range.
    Text(Text<'source>),
    /// Statement is a variable interpolation, partial render or helper call.
    Statement(Call<'source>),
    /// Blocks encapsulate an inner template.
    ///
    /// Blocks have a `raw` flag which indicates that the content
    /// should not be interpreted. When the `raw` flag is set a block
    /// must only have a single `Text` child node.
    Block(Block<'source>),
    /// Raw statement is a statement preceeded by a backslash
    /// that should not be interpreted.
    RawStatement(TextBlock<'source>),
    /// Raw comments may contain nested templates (`{{!-- comment --}}`).
    RawComment(TextBlock<'source>),
    /// Comments may **not** contain nested templates (`{{! comment }}`).
    Comment(TextBlock<'source>),
    /// Link nodes are parsed from wiki-style links.
    Link(Link<'source>),
}

impl<'source> Node<'source> {
    /// Get the trim hint for this node.
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
            | Self::Comment(_)
            | Self::Link(_) => false,
            Self::Statement(ref n) => n.trim_before(),
            Self::Block(ref n) => n.trim_before(),
        }
    }

    fn trim_after(&self) -> bool {
        match *self {
            Self::Document(_)
            | Self::Text(_)
            | Self::RawStatement(_)
            | Self::RawComment(_)
            | Self::Comment(_)
            | Self::Link(_) => false,
            Self::Statement(ref n) => n.trim_after(),
            Self::Block(ref n) => n.trim_after(),
        }
    }

    /// Iterate descendants of documents and blocks.
    pub fn into_iter<'a>(&'a self) -> BranchIter<'a> {
        BranchIter::new(self)
    }
}

impl<'source> Slice<'source> for Node<'source> {
    fn as_str(&self) -> &'source str {
        match *self {
            Self::Document(ref n) => n.as_str(),
            Self::Text(ref n) => n.as_str(),
            Self::Statement(ref n) => n.as_str(),
            Self::Block(ref n) => n.as_str(),
            Self::Link(ref n) => n.as_str(),
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
            Self::Link(ref n) => n.source(),
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
            Self::Link(ref n) => n.fmt(f),
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
            Self::Statement(ref n) => fmt::Debug::fmt(n, f),
            Self::Block(ref n) => fmt::Debug::fmt(n, f),
            Self::Link(ref n) => fmt::Debug::fmt(n, f),
            Self::RawStatement(ref n)
            | Self::RawComment(ref n)
            | Self::Comment(ref n) => fmt::Debug::fmt(n, f),
        }
    }
}

/// Text nodes refer to a consecutive range of bytes.
#[derive(Eq, PartialEq)]
pub struct Text<'source> {
    source: &'source str,
    span: Range<usize>,
    line: Range<usize>,
}

impl<'source> Text<'source> {
    /// Create a new text node.
    pub fn new(
        source: &'source str,
        span: Range<usize>,
        line: Range<usize>,
    ) -> Self {
        Self { source, span, line }
    }
}

impl<'source> Lines for Text<'source> {
    fn lines(&self) -> &Range<usize> {
        &self.line
    }

    fn lines_mut(&mut self) -> &mut Range<usize> {
        &mut self.line
    }
}

impl<'source> Slice<'source> for Text<'source> {
    fn as_str(&self) -> &'source str {
        &self.source[self.span.start..self.span.end]
    }

    fn source(&self) -> &'source str {
        self.source
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
            .field("span", &self.span)
            .field("line", &self.line)
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
    /// Create a new text block.
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

impl<'source> Lines for TextBlock<'source> {
    fn lines(&self) -> &Range<usize> {
        self.text.lines()
    }

    fn lines_mut(&mut self) -> &mut Range<usize> {
        self.text.lines_mut()
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
            .field("line", self.lines())
            .finish()
    }
}

/// Indicates the kind of escaping using for raw
/// identifiers.
#[derive(Debug, Eq, PartialEq)]
pub enum RawIdType {
    /// Raw identifier in single quotes.
    Single,
    /// Raw identifier in double quotes.
    Double,
    /// Raw identifier in square brackets.
    Array,
}

/// Indicates the kind of path component.
#[derive(Debug, Eq, PartialEq)]
pub enum ComponentType {
    /// Parent reference type.
    Parent,
    /// Explicit this keyword type.
    ThisKeyword,
    /// Explicit this using dot slash notation.
    ThisDotSlash,
    /// Identifier path component.
    Identifier,
    /// Local identifier path component.
    LocalIdentifier,
    /// Raw identifier path component.
    RawIdentifier(RawIdType),
    /// Path delimiter.
    Delimiter,
}

/// Components form part of a path.
#[derive(Eq, PartialEq)]
pub struct Component<'source> {
    source: &'source str,
    kind: ComponentType,
    span: Range<usize>,
    value: Option<String>,
}

impl<'source> Component<'source> {
    /// Create a new component path.
    ///
    /// If a component path contains escape sequences an
    /// owned value should be given otherwise the component
    /// path will use the supplied span.
    pub fn new(
        source: &'source str,
        kind: ComponentType,
        span: Range<usize>,
        value: Option<String>,
    ) -> Self {
        Self {
            source,
            kind,
            span,
            value,
        }
    }

    /// Determine if this is the special `@root` component.
    pub fn is_root(&self) -> bool {
        self.as_str() == ROOT
    }

    /// Get the kind of this component.
    pub fn kind(&self) -> &ComponentType {
        &self.kind
    }

    /// The span for this component.
    pub fn span(&self) -> &Range<usize> {
        &self.span
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

    /// Get the underlying value for the path component.
    ///
    /// If an owned value has been given to this path component
    /// (which is necessary when the path component includes escape sequences)
    /// then a reference to the owned value is returned otherwise
    /// a string slice into the original template for the span
    /// assigned to this component path is returned.
    ///
    /// When performing lookup of values using a path a caller must use
    /// this function and **not** `as_str()` otherwise literal strings
    /// with escape sequences will not be respected.
    pub fn as_value(&self) -> &str {
        if let Some(ref value) = self.value {
            return value;
        }
        self.as_str()
    }
}

impl<'source> Slice<'source> for Component<'source> {
    fn as_str(&self) -> &'source str {
        &self.source[self.span().start..self.span().end]
    }

    fn source(&self) -> &'source str {
        self.source
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
            .field("kind", &self.kind)
            .field("span", &self.span)
            .finish()
    }
}

/// Path to a variable.
#[derive(Eq, PartialEq)]
pub struct Path<'source> {
    source: &'source str,
    components: Vec<Component<'source>>,
    parents: u8,
    explicit: bool,
    root: bool,
    open: Range<usize>,
    line: Range<usize>,
}

impl<'source> Path<'source> {
    /// Create a new path.
    pub fn new(
        source: &'source str,
        open: Range<usize>,
        line: Range<usize>,
    ) -> Self {
        Self {
            source,
            components: Vec::new(),
            parents: 0,
            explicit: false,
            root: false,
            open,
            line,
        }
    }

    /// Get the open span for the path.
    pub fn open_span(&self) -> &Range<usize> {
        &self.open
    }

    /// Add a component to this path.
    pub fn add_component(&mut self, part: Component<'source>) {
        self.components.push(part);
    }

    /// Get the path components.
    pub fn components(&self) -> &Vec<Component<'source>> {
        &self.components
    }

    /// Get the number of parent references.
    pub fn parents(&self) -> u8 {
        self.parents
    }

    /// Set the number of parent references.
    pub fn set_parents(&mut self, parents: u8) {
        self.parents = parents;
    }

    /// Flag this path as resolved relative to the root value.
    pub fn is_root(&self) -> bool {
        self.root
    }

    /// Set whether to resolve relative to a root value.
    pub fn set_root(&mut self, root: bool) {
        self.root = root;
    }

    /// Flag this path as an explicit scope reference (eg: `this` or `./`).
    pub fn is_explicit(&self) -> bool {
        self.explicit
    }

    /// Set whether this path is an explicit reference.
    pub fn set_explicit(&mut self, explicit: bool) {
        self.explicit = explicit;
    }

    /// Determine if the path components are empty.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Determine if the first component is a local identifier.
    pub fn is_local(&self) -> bool {
        return !self.components.is_empty()
            && self.components.first().unwrap().is_local();
    }

    /// Determine if this path is a simple identifier.
    pub fn is_simple(&self) -> bool {
        return self.components.len() == 1
            && self.components.first().unwrap().kind
                == ComponentType::Identifier;
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

impl<'source> Lines for Path<'source> {
    fn lines(&self) -> &Range<usize> {
        &self.line
    }

    fn lines_mut(&mut self) -> &mut Range<usize> {
        &mut self.line
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
            .field("line", &self.line)
            .finish()
    }
}

/// Parameter values can be used as arguments or hash values.
#[derive(Debug, Eq, PartialEq)]
pub enum ParameterValue<'source> {
    /// A parameter that should resolve to a runtime variable.
    Path(Path<'source>),
    /// A literal JSON value.
    Json {
        /// The literal JSON value.
        value: Value,
        /// The byte span for the value.
        span: Range<usize>,
        /// The line range for the value.
        line: Range<usize>,
    },
    /// A sub-expression to be invoked at runtime to determine the value.
    SubExpr(Call<'source>),
}

impl<'source> From<(Value, Range<usize>, Range<usize>)>
    for ParameterValue<'source>
{
    fn from(value: (Value, Range<usize>, Range<usize>)) -> Self {
        ParameterValue::Json {
            value: value.0,
            span: value.1,
            line: value.2,
        }
    }
}

impl<'source> Lines for ParameterValue<'source> {
    fn lines(&self) -> &Range<usize> {
        match *self {
            ParameterValue::Path(ref path) => path.lines(),
            ParameterValue::Json {
                value: _,
                span: _,
                ref line,
            } => line,
            ParameterValue::SubExpr(ref call) => call.lines(),
        }
    }

    fn lines_mut(&mut self) -> &mut Range<usize> {
        match *self {
            ParameterValue::Path(ref mut path) => path.lines_mut(),
            ParameterValue::Json {
                value: _,
                span: _,
                ref mut line,
            } => line,
            ParameterValue::SubExpr(ref mut call) => call.lines_mut(),
        }
    }
}

/// Call targets represent either a helper call, partial render or variable path.
///
/// To support dynamic partials call targets may also be sub-expressions.
#[derive(Debug, Eq, PartialEq)]
pub enum CallTarget<'source> {
    /// Path call target.
    Path(Path<'source>),
    /// Sub expression call target.
    SubExpr(Box<Call<'source>>),
}

impl<'source> CallTarget<'source> {
    /// Determine if this call target is empty.
    pub fn is_empty(&self) -> bool {
        match *self {
            Self::Path(ref path) => path.is_empty(),
            Self::SubExpr(ref call) => call.is_empty(),
        }
    }

    /// Get the open span for the call target.
    pub fn open_span(&self) -> &Range<usize> {
        match *self {
            Self::Path(ref path) => path.open_span(),
            Self::SubExpr(ref call) => call.open_span(),
        }
    }
}

impl<'source> Slice<'source> for CallTarget<'source> {
    fn as_str(&self) -> &'source str {
        match *self {
            Self::Path(ref path) => path.as_str(),
            Self::SubExpr(ref call) => call.as_str(),
        }
    }

    fn source(&self) -> &'source str {
        match *self {
            Self::Path(ref path) => path.source(),
            Self::SubExpr(ref call) => call.source(),
        }
    }
}

impl<'source> Lines for CallTarget<'source> {
    fn lines(&self) -> &Range<usize> {
        match *self {
            Self::Path(ref path) => path.lines(),
            Self::SubExpr(ref call) => call.lines(),
        }
    }

    fn lines_mut(&mut self) -> &mut Range<usize> {
        match *self {
            Self::Path(ref mut path) => path.lines_mut(),
            Self::SubExpr(ref mut call) => call.lines_mut(),
        }
    }
}

impl fmt::Display for CallTarget<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for CallTarget<'_> {
    fn default() -> Self {
        CallTarget::Path(Path::new("", 0..0, 0..0))
    }
}

/// Call is a variable interpolation, helper invocation or partial
/// render.
///
/// A call has zero or more arguments and optional hash parameters.
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
    parameters: HashMap<&'source str, ParameterValue<'source>>,
    line: Range<usize>,
}

impl<'source> Call<'source> {
    /// Create an open call.
    ///
    /// If it is correctly terminated the parser will call `exit()` to terminate
    /// the call statement.
    pub fn new(
        source: &'source str,
        open: Range<usize>,
        line: Range<usize>,
    ) -> Self {
        Self {
            source,
            partial: false,
            conditional: false,
            open,
            close: None,
            target: CallTarget::Path(Path::new(source, 0..0, 0..0)),
            arguments: Vec::new(),
            parameters: HashMap::new(),
            line,
        }
    }

    /// Determine if the target for this call is empty.
    pub fn is_empty(&self) -> bool {
        self.target.is_empty()
    }

    /// Get the call target.
    pub fn target(&self) -> &CallTarget<'source> {
        &self.target
    }

    /// Determine if a call target is available.
    pub fn has_target(&self) -> bool {
        self.target.as_str() != ""
    }

    /// Set the call target.
    pub fn set_target(&mut self, target: CallTarget<'source>) {
        self.target = target;
    }

    /// Add an argument to this call.
    pub fn add_argument(&mut self, arg: ParameterValue<'source>) {
        self.arguments.push(arg);
    }

    /// Get the list of arguments.
    pub fn arguments(&self) -> &Vec<ParameterValue<'source>> {
        &self.arguments
    }

    /// Add a hash parameter to this call.
    pub fn add_parameter(
        &mut self,
        key: &'source str,
        val: ParameterValue<'source>,
    ) {
        self.parameters.insert(key, val);
    }

    /// Get the map of hash parameters.
    pub fn parameters(
        &self,
    ) -> &HashMap<&'source str, ParameterValue<'source>> {
        &self.parameters
    }

    /// Determine if this call has the partial flag.
    pub fn is_partial(&self) -> bool {
        self.partial
    }

    /// Set the partial flag.
    pub fn set_partial(&mut self, partial: bool) {
        self.partial = partial;
    }

    /// Determine if this call has a conditional flag (the `else` keyword).
    pub fn is_conditional(&self) -> bool {
        self.conditional
    }

    /// Set the conditional flag.
    pub fn set_conditional(&mut self, conditional: bool) {
        self.conditional = conditional;
    }

    /// Determine if the content of this call should be escaped.
    pub fn is_escaped(&self) -> bool {
        // FIXME: ensure this is not `true` for raw blocks!
        !self.open().starts_with("{{{")
    }

    fn trim_before(&self) -> bool {
        self.open().ends_with(WHITESPACE)
    }

    fn trim_after(&self) -> bool {
        self.close().starts_with(WHITESPACE)
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

impl<'source> Lines for Call<'source> {
    fn lines(&self) -> &Range<usize> {
        &self.line
    }

    fn lines_mut(&mut self) -> &mut Range<usize> {
        &mut self.line
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
            .field("parameters", &self.parameters)
            .finish()
    }
}

/// Documents are abstract nodes that encapsulate a collection
/// of child nodes.
///
/// They are used as the root node of a compiled template.
#[derive(Eq, PartialEq)]
pub struct Document<'source>(pub &'source str, pub Vec<Node<'source>>);

impl<'source> Document<'source> {
    /// List of child nodes.
    pub fn nodes(&self) -> &Vec<Node<'source>> {
        &self.1
    }

    /// Mutable list of child nodes.
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
    line: Range<usize>,
}

impl<'source> Block<'source> {
    /// Create a new block.
    pub fn new(
        source: &'source str,
        open: Range<usize>,
        raw: bool,
        line: Range<usize>,
    ) -> Self {
        Self {
            source,
            nodes: Vec::new(),
            raw,
            open,
            close: None,
            call: Default::default(),
            conditionals: Vec::new(),
            line,
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
                    Node::Block(ref mut condition) => {
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

impl<'source> Lines for Block<'source> {
    fn lines(&self) -> &Range<usize> {
        &self.line
    }

    fn lines_mut(&mut self) -> &mut Range<usize> {
        &mut self.line
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
            .field("line", &self.line)
            .field("open", &self.open)
            .field("close", &self.close)
            .field("call", &self.call)
            .field("nodes", &self.nodes)
            .finish()
    }
}

/// Link node for wiki-style links.
#[derive(Eq, PartialEq)]
pub struct Link<'source> {
    source: &'source str,
    open: Range<usize>,
    close: Option<Range<usize>>,
    line: Range<usize>,
    href_span: Range<usize>,
    label_span: Range<usize>,
    title_span: Range<usize>,

    // Owned value when escape sequences are detected
    href: Option<String>,
    label: Option<String>,
    title: Option<String>,
}

impl<'source> Link<'source> {
    /// Create a new link.
    pub fn new(
        source: &'source str,
        open: Range<usize>,
        line: Range<usize>,
    ) -> Self {
        Self {
            source,
            href_span: open.end..open.end,
            label_span: open.end..open.end,
            title_span: open.end..open.end,
            open,
            line,
            close: None,
            href: None,
            label: None,
            title: None,
        }
    }

    /// Get the link href.
    ///
    /// If an owned value has been set it is preferred.
    pub fn href(&self) -> &str {
        if let Some(ref href) = self.href {
            return href;
        }
        &self.source[self.href_span.start..self.href_span.end]
    }

    /// Get the link label.
    ///
    /// If the label is the empty string the href will be used instead.
    ///
    /// If an owned value has been set it is preferred.
    pub fn label(&self) -> &str {
        let lbl = if let Some(ref label) = self.label {
            return label;
        } else {
            &self.source[self.label_span.start..self.label_span.end]
        };

        if lbl.is_empty() {
            self.href()
        } else {
            lbl
        }
    }

    /// Get the link title.
    ///
    /// If the title is the empty string the label will be used instead.
    ///
    /// If an owned value has been set it is preferred.
    pub fn title(&self) -> &str {
        let title = if let Some(ref title) = self.title {
            return title;
        } else {
            &self.source[self.title_span.start..self.title_span.end]
        };

        if title.is_empty() {
            self.label()
        } else {
            title
        }
    }

    /// Get the span for the href.
    pub fn href_span(&self) -> &Range<usize> {
        &self.href_span
    }

    /// Get the span for the label.
    pub fn label_span(&self) -> &Range<usize> {
        &self.label_span
    }

    /// Get the span for the title.
    pub fn title_span(&self) -> &Range<usize> {
        &self.title_span
    }

    /// Update the end of the href span.
    pub fn href_end(&mut self, end: usize) {
        self.href_span.end = end;
    }

    /// Update the start of the label span.
    pub fn label_start(&mut self, start: usize) {
        self.label_span.start = start;
    }

    /// Update the end of the label span.
    pub fn label_end(&mut self, end: usize) {
        self.label_span.end = end;
    }

    /// Update the start of the title span.
    pub fn title_start(&mut self, start: usize) {
        self.title_span.start = start;
    }

    /// Update the end of the title span.
    pub fn title_end(&mut self, end: usize) {
        self.title_span.end = end;
    }

    /// Set an owned value for the href.
    ///
    /// Only available when the parser detects escape sequences
    /// in the input.
    pub fn set_href(&mut self, value: String) {
        self.href = Some(value);
    }

    /// Set an owned value for the label.
    ///
    /// Only available when the parser detects escape sequences
    /// in the input.
    pub fn set_label(&mut self, value: String) {
        self.label = Some(value);
    }

    /// Set an owned value for the title.
    ///
    /// Only available when the parser detects escape sequences
    /// in the input.
    pub fn set_title(&mut self, value: String) {
        self.title = Some(value);
    }
}

impl<'source> Slice<'source> for Link<'source> {
    fn as_str(&self) -> &'source str {
        let close = self.close.clone().unwrap_or(0..self.open.len());
        &self.source[self.open.start..close.end]
    }

    fn source(&self) -> &'source str {
        self.source
    }
}

impl<'source> Lines for Link<'source> {
    fn lines(&self) -> &Range<usize> {
        &self.line
    }

    fn lines_mut(&mut self) -> &mut Range<usize> {
        &mut self.line
    }
}

impl<'source> Element<'source> for Link<'source> {
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
        self.close = Some(span);
    }
}

impl fmt::Display for Link<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for Link<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Link")
            .field("open", &self.open)
            .field("close", &self.close)
            .field("href", &self.href)
            .field("label", &self.label)
            .finish()
    }
}
