use logos::{Lexer, Logos, Span};

#[derive(Clone, Default)]
pub struct Extras {
    pub lines: usize,
}

#[derive(Logos, Clone, Debug, Eq, PartialEq)]
#[logos(extras = Extras)]
pub enum Block {
    #[regex(r"\{\{\{\{\s*raw\s*\}\}\}\}")]
    StartRawBlock,

    #[regex(r"\{\{!--")]
    StartRawComment,

    #[regex(r"\\\{\{\{?")]
    StartRawStatement,

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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum RawBlock {
    #[regex(r".")]
    Text,

    #[regex("\r?\n", |lex| {
        lex.extras.lines += 1;
    })]
    Newline,

    #[regex(r"\{\{\{\{\s*/\s*raw\s*\}\}\}\}")]
    End,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum RawComment {
    #[regex(r".")]
    Text,

    #[regex("\r?\n", |lex| {
        lex.extras.lines += 1;
    })]
    Newline,

    #[regex(r"--\}\}")]
    End,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum RawStatement {
    #[regex(r".")]
    Text,

    #[regex("\r?\n", |lex| {
        lex.extras.lines += 1;
    })]
    Newline,

    #[regex(r"\}?\}\}")]
    End,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
#[logos(subpattern identifier = r#"[^\s"!#%&'()*+,./;<=>@\[/\]^`{|}~]"#)]
pub enum Statement {
    #[token(r">")]
    Partial,

    #[regex(r"(?&identifier)+", priority = 2)]
    Identifier,

    #[regex(r"[./]")]
    PathDelimiter,

    #[token("\"")]
    StartStringLiteral,

    #[regex(r"-?[0-9]*\.?[0-9]+")]
    Number,

    #[regex(r"(true|false)")]
    Bool,

    #[token("null")]
    Null,

    #[regex("\r?\n", |lex| {
        lex.extras.lines += 1;
    })]
    Newline,

    #[regex(r" +")]
    WhiteSpace,

    #[regex(r"\}?\}\}")]
    End,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum StringLiteral {
    #[regex(r#"[^\\"]+"#)]
    Text,

    #[token("\\n")]
    EscapedNewline,

    //#[regex(r"\\u\{[^}]*\}")]
    //EscapedCodepoint,
    #[token(r#"\""#)]
    EscapedQuote,

    #[token("\"")]
    End,

    #[error]
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    Block(Block, Span),
    RawBlock(RawBlock, Span),
    RawComment(RawComment, Span),
    RawStatement(RawStatement, Span),
    Statement(Statement, Span),
}

impl Token {
    pub fn span(&self) -> &Span {
        match self {
            Token::Block(_, ref span) => span,
            Token::RawBlock(_, ref span) => span,
            Token::RawComment(_, ref span) => span,
            Token::RawStatement(_, ref span) => span,
            Token::Statement(_, ref span) => span,
        }
    }

    pub fn is_text(&self) -> bool {
        match self {
            Token::Block(ref t, _) => t == &Block::Text || t == &Block::Newline,
            Token::RawBlock(ref t, _) => {
                t == &RawBlock::Text || t == &RawBlock::Newline
            }
            Token::RawComment(ref t, _) => {
                t == &RawComment::Text || t == &RawComment::Newline
            }
            Token::RawStatement(ref t, _) => {
                t == &RawStatement::Text || t == &RawStatement::Newline
            }
            Token::Statement(ref t, _) => false,
        }
    }
}

//pub struct Token(pub Box<dyn LexToken>, pub Span);

enum Modes<'source> {
    Block(Lexer<'source, Block>),
    RawBlock(Lexer<'source, RawBlock>),
    RawComment(Lexer<'source, RawComment>),
    RawStatement(Lexer<'source, RawStatement>),
    Statement(Lexer<'source, Statement>),
}

impl<'source> Modes<'source> {
    fn new(s: &'source str) -> Self {
        Self::Block(Block::lexer(s))
    }
}

struct ModeBridge<'source> {
    mode: Modes<'source>,
}

// Clones as we switch between modes
impl<'source> Iterator for ModeBridge<'source> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.mode {
            Modes::RawBlock(inner) => {
                let result = inner.next();
                let span = inner.span();
                if let Some(token) = result {
                    if RawBlock::End == token {
                        self.mode = Modes::Block(inner.to_owned().morph());
                    }
                    let t = Token::RawBlock(token, span);
                    Some(t)
                } else {
                    None
                }
            }
            Modes::RawComment(inner) => {
                let result = inner.next();
                let span = inner.span();
                if let Some(token) = result {
                    if RawComment::End == token {
                        self.mode = Modes::Block(inner.to_owned().morph());
                    }
                    let t = Token::RawComment(token, span);
                    Some(t)
                } else {
                    None
                }
            }
            Modes::RawStatement(inner) => {
                let result = inner.next();
                let span = inner.span();
                if let Some(token) = result {
                    if RawStatement::End == token {
                        self.mode = Modes::Block(inner.to_owned().morph());
                    }
                    let t = Token::RawStatement(token, span);
                    Some(t)
                } else {
                    None
                }
            }
            Modes::Statement(inner) => {
                let result = inner.next();
                let span = inner.span();
                if let Some(token) = result {
                    if Statement::End == token {
                        self.mode = Modes::Block(inner.to_owned().morph());
                    }
                    let t = Token::Statement(token, span);
                    Some(t)
                } else {
                    None
                }
            }
            Modes::Block(outer) => {
                let result = outer.next();
                let span = outer.span();
                if let Some(token) = result {
                    if Block::StartRawBlock == token {
                        self.mode = Modes::RawBlock(outer.to_owned().morph());
                    } else if Block::StartRawComment == token {
                        self.mode = Modes::RawComment(outer.to_owned().morph());
                    } else if Block::StartRawStatement == token {
                        self.mode =
                            Modes::RawStatement(outer.to_owned().morph());
                    } else if Block::StartStatement == token {
                        self.mode = Modes::Statement(outer.to_owned().morph());
                    }
                    let t = Token::Block(token, span);
                    Some(t)
                } else {
                    None
                }
            }
        }
    }
}

fn normalize(tokens: Vec<Token>) -> Vec<Token> {
    let mut normalized: Vec<Token> = Vec::new();
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
                normalized.push(Token::Block(Block::Text, span));
                normalized.push(t);
            } else {
                normalized.push(t);
            }
        }
    }

    if let Some(span) = span.take() {
        normalized.push(Token::Block(Block::Text, span));
    }

    normalized
}

/// Lex the input source into a stream of tokens.
///
/// If the normalized flag is given consecutive text tokens
/// are coalesced into a single token.
///
/// The normalized flag is useful for test cases; the parser
/// will perform it's own normalization to reduce the number of
/// passes on the token strea,.
pub fn lex(s: &str, normalized: bool) -> Vec<Token> {
    let moded = ModeBridge {
        mode: Modes::new(s),
    };
    let tokens = moded.collect();
    if normalized {
        normalize(tokens)
    } else {
        tokens
    }
}
