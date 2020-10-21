use std::fmt;
use logos::Logos;

use crate::{Error, Result};

#[derive(Logos, Debug, PartialEq)]
#[logos(subpattern simple_name = r"[a-zA-Z0-9_-]+")]
#[logos(subpattern path = r"[@a-zA-Z0-9._-]+")]
enum Token {

    #[regex(r"[\\]?\{\{\{?>?\s*(?&path)\s*\}?\}\}", |lex| lex.slice().to_string())]
    Expression(String),

    //.*\{\{\{\{/raw\}\}\}\}

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

    #[regex(r"[^\n{]+", |lex| lex.slice().to_string())]
    Text(String),

    #[error]
    Error,
}

#[derive(Debug)]
struct SourceInfo {
    line: usize, 
    span: logos::Span,
}

#[derive(Debug)]
struct Expression {
    info: SourceInfo,
    value: String,
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.value)
    }
}

#[derive(Debug)]
struct Text {
    info: SourceInfo,
    value: String,
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.value)
    }
}

#[derive(Debug)]
enum AstToken {
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
enum BlockType {
    Root,
    Raw,
    Named(String),
}

impl Default for BlockType {
    fn default() -> Self {
        Self::Root
    }
}

#[derive(Debug, Default)]
struct Block {
    block_type: BlockType, 
    tokens: Vec<AstToken>,
    open: Option<String>,
    close: Option<String>,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self {block_type, tokens: Vec::new(), open: None, close: None}
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

#[derive(Debug)]
pub struct Template {
    ast: Block,
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ast.fmt(f)
    }
}

impl Template {

    pub fn compile(s: &str) -> Result<Template> {
        let lex = Token::lexer(s);
        let mut ast = Block::new(BlockType::Root);
        let mut stack: Vec<Block> = vec![];
        let mut line = 0;

        let mut last: Option<Block> = None;

        for (token, span) in lex.spanned().into_iter() {

            let len = stack.len();
            let mut current = if stack.is_empty() {
                &mut ast
            } else {
                stack.get_mut(len - 1).unwrap()
            };

            if let Some(last) = last.take() {
                current.push(AstToken::Block(last));
            }

            println!("{:?}", token);

            let info = SourceInfo {line, span};
            match token {
                Token::Expression(value) => {
                    current.push(AstToken::Expression(Expression {info, value}));
                }
                Token::Text(value) => {
                    current.push(AstToken::Text(Text {info, value}));
                }
                Token::StartRawBlock(value) => {
                    let mut block = Block::new(BlockType::Raw);
                    block.open = Some(value);
                    stack.push(block);
                }
                Token::EndRawBlock(value) => {
                    last = stack.pop();
                    if let Some(ref mut block) = last {
                        if !block.is_raw() {
                            return Err(Error::BadEndRawBlock)
                        }

                        block.close = Some(value);
                    } else {
                        return Err(Error::BadEndBlock)
                    }
                }
                Token::Newline(value) => {
                    current.push(AstToken::Newline(Text {info, value}));
                    line = line + 1; 
                }
                Token::StartBlock(value) => {
                    // TODO: parse block name
                    let mut block = Block::new(BlockType::Named("nested".to_string()));
                    block.open = Some(value);
                    stack.push(block);

                }
                Token::EndBlock(value) => {
                    // TODO: check the end block name matches
                    last = stack.pop();
                    if let Some(ref mut block) = last {
                        //if !block.is_raw() {
                            //return Err(Error::BadEndRawBlock)
                        //}

                        block.close = Some(value);
                    } else {
                        return Err(Error::BadEndBlock)
                    }
                }
                Token::Error => {
                    return Err(Error::InvalidToken);
                }
            }
        }

        Ok(Template {ast})
    }
}

