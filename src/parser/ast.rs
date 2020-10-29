use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

use serde_json::Value;

static WHITESPACE: &str = "~";

pub static ROOT: &str = "@root";
pub static FIRST: &str = "@first";
pub static LAST: &str = "@last";
pub static KEY: &str = "@key";
pub static INDEX: &str = "@index";
pub static LEVEL: &str = "@level";

#[derive(Eq, PartialEq)]
pub enum Node<'source> {
    Text(Text<'source>),
    Statement(Call<'source>),
    Block(Block<'source>),
}

impl<'source> Node<'source> {
    pub fn as_str(&self) -> &'source str {
        match *self {
            Self::Text(ref n) => n.as_str(),
            Self::Statement(ref n) => n.as_str(),
            Self::Block(ref n) => n.as_str(),
        }
    }

    pub fn trim_before(&self) -> bool {
        match *self {
            Self::Text(_) => false,
            Self::Statement(ref n) => n.open().ends_with(WHITESPACE),
            Self::Block(ref n) => {
                let open = n.open();
                open.len() > 2 && WHITESPACE == &open[2..3]
            }
        }
    }

    pub fn trim_after(&self) -> bool {
        match *self {
            Self::Text(_) => false,
            Self::Statement(ref n) => n.close().starts_with(WHITESPACE),
            Self::Block(ref n) => {
                let open = n.open();
                open.len() > 2 && WHITESPACE == &open[2..3]
            }
        }
    }
}

impl fmt::Display for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Text(ref n) => n.fmt(f),
            Self::Statement(ref n) => n.fmt(f),
            Self::Block(ref n) => n.fmt(f),
        }
    }
}

impl fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Text(ref t) => {
                f.debug_struct("Text").field("source", &t.as_str()).finish()
            }
            Self::Block(ref b) => fmt::Debug::fmt(b, f),
            Self::Statement(ref s) => fmt::Debug::fmt(s, f),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Text<'source>(pub &'source str, pub Range<usize>);

impl<'source> Text<'source> {
    pub fn as_str(&self) -> &'source str {
        &self.0[self.1.start..self.1.end]
    }
}

impl fmt::Display for Text<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ComponentType {
    Parent,
    ThisKeyword,
    ThisDotSlash,
    Identifier,
    LocalIdentifier,
    Delimiter,
}

#[derive(Eq, PartialEq)]
pub struct Component<'source>(
    pub &'source str,
    pub ComponentType,
    pub Range<usize>,
);

impl<'source> Component<'source> {
    pub fn is_root(&self) -> bool {
        self.as_str() == ROOT
    }

    pub fn kind(&self) -> &ComponentType {
        &self.1
    }

    pub fn span(&self) -> &Range<usize> {
        &self.2
    }

    pub fn is_local(&self) -> bool {
        &ComponentType::LocalIdentifier == self.kind()
    }

    pub fn is_identifier(&self) -> bool {
        &ComponentType::Identifier == self.kind()
    }

    pub fn is_explicit(&self) -> bool {
        &ComponentType::ThisKeyword == self.kind()
            || self.is_explicit_dot_slash()
    }

    pub fn is_explicit_dot_slash(&self) -> bool {
        &ComponentType::ThisDotSlash == self.kind()
    }

    pub fn as_str(&self) -> &'source str {
        &self.0[self.span().start..self.span().end]
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

