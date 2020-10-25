use std::fmt;
use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug, Eq, PartialEq)]
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
pub struct Path(pub Vec<Range<usize>>);

#[derive(Debug, Eq, PartialEq)]
pub enum ParameterValue {
    Path(Path),
    Literal,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Call<'source> {
    // Raw source input.
    source: &'source str,
    span: Range<usize>,
    name: Path,
    arguments: Vec<ParameterValue>,
    hash: HashMap<String, ParameterValue>,
}

impl<'source> Call<'source> {
    pub fn new(
        source: &'source str,
        span: Range<usize>,
        name: Path,
        arguments: Option<Vec<ParameterValue>>,
        hash: Option<HashMap<String, ParameterValue>>) -> Self {
        Self {
            source,
            span,
            name,
            arguments: arguments.unwrap_or(Default::default()),
            hash: hash.unwrap_or(Default::default()),
        } 
    }

    pub fn as_str(&self) -> &'source str {
        &self.source[self.span.start..self.span.end]
    }
}

impl fmt::Display for Call<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum BlockType {
    Root,
    RawBlock,       // {{{{raw}}}}{{expr}}{{{{/raw}}}}
    RawStatement,   // \{{expr}}
    RawComment,     // {{!-- {{expr}} --}}
    Comment,        // {{! comment }}
    Scoped,         // {{#> partial|helper}}{{/partial|helper}}
}

impl Default for BlockType {
    fn default() -> Self {
        Self::Root
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Block<'source> {
    // Raw source input.
    source: &'source str,
    kind: BlockType,
    blocks: Vec<Node<'source>>,
    open: Option<Range<usize>>,
    close: Option<Range<usize>>,
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
            blocks: Vec::new(),
            open,
            close: None,
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

    pub fn push(&mut self, token: Node<'source>) {
        self.blocks.push(token);
    }

    pub fn kind(&self) -> &BlockType {
        &self.kind
    }

    pub fn blocks(&self) -> &'source Vec<Node> {
        &self.blocks
    }
}

impl fmt::Display for Block<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            BlockType::Root => write!(f, "{}", self.source),
            _ => {
                for t in self.blocks() {
                    t.fmt(f)?;
                }
                Ok(())
            }
        }
    }
}
