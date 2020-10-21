use std::fmt;
use logos::Logos;
use regex::Regex;

pub fn parse_block_name(value: &str) -> String {
    let re = Regex::new(r"\{\{\{?#?>?/?\s*([^}]*)\s*\}?\}\}").unwrap();
    let cap = re.captures_iter(value).next().unwrap();
    cap[1].to_string()
}

#[derive(Logos, Debug, PartialEq)]
#[logos(subpattern simple_name = r"[a-zA-Z0-9_-]+")]
pub(crate) enum Token {

    #[regex(r"[\\]?\{\{(\{|[^!])?>?\s*[^}]+\s*\}?\}\}", |lex| lex.slice().to_string())]
    Expression(String),

    #[regex(r"\{\{\{\{\s*raw\s*\}\}\}\}", |lex| lex.slice().to_string())]
    StartRawBlock(String),

    #[regex(r"\{\{\{\{\s*/raw\s*\}\}\}\}", |lex| lex.slice().to_string())]
    EndRawBlock(String),

    #[regex(r"\r?\n", |lex| lex.slice().to_string())]
    Newline(String),

    #[regex(r"\{\{#>?\s*(?&simple_name)\s*\}\}", |lex| lex.slice().to_string())]
    StartBlock(String),

    #[regex(r"\{\{/\s*(?&simple_name)\s*\}\}", |lex| lex.slice().to_string())]
    EndBlock(String),

    //#[regex(r"(\{\{!(--)?|<!--)", |lex| lex.slice().to_string())]
    #[token("{{!--", |lex| lex.slice().to_string())]
    StartCommentBlock(String),

    #[regex(r"((--)?\}\}|-->)", |lex| lex.slice().to_string())]
    EndCommentBlock(String),

    #[regex(r"[^\n{]+", |lex| lex.slice().to_string())]
    Text(String),

    #[error]
    Error,
}

#[derive(Debug)]
pub(crate) struct SourceInfo {
    pub(crate) line: usize, 
    pub(crate) span: logos::Span,
}

#[derive(Debug)]
pub(crate) struct Expression {
    pub(crate) info: SourceInfo,
    pub(crate) value: String,
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.value)
    }
}

#[derive(Debug)]
pub(crate) struct Text {
    pub(crate) info: SourceInfo,
    pub(crate) value: String,
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.value)
    }
}

#[derive(Debug)]
pub(crate) enum AstToken {
    Expression(Expression),
    Text(Text),
    Block(Block),
    Newline(Text),
}

impl fmt::Display for AstToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Expression(ref t) => t.fmt(f),
            Self::Block(ref t) => t.fmt(f),
            Self::Text(ref t)
                | Self::Newline(ref t) => t.fmt(f),
        }
    }
}

#[derive(Debug)]
pub(crate) enum BlockType {
    Root,
    Raw,
    Comment,
    Named(String),
}

impl Default for BlockType {
    fn default() -> Self {
        Self::Root
    }
}

#[derive(Debug, Default)]
pub(crate) struct Block {
    pub(crate) block_type: BlockType, 
    pub(crate) tokens: Vec<AstToken>,
    pub(crate) open: Option<String>,
    pub(crate) close: Option<String>,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self {block_type, tokens: Vec::new(), open: None, close: None}
    }

    pub fn new_named(value: String) -> Self {
        let name = parse_block_name(&value);
        let mut block = Block::new(BlockType::Named(name));
        block.open = Some(value);
        block
    }

    pub fn push(&mut self, token: AstToken) {
        self.tokens.push(token); 
    }

    pub fn is_raw(&self) -> bool {
        match self.block_type {
            BlockType::Raw => true,
            _=> false
        }
    }

    pub fn is_named(&self) -> bool {
        match self.block_type {
            BlockType::Named(_) => true,
            _=> false
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref s) = self.open {
            write!(f, "{}", s)?;
        }
        for t in self.tokens.iter() {
            t.fmt(f)?;
        }
        if let Some(ref s) = self.close {
            write!(f, "{}", s)?;
        }
        Ok(())
    }
}