    pub fn as_str(&self) -> &'source str {
        if !self.components.is_empty() {
            let first = self.components.first().unwrap();
            let last = self.components.last().unwrap();
            &self.source[first.span().start..last.span().end]
        } else {
            ""
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

    pub fn is_simple(&self) -> bool {
        return self.components.len() == 1
            && self.components.first().unwrap().1 == ComponentType::Identifier;
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

#[derive(Debug, Eq, PartialEq)]
pub enum ParameterValue<'source> {
    Path(Path<'source>),
    Json(Value),
    SubExpr(Call<'source>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum CallTarget<'source> {
    Path(Path<'source>),
    SubExpr(Box<Call<'source>>),
}

impl<'source> CallTarget<'source> {
    // FIXME!
    pub fn is_empty(&self) -> bool {
        match *self {
            Self::Path(ref path) => path.is_empty(),
            Self::SubExpr(ref call) => {
                println!("Checking empty on sub expression...");
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

#[derive(Default, Eq, PartialEq)]
pub struct Call<'source> {
    // Raw source input.
    source: &'source str,
    partial: bool,
    open: Range<usize>,
    close: Range<usize>,
    target: CallTarget<'source>,
    arguments: Vec<ParameterValue<'source>>,
    hash: HashMap<&'source str, ParameterValue<'source>>,
}

impl<'source> Call<'source> {
    pub fn new(
        source: &'source str,
        partial: bool,
        open: Range<usize>,
        close: Range<usize>,
    ) -> Self {
        Self {
            source,
            partial,
            open,
            close,
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

    //pub fn as_str(&self) -> &'source str {
        //&self.source[self.open.start..self.close.end]
    //}

    pub fn as_str(&self) -> &'source str {
        &self.source[self.open.end..self.close.start]
    }

    pub fn open(&self) -> &'source str {
        &self.source[self.open.start..self.open.end]
    }

    pub fn close(&self) -> &'source str {
        &self.source[self.close.start..self.close.end]
    }

    pub fn is_partial(&self) -> bool {
        self.partial
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

#[derive(Debug, Eq, PartialEq)]
pub enum BlockType {
    Root,
    RawBlock,     // {{{{raw}}}}{{expr}}{{{{/raw}}}}
    RawStatement, // \{{expr}}
    RawComment,   // {{!-- {{expr}} --}}
    Comment,      // {{! comment }}
    Scoped,       // {{#> partial|helper}}{{/partial|helper}}
}

impl Default for BlockType {
    fn default() -> Self {
        Self::Root
    }
}

#[derive(Default, Eq, PartialEq)]
pub struct Block<'source> {
    // Raw source input.
    source: &'source str,
    kind: BlockType,
    nodes: Vec<Node<'source>>,
    open: Option<Range<usize>>,
    close: Option<Range<usize>>,
    call: Call<'source>,
}

impl<'source> Block<'source> {
    pub fn new(
        source: &'source str,
        kind: BlockType,
        open: Option<Range<usize>>,
    ) -> Self {
        Self {
            source,
            kind,
            nodes: Vec::new(),
            open,
            close: None,
            call: Default::default(),
        }
    }

    pub fn call(&self) -> &Call<'source> {
        &self.call
    }

    pub fn set_call(&mut self, call: Call<'source>) {
        self.call = call;
    }

    /// The name of this block.
    pub fn name(&self) -> Option<&'source str> {
        match self.call.target() {
            CallTarget::Path(ref path) => {
                if path.is_simple() {
                    let id = path.components().first().unwrap(); 
                    Some(id.as_str())
                } else { None }
            }
            CallTarget::SubExpr(_) => None
        } 
    }

    pub(crate) fn exit(&mut self, span: Range<usize>) {
        self.close = Some(span);
    }

    pub fn as_str(&self) -> &'source str {
        match self.kind() {
            BlockType::Root => self.source,
            _ => {
                let open = self.open.clone().unwrap_or(0..0);
                let close = self.close.clone().unwrap_or(0..self.source.len());
                &self.source[open.start..close.end]
            }
        }
    }

    pub fn open(&self) -> &'source str {
        if let Some(ref open) = self.open {
            &self.source[open.start..open.end]
        } else {
            ""
        }
    }

    pub fn between(&self) -> &'source str {
        let open = self.open.clone().unwrap_or(0..0);
        let close = self.close.clone().unwrap_or(0..self.source.len());
        &self.source[open.end..close.start]
    }

    pub fn close(&self) -> &'source str {
        if let Some(ref close) = self.close {
            &self.source[close.start..close.end]
        } else {
            ""
        }
    }

    pub fn push(&mut self, node: Node<'source>) {
        self.nodes.push(node);
    }

    pub fn kind(&self) -> &BlockType {
        &self.kind
    }

    pub fn nodes(&self) -> &'source Vec<Node> {
        &self.nodes
    }

    pub fn trim_before_close(&self) -> bool {
        match self.kind {
            BlockType::Scoped => {
                let close = self.close();
                println!("Got before close {:?}", close);
                close.len() > 2 && WHITESPACE == &close[2..3]
            }
            _ => false,
        }
    }

    pub fn trim_after_close(&self) -> bool {
        match self.kind {
            BlockType::Scoped => {
                let close = self.call.close();
                close.len() > 2 && WHITESPACE == &close[0..1]
            }
            _ => false,
        }
    }
}

impl fmt::Display for Block<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            BlockType::Root => write!(f, "{}", self.source),
            _ => {
                for t in self.nodes() {
                    t.fmt(f)?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Debug for Block<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("kind", &self.kind)
            .field("open", &self.open)
            .field("close", &self.close)
            .field("call", &self.call)
            .field("nodes", &self.nodes)
            .finish()
    }
}
