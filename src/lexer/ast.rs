use crate::lexer::{parser, SourceInfo};
use std::fmt;

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
pub struct Text<'source> {
    pub info: SourceInfo,
    pub value: &'source str,
}

impl fmt::Display for Text<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Token<'source> {
    Expression(Expr<'source>),
    Text(Text<'source>),
    Block(Block<'source>),
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Expression(ref t) => t.fmt(f),
            Self::Block(ref t) => t.fmt(f),
            Self::Text(ref t) => t.fmt(f),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum BlockType {
    Root,
    Raw,
    Comment,
    // TODO: use &'source ref
    Named(String),
}

impl Default for BlockType {
    fn default() -> Self {
        Self::Root
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Block<'source> {
    block_type: BlockType,
    tokens: Vec<Token<'source>>,
    pub(crate) open: Option<&'source str>,
    pub(crate) close: Option<&'source str>,

    // Used to coalesce content for raw blocks
    info: Option<SourceInfo>,
    value: Option<&'source str>,
}

impl<'source> Block<'source> {
    pub fn new(block_type: BlockType) -> Self {
        Self {
            block_type,
            tokens: Vec::new(),
            open: None,
            close: None,
            info: None,
            value: None,
        }
    }

    pub fn new_named(value: &'source str) -> Self {
        let name = parser::block_name(&value);
        let mut block = Block::new(BlockType::Named(name));
        block.open = Some(value);
        block
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

    pub fn value(&self) -> Option<&'source str> {
        self.value
    }

    /*
    pub fn terminated(&self) -> bool {
        self.close.is_some()
    }
    */

    pub fn replace(&mut self, info: SourceInfo, value: &'source str) {
        self.info = Some(info);
        self.value = Some(value);
    }

    /*
    pub fn tokens_mut(&mut self) -> &'source mut Vec<Token> {
        &mut self.tokens
    }
    */

    pub fn is_raw(&self) -> bool {
        match self.block_type {
            BlockType::Raw => true,
            _ => false,
        }
    }

    pub fn is_named(&self) -> bool {
        match self.block_type {
            BlockType::Named(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Block<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref s) = self.open {
            write!(f, "{}", s)?;
        }
        if let Some(val) = self.value {
            write!(f, "{}", val)?;
        } else {
            for t in self.tokens.iter() {
                t.fmt(f)?;
            }
        }
        if let Some(ref s) = self.close {
            write!(f, "{}", s)?;
        }
        Ok(())
    }
}
