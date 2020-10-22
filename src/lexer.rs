use logos::Logos;
use regex::Regex;
use std::fmt;
use std::ops::Range;

pub fn parse_block_name(value: &str) -> String {
    let re = Regex::new(r"\{\{\{?#?>?/?\s*([^}]*)\s*\}?\}\}").unwrap();
    let cap = re.captures_iter(value).next().unwrap();
    cap[1].to_string()
}

#[derive(Logos, Debug, PartialEq)]
#[logos(subpattern simple_name = r"[a-zA-Z0-9_-]+")]
pub(crate) enum Token {
    #[regex(r"[\\]?\{\{\{?[^!]>?\s*[^}]+\s*\}?\}\}", |lex| lex.slice().to_string())]
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

    #[regex(r"(\{\{!(--)?|<!--)", |lex| lex.slice().to_string())]
    StartCommentBlock(String),

    #[regex(r"((--)?\}\}|-->)", |lex| lex.slice().to_string())]
    EndCommentBlock(String),

    #[regex(r"[^\n{]", |lex| lex.slice().to_string())]
    Text(String),

    #[error]
    Error,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SourceInfo {
    pub line: Range<usize>,
    pub span: logos::Span,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Expression {
    pub info: SourceInfo,
    pub value: String,
}

impl Expression {
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
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.value)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Text {
    pub info: SourceInfo,
    pub value: String,
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.value)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum AstToken {
    Expression(Expression),
    Text(Text),
    Block(Block),
    //Newline(Text),
}

impl fmt::Display for AstToken {
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
    Named(String),
}

impl Default for BlockType {
    fn default() -> Self {
        Self::Root
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Block {
    pub(crate) block_type: BlockType,
    tokens: Vec<AstToken>,
    pub(crate) open: Option<String>,
    pub(crate) close: Option<String>,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self {
            block_type,
            tokens: Vec::new(),
            open: None,
            close: None,
        }
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

    pub fn tokens(&self) -> &Vec<AstToken> {
        &self.tokens 
    }

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

    /// Concatenate consecutive text tokens.
    ///
    /// The lexer needs to read unrecognised characters with a low
    /// priority (1) so that matching works as expected but it makes
    /// sense for us to normalize consecutive text tokens as we lex.
    pub fn add_text(&mut self, info: SourceInfo, value: String) {
        if self.tokens.is_empty() {
            self.tokens.push(AstToken::Text(Text { value, info }));
        } else {
            let len = self.tokens.len();
            let last = self.tokens.get_mut(len - 1).unwrap();
            match last {
                AstToken::Text(ref mut txt) => {
                    txt.value.push_str(&value);
                    txt.info.span.end = info.span.end;
                    txt.info.line.end = info.line.end;
                }
                _ => {
                    self.tokens.push(AstToken::Text(Text { value, info }));
                }
            }
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
