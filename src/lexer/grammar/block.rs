use logos::{Lexer, Logos, Span};

use super::{LexToken, modes::{self,Extras}, statement, raw_block, raw_comment};

pub type Token = Box<dyn LexToken>;

#[derive(Logos, Clone, Debug, Eq, PartialEq)]
#[logos(extras = Extras)]
pub enum Outer {

    #[regex(r"\{\{\{\{\s*raw\s*\}\}\}\}")]
    StartRawBlock,

    #[regex(r"\{\{!--")]
    StartRawComment,

    #[regex(r"\{\{\{?")]
    StartStatement,

    #[regex(r".")]
    Text,

    #[regex("\r?\n", |lex| {
        lex.extras.lines += 1;
    })]
    Newline,

    #[error]
    Error,
}

impl LexToken for Outer {
    fn is_text(&self) -> bool {
        self == &Outer::Text
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlockToken {
    Block(Outer, Span),
    RawBlock(raw_block::Inner, Span),
    RawComment(raw_comment::Inner, Span),
    Statement(statement::Inner, Span),
}

impl BlockToken {
    fn span(&self) -> &Span {
        match self {
            BlockToken::Block(_, ref span) => span,
            BlockToken::RawBlock(_, ref span) => span,
            BlockToken::RawComment(_, ref span) => span,
            BlockToken::Statement(_, ref span) => span,
        } 
    }

    fn is_text(&self) -> bool {
        match self {
            BlockToken::Block(ref t, _) => t == &Outer::Text,
            BlockToken::RawBlock(ref t, _) => t == &raw_block::Inner::Text,
            BlockToken::RawComment(ref t, _) => t == &raw_comment::Inner::Text,
            BlockToken::Statement(ref t, _) => false,
        } 
    }
}

//pub struct BlockToken(pub Box<dyn LexToken>, pub Span);

enum Modes<'source> {
    Outer(Lexer<'source, Outer>),
    RawBlock(Lexer<'source, raw_block::Inner>),
    RawComment(Lexer<'source, raw_comment::Inner>),
    Statement(Lexer<'source, statement::Inner>),
}

impl<'source> Modes<'source> {
    fn new(s: &'source str) -> Self {
        Self::Outer(Outer::lexer(s))
    }
}

struct ModeBridge<'source> {
    mode: Modes<'source>,
}

// Clones as we switch between modes
impl<'source> Iterator for ModeBridge<'source> {
    type Item = BlockToken;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.mode {
            Modes::RawBlock(inner) => {
                let result = inner.next();
                let span = inner.span();
                if let Some(token) = result {
                    if raw_block::Inner::End  == token {
                        self.mode = Modes::Outer(inner.to_owned().morph());
                    }
                    let t = BlockToken::RawBlock(token, span);
                    Some(t)
                } else {
                    None
                }
            }
            Modes::RawComment(inner) => {
                let result = inner.next();
                let span = inner.span();
                if let Some(token) = result {
                    if raw_comment::Inner::End  == token {
                        self.mode = Modes::Outer(inner.to_owned().morph());
                    }
                    let t = BlockToken::RawComment(token, span);
                    Some(t)
                } else {
                    None
                }
            }
            Modes::Statement(inner) => {
                let result = inner.next();
                let span = inner.span();
                if let Some(token) = result {
                    if statement::Inner::End  == token {
                        self.mode = Modes::Outer(inner.to_owned().morph());
                    }
                    let t = BlockToken::Statement(token, span);
                    Some(t)
                } else {
                    None
                }
            }
            Modes::Outer(outer) => {
                let result = outer.next();
                let span = outer.span();
                if let Some(token) = result {
                    if Outer::StartRawBlock == token {
                        self.mode = Modes::RawBlock(outer.to_owned().morph());
                    } else if Outer::StartRawComment == token {
                        self.mode = Modes::RawComment(outer.to_owned().morph());
                    } else if Outer::StartStatement == token {
                        self.mode = Modes::Statement(outer.to_owned().morph());
                    }
                    let t = BlockToken::Block(token, span);
                    Some(t)
                } else {
                    None
                }
            }
        }
    }
}

fn normalize(tokens: Vec<BlockToken>) -> Vec<BlockToken> {
    let mut normalized: Vec<BlockToken> = Vec::new();
    let mut span: Option<Span> = None;

    for t in tokens.into_iter() {

        if t.is_text() {
            if let Some(ref mut span) = span {
                span.end = t.span().end; 
            } else {
                span = Some(t.span().clone());
            }
        } else {
            if let Some(span) = span.take() {
                normalized.push(BlockToken::Block(Outer::Text, span));
                normalized.push(t);
            } else {
                normalized.push(t);
            }
        }
    }

    normalized
}

pub fn lex(s: &str) -> Vec<BlockToken> {
    let moded = ModeBridge {
        mode: Modes::new(s),
    };
    let tokens = moded.collect();
    normalize(tokens)
}
