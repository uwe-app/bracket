use std::fmt;
use std::ops::Range;

use crate::lexer::parser;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceInfo {
    //pub line: Range<usize>,
    pub span: Range<usize>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Text<'source>(pub &'source str, pub Range<usize>);

impl<'source> Text<'source> {
    pub fn as_str(&self) -> &'source str {
        &self.0[self.1.start..self.1.end]
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct RawComment<'source> {
    // Raw source input.
    source: &'source str,
    /// Range of the open tag.
    open: Range<usize>,
    /// Range of the close tag.
    close: Range<usize>,
}

impl<'source> RawComment<'source> {
    pub fn as_str(&self) -> &'source str {
        &self.source[self.open.start..self.close.end]
    }
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct Statement<'source> {
    tokens: Vec<Token<'source>>,
}

impl Statement<'_> {
    pub fn tokens(&self) -> &Vec<Token> {
        &self.tokens
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Expr<'source> {
    info: SourceInfo,
    value: &'source str,
}

impl<'source> Expr<'source> {
    pub fn new(info: SourceInfo, value: &'source str) -> Self {
        Self { info, value }
    }

    pub fn is_raw(&self) -> bool {
        if !self.value.is_empty() {
            let first = self.value.chars().nth(0).unwrap();
            return first == '\\';
        }
        false
    }

    pub fn escapes(&self) -> bool {
        !self.value.starts_with("{{{")
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn info(&self) -> &SourceInfo {
        &self.info
    }
}

impl fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Token<'source> {
    Text(Text<'source>),
    RawComment(RawComment<'source>),

    Expression(Expr<'source>),
    Block(Block<'source>),
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Text(ref t) => write!(f, "{}", t.as_str()),
            Self::RawComment(ref t) => write!(f, "{}", t.as_str()),
            Self::Expression(ref t) => t.fmt(f),
            Self::Block(ref t) => t.fmt(f),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum BlockType {
    Root,
    Raw,
    RawStatement,
    RawComment,
    Scoped,
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
    block_type: BlockType,
    tokens: Vec<Token<'source>>,
    open: Option<Range<usize>>,
    close: Option<Range<usize>>,
}

impl<'source> Block<'source> {
    pub fn new(
        source: &'source str,
        block_type: BlockType,
        open: Option<Range<usize>>,
    ) -> Self {
        Self {
            source,
            block_type,
            tokens: Vec::new(),
            open,
            close: None,
        }
    }

    pub(crate) fn exit(&mut self, span: Range<usize>) {
        self.close = Some(span);
    }

    pub fn as_str(&self) -> &'source str {
        match self.block_type() {
            BlockType::Root => self.source,
            _ => {
                let open = self.open.clone().unwrap_or(0..0);
                let close = self.close.clone().unwrap_or(0..0);
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

    pub fn close(&self) -> &'source str {
        if let Some(ref close) = self.close {
            &self.source[close.start..close.end]
        } else {
            ""
        }
    }

    pub fn push(&mut self, token: Token<'source>) {
        self.tokens.push(token);
    }

    pub fn block_type(&self) -> &BlockType {
        &self.block_type
    }

    pub fn tokens(&self) -> &'source Vec<Token> {
        &self.tokens
    }

    pub fn is_raw(&self) -> bool {
        match self.block_type {
            BlockType::Raw => true,
            _ => false,
        }
    }
}

impl fmt::Display for Block<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.block_type() {
            BlockType::Root => write!(f, "{}", self.source),
            _ => {
                for t in self.tokens() {
                    t.fmt(f)?;
                }
                Ok(())
            }
        }
    }
}
