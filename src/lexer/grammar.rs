use logos::{Lexer, Logos};
use std::ops::Range;

/// Type to indicate a line number range: `Range<usize>`.
pub type Span = Range<usize>;
pub type LineNumber = Span;
//pub type SourceMap = (pub Span, pub LineNumber);

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

    #[regex(r"\{\{!")]
    StartComment,

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
pub enum Comment {
    #[regex(r".")]
    Text,

    #[regex("\r?\n", |lex| {
        lex.extras.lines += 1;
    })]
    Newline,

    #[regex(r"\}\}")]
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

    #[regex("\r?\n", |lex| {
        lex.extras.lines += 1;
    })]
    Newline,

    #[token("\"")]
    End,

    #[error]
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    Block(Block, Span, LineNumber),
    RawBlock(RawBlock, Span, LineNumber),
    RawComment(RawComment, Span, LineNumber),
    RawStatement(RawStatement, Span, LineNumber),
    Comment(Comment, Span, LineNumber),
    Statement(Statement, Span, LineNumber),
}

impl Token {
    pub fn span(&self) -> &Span {
        match self {
            Token::Block(_, ref span, _) => span,
            Token::RawBlock(_, ref span, _) => span,
            Token::RawComment(_, ref span, _) => span,
            Token::RawStatement(_, ref span, _) => span,
            Token::Comment(_, ref span, _) => span,
            Token::Statement(_, ref span, _) => span,
        }
    }

    pub fn lines(&self) -> &LineNumber {
        match self {
            Token::Block(_, _, ref lines) => lines,
            Token::RawBlock(_, _, ref lines) => lines,
            Token::RawComment(_, _, ref lines) => lines,
            Token::RawStatement(_, _, ref lines) => lines,
            Token::Comment(_, _, ref lines) => lines,
            Token::Statement(_, _, ref lines) => lines,
        }
    }

    pub fn is_text(&self) -> bool {
        match self {
            Token::Block(ref t, _, _) => {
                t == &Block::Text || t == &Block::Newline
            }
            Token::RawBlock(ref t, _, _) => {
                t == &RawBlock::Text || t == &RawBlock::Newline
            }
            Token::RawComment(ref t, _, _) => {
                t == &RawComment::Text || t == &RawComment::Newline
            }
            Token::RawStatement(ref t, _, _) => {
                t == &RawStatement::Text || t == &RawStatement::Newline
            }
            Token::Comment(ref t, _, _) => {
                t == &Comment::Text || t == &Comment::Newline
            }
            Token::Statement(_, _, _) => false,
        }
    }
}

//pub struct Token(pub Box<dyn LexToken>, pub Span);

enum Modes<'source> {
    Block(Lexer<'source, Block>),
    RawBlock(Lexer<'source, RawBlock>),
    RawComment(Lexer<'source, RawComment>),
    RawStatement(Lexer<'source, RawStatement>),
    Comment(Lexer<'source, Comment>),
    Statement(Lexer<'source, Statement>),
}

impl<'source> Modes<'source> {
    fn new(s: &'source str) -> Self {
        Self::Block(Block::lexer(s))
    }
}

pub struct ModeBridge<'source> {
    mode: Modes<'source>,
}

/// Clone lexers as we switch between modes.
impl<'source> Iterator for ModeBridge<'source> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.mode {
            Modes::RawBlock(lexer) => {
                let result = lexer.next();
                let span = lexer.span();
                let lines = lexer.extras.lines..lexer.extras.lines;

                if let Some(token) = result {
                    if RawBlock::End == token {
                        self.mode = Modes::Block(lexer.to_owned().morph());
                    }
                    Some(Token::RawBlock(token, span, lines))
                } else {
                    None
                }
            }
            Modes::RawComment(lexer) => {
                let result = lexer.next();
                let span = lexer.span();
                let lines = lexer.extras.lines..lexer.extras.lines;

                if let Some(token) = result {
                    if RawComment::End == token {
                        self.mode = Modes::Block(lexer.to_owned().morph());
                    }
                    Some(Token::RawComment(token, span, lines))
                } else {
                    None
                }
            }
            Modes::RawStatement(lexer) => {
                let result = lexer.next();
                let span = lexer.span();
                let lines = lexer.extras.lines..lexer.extras.lines;

                if let Some(token) = result {
                    if RawStatement::End == token {
                        self.mode = Modes::Block(lexer.to_owned().morph());
                    }
                    Some(Token::RawStatement(token, span, lines))
                } else {
                    None
                }
            }
            Modes::Comment(lexer) => {
                let result = lexer.next();
                let span = lexer.span();
                let lines = lexer.extras.lines..lexer.extras.lines;

                if let Some(token) = result {
                    if Comment::End == token {
                        self.mode = Modes::Block(lexer.to_owned().morph());
                    }
                    Some(Token::Comment(token, span, lines))
                } else {
                    None
                }
            }
            Modes::Statement(lexer) => {
                let result = lexer.next();
                let span = lexer.span();
                let lines = lexer.extras.lines..lexer.extras.lines;

                if let Some(token) = result {
                    if Statement::End == token {
                        self.mode = Modes::Block(lexer.to_owned().morph());
                    }
                    Some(Token::Statement(token, span, lines))
                } else {
                    None
                }
            }
            Modes::Block(lexer) => {
                let result = lexer.next();
                let span = lexer.span();
                let lines = lexer.extras.lines..lexer.extras.lines;

                if let Some(token) = result {
                    if Block::StartRawBlock == token {
                        self.mode = Modes::RawBlock(lexer.to_owned().morph());
                    } else if Block::StartRawComment == token {
                        self.mode = Modes::RawComment(lexer.to_owned().morph());
                    } else if Block::StartRawStatement == token {
                        self.mode =
                            Modes::RawStatement(lexer.to_owned().morph());
                    } else if Block::StartComment == token {
                        self.mode = Modes::Comment(lexer.to_owned().morph());
                    } else if Block::StartStatement == token {
                        self.mode = Modes::Statement(lexer.to_owned().morph());
                    }
                    Some(Token::Block(token, span, lines))
                } else {
                    None
                }
            }
        }
    }
}

fn normalize(tokens: Vec<Token>) -> Vec<Token> {
    let mut normalized: Vec<Token> = Vec::new();
    let mut span: Option<(Span, LineNumber)> = None;

    for t in tokens.into_iter() {
        if t.is_text() {
            if let Some((ref mut span, ref mut lines)) = span {
                span.end = t.span().end;
                lines.end = t.lines().end;
            } else {
                span = Some((t.span().clone(), t.lines().clone()));
            }
        } else {
            if let Some((span, lines)) = span.take() {
                normalized.push(Token::Block(Block::Text, span, lines));
                normalized.push(t);
            } else {
                normalized.push(t);
            }
        }
    }

    if let Some((span, lines)) = span.take() {
        normalized.push(Token::Block(Block::Text, span, lines));
    }

    normalized
}

/// Iterator for the grammar tokens.
pub fn lex(s: &str) -> ModeBridge {
    ModeBridge {
        mode: Modes::new(s),
    }
}

/// Collect the input source into a vector of tokens.
///
/// If the normalized flag is given consecutive text tokens
/// are coalesced into a single token.
///
/// The normalized flag is useful for test cases; the parser
/// will perform it's own normalization to reduce the number of
/// passes on the token stream.
pub fn collect(s: &str, normalized: bool) -> Vec<Token> {
    let tokens = lex(s).collect();
    if normalized {
        normalize(tokens)
    } else {
        tokens
    }
}
