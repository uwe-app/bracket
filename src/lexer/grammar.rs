use logos::{Lexer, Logos, Span};
use std::ops::Range;

#[derive(Clone, Default)]
pub struct Extras;

#[derive(Logos, Clone, Debug, Eq, PartialEq)]
#[logos(extras = Extras)]
#[logos(subpattern identifier = r#"[^\s"!#%&'()*+,./;<=>@\[/\]^`{|}~]"#)]
pub enum Block {
    #[regex(r"\{\{\{\{\s*raw\s*\}\}\}\}")]
    StartRawBlock,

    #[regex(r"\{\{!--")]
    StartRawComment,

    #[regex(r"\\\{\{\{?")]
    StartRawStatement,

    #[regex(r"\{\{!")]
    StartComment,

    #[regex(r"\{\{\{?~?")]
    StartStatement,

    #[regex(r"\{\{\~?#")]
    StartBlockScope,

    #[regex(r"\{\{\~?\s*else")]
    StartElseScope,

    #[regex(r"\{\{\~?s*/(?&identifier)+\s*~?\}\}")]
    EndBlockScope,

    #[regex(r".")]
    Text,

    #[token("\n")]
    Newline,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum RawBlock {
    #[regex(r".")]
    Text,

    #[regex(r"\{\{\{\{\s*/\s*raw\s*\}\}\}\}")]
    End,

    #[token("\n")]
    Newline,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum RawComment {
    #[regex(r".")]
    Text,

    #[regex(r"--\}\}")]
    End,

    #[token("\n")]
    Newline,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum RawStatement {
    #[regex(r".")]
    Text,

    #[regex(r"\}?\}\}")]
    End,

    #[token("\n")]
    Newline,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum Comment {
    #[regex(r".")]
    Text,

    #[regex(r"\}\}")]
    End,

    #[token("\n")]
    Newline,

    #[error]
    Error,
}

/*
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
#[logos(subpattern identifier = r#"[^\s"!#%&'()*+,./;<=>@\[/\]^`{|}~]"#)]
pub enum BlockScope {
    #[regex(r".")]
    Text,

    #[token("\n")]
    Newline,

    #[error]
    Error,
}
*/

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
#[logos(subpattern identifier = r#"[^\s"!#%&'()*+,./;<=>@\[/\]^`{|}~]"#)]
pub enum Parameters {
    #[token(r">")]
    Partial,

    #[regex(r"(this|\./)")]
    ExplicitThisRef,

    #[token("../")]
    ParentRef,

    #[token("(", priority = 3)]
    StartSubExpression,

    #[token(")")]
    EndSubExpression,

    #[regex(r"(?&identifier)+", priority = 2)]
    Identifier,

    #[regex(r"@(?&identifier)+")]
    LocalIdentifier,

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

    #[regex(r" +")]
    WhiteSpace,

    #[regex(r"~?\}?\}\}")]
    End,

    #[token("\n")]
    Newline,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum Statement {
    #[regex(r"\}?\}\}")]
    End,

    #[token("\n")]
    Newline,

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

    #[token("\n")]
    Newline,

    #[error]
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    Block(Block, Span),
    RawBlock(RawBlock, Span),
    RawComment(RawComment, Span),
    RawStatement(RawStatement, Span),
    Comment(Comment, Span),
    //BlockScope(BlockScope, Span),
    Parameters(Parameters, Span),
}

impl Token {
    pub fn span(&self) -> &Span {
        match self {
            Token::Block(_, ref span) => span,
            Token::RawBlock(_, ref span) => span,
            Token::RawComment(_, ref span) => span,
            Token::RawStatement(_, ref span) => span,
            Token::Comment(_, ref span) => span,
            //Token::BlockScope(_, ref span) => span,
            Token::Parameters(_, ref span) => span,
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
            Token::Comment(ref t, _) => {
                t == &Comment::Text || t == &Comment::Newline
            }
            //Token::BlockScope(ref t, _) => {
            //t == &BlockScope::Text || t == &BlockScope::Newline
            //}
            Token::Parameters(_, _) => false,
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
    //BlockScope(Lexer<'source, BlockScope>),
    Parameters(Lexer<'source, Parameters>),
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
            Modes::Block(lexer) => {
                let result = lexer.next();
                let span = lexer.span();

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
                        self.mode = Modes::Parameters(lexer.to_owned().morph());
                    } else if Block::StartBlockScope == token {
                        self.mode = Modes::Parameters(lexer.to_owned().morph());
                    }
                    Some(Token::Block(token, span))
                } else {
                    None
                }
            }
            Modes::RawBlock(lexer) => {
                let result = lexer.next();
                let span = lexer.span();

                if let Some(token) = result {
                    if RawBlock::End == token {
                        self.mode = Modes::Block(lexer.to_owned().morph());
                    }
                    Some(Token::RawBlock(token, span))
                } else {
                    None
                }
            }
            Modes::RawComment(lexer) => {
                let result = lexer.next();
                let span = lexer.span();

                if let Some(token) = result {
                    if RawComment::End == token {
                        self.mode = Modes::Block(lexer.to_owned().morph());
                    }
                    Some(Token::RawComment(token, span))
                } else {
                    None
                }
            }
            Modes::RawStatement(lexer) => {
                let result = lexer.next();
                let span = lexer.span();

                if let Some(token) = result {
                    if RawStatement::End == token {
                        self.mode = Modes::Block(lexer.to_owned().morph());
                    }
                    Some(Token::RawStatement(token, span))
                } else {
                    None
                }
            }
            Modes::Comment(lexer) => {
                let result = lexer.next();
                let span = lexer.span();

                if let Some(token) = result {
                    if Comment::End == token {
                        self.mode = Modes::Block(lexer.to_owned().morph());
                    }
                    Some(Token::Comment(token, span))
                } else {
                    None
                }
            }
            //Modes::BlockScope(lexer) => {
            //let result = lexer.next();
            //let span = lexer.span();
            //if let Some(token) = result {
            //Some(Token::BlockScope(token, span))
            //} else {
            //None
            //}
            //}
            Modes::Parameters(lexer) => {
                let result = lexer.next();
                let span = lexer.span();

                if let Some(token) = result {
                    if Parameters::End == token {
                        self.mode = Modes::Block(lexer.to_owned().morph());
                    }
                    Some(Token::Parameters(token, span))
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
