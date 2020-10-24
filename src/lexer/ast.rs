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
pub enum BlockType {
    Root,
    /// Text blocks use the `open` range for the entire slice.
    Text,
    RawBlock,
    RawStatement,
    RawComment,
    Scoped,
}

impl Default for BlockType {
    fn default() -> Self { Self::Root }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Block<'source> {
    // Raw source input.
    source: &'source str,
    block_type: BlockType,
    blocks: Vec<Block<'source>>,
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
            blocks: Vec::new(),
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

    pub fn between(&self) -> &'source str {
        let open = self.open.clone().unwrap_or(0..0);
        let close = self.close.clone().unwrap_or(0..0);
        println!("Rendering between!!");
        println!("{:?}", open);
        println!("{:?}", close);
        &self.source[open.end..close.start]
    }

    pub fn close(&self) -> &'source str {
        if let Some(ref close) = self.close {
            &self.source[close.start..close.end]
        } else {
            ""
        }
    }

    pub fn push(&mut self, token: Block<'source>) {
        self.blocks.push(token);
    }

    pub fn block_type(&self) -> &BlockType {
        &self.block_type
    }

    pub fn blocks(&self) -> &'source Vec<Block> {
        &self.blocks
    }
}

impl<'source> From<Text<'source>> for Block<'source> {
    fn from(txt: Text<'source>) -> Block<'source> {
        Block::new(
            txt.0,
            BlockType::Text,
            Some(txt.1),
        ) 
    }
}

impl fmt::Display for Block<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.block_type() {
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
