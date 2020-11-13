//! Iterator for grammar tokens.
use logos::{Lexer as Lex, Logos, Span};

/// Identity type for the lexer modes.
#[derive(Clone, Default)]
pub struct Extras;

#[derive(Logos, Clone, Debug, Eq, PartialEq)]
#[logos(extras = Extras)]
#[logos(subpattern identifier = r#"[^\s"!#%&'()*+,./;<=>@\[/\]^`{|}~]"#)]
pub enum Block {
    #[regex(r"\{\{\{\{~?[\t ]*")]
    StartRawBlock,

    #[regex(r"\{\{!--")]
    StartRawComment,

    #[regex(r"\\\{\{\{?")]
    StartRawStatement,

    #[regex(r"\{\{!")]
    StartComment,

    #[regex(r"\{\{\{?~?[\t ]*")]
    StartStatement,

    #[regex(r"\{\{\~?[\t ]*#[\t ]*")]
    StartBlockScope,

    #[regex(r"\{\{\~?[\t ]*/")]
    EndBlockScope,

    #[regex(r"\{\{\{\{~?[\t ]*/")]
    EndRawBlock,

    #[regex(r".")]
    Text,

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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
#[logos(subpattern identifier = r#"[^\s"!#%&'()*+,./;<=>@\[/\]^`{|}~]"#)]
pub enum Parameters {
    #[token(r">")]
    Partial,

    #[token(r"else")]
    ElseKeyword,

    #[token(r"this")]
    ExplicitThisKeyword,

    #[token("./")]
    ExplicitThisDotSlash,

    #[token("../")]
    ParentRef,

    #[regex(r"(?&identifier)+", priority = 2)]
    Identifier,

    #[regex(r"@(?&identifier)+")]
    LocalIdentifier,

    #[regex(r"[./]")]
    PathDelimiter,

    #[regex(r"\[\d+\]+")]
    ArrayAccess,

    #[token("(", priority = 3)]
    StartSubExpression,

    #[token(")")]
    EndSubExpression,

    #[regex(r"(?&identifier)+=")]
    HashKey,

    #[token("\"")]
    StringLiteral,

    // NOTE: Must have higher priority than identifier
    // NOTE: otherwise numbers become identifiers
    #[regex(r"-?([0-9]+\.)?[0-9]+((e|E)[+-]?[0-9]+)?", priority = 3)]
    Number,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[token("null")]
    Null,

    #[regex(r" +")]
    WhiteSpace,

    #[regex(r"~?\}?\}?\}\}")]
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
    #[regex(r#"[^\\"\n]+"#)]
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

/// Type emitted by the iterator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    Block(Block, Span),
    //RawBlock(Block, Span),
    RawComment(RawComment, Span),
    RawStatement(RawStatement, Span),
    Comment(Comment, Span),
    Parameters(Parameters, Span),
    StringLiteral(StringLiteral, Span),
}

impl Token {
    pub fn span(&self) -> &Span {
        match self {
            Token::Block(_, ref span) => span,
            //Token::RawBlock(_, ref span) => span,
            Token::RawComment(_, ref span) => span,
            Token::RawStatement(_, ref span) => span,
            Token::Comment(_, ref span) => span,
            Token::Parameters(_, ref span) => span,
            Token::StringLiteral(_, ref span) => span,
        }
    }

    pub fn is_text(&self) -> bool {
        match self {
            Token::Block(ref t, _) => t == &Block::Text || t == &Block::Newline,
            //Token::RawBlock(ref t, _) => {
            //t == &Block::Text || t == &Block::Newline
            //}
            Token::RawComment(ref t, _) => {
                t == &RawComment::Text || t == &RawComment::Newline
            }
            Token::RawStatement(ref t, _) => {
                t == &RawStatement::Text || t == &RawStatement::Newline
            }
            Token::Comment(ref t, _) => {
                t == &Comment::Text || t == &Comment::Newline
            }
            Token::Parameters(_, _) => false,
            Token::StringLiteral(_, _) => false,
        }
    }

    pub fn is_newline(&self) -> bool {
        match *self {
            Token::RawComment(ref lex, _) => lex == &RawComment::Newline,
            Token::RawStatement(ref lex, _) => lex == &RawStatement::Newline,
            Token::Comment(ref lex, _) => lex == &Comment::Newline,
            //Token::RawBlock(ref lex, _) => lex == &Block::Newline,
            Token::Block(ref lex, _) => lex == &Block::Newline,
            Token::Parameters(ref lex, _) => lex == &Parameters::Newline,
            // NOTE: new lines are not allowed in string literals
            // NOTE: so we have special handling for this case
            Token::StringLiteral(_, _) => false,
        }
    }
}

//pub struct Token(pub Box<dyn LexToken>, pub Span);

enum Modes<'source> {
    Block(Lex<'source, Block>),
    //RawBlock(Lex<'source, Block>),
    RawComment(Lex<'source, RawComment>),
    RawStatement(Lex<'source, RawStatement>),
    Comment(Lex<'source, Comment>),
    Parameters(Lex<'source, Parameters>),
    StringLiteral(Lex<'source, StringLiteral>),
}

impl<'source> Modes<'source> {
    fn new(s: &'source str) -> Self {
        Self::Block(Block::lexer(s))
    }
}

/// Iterator for a stream of grammar tokens.
pub struct Lexer<'source> {
    mode: Modes<'source>,
}

impl<'source> Lexer<'source> {
    /// Utility for switching the lexer to parameters mode.
    ///
    /// Must be called immediately after creating the lexer otherwise
    /// it is not guaranteed to change the lexer mode.
    pub(crate) fn set_parameters_mode(&mut self) {
        match &mut self.mode {
            Modes::Block(lexer) => {
                self.mode = Modes::Parameters(lexer.to_owned().morph())
            }
            _ => {}
        }
    }
}

/// Clone lexers as we switch between modes.
impl<'source> Iterator for Lexer<'source> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.mode {
            Modes::Block(lexer) => {
                let result = lexer.next();
                let span = lexer.span();

                if let Some(token) = result {
                    if Block::StartRawBlock == token {
                        self.mode = Modes::Parameters(lexer.to_owned().morph());
                    } else if Block::EndRawBlock == token {
                        self.mode = Modes::Parameters(lexer.to_owned().morph());
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
                    } else if Block::EndBlockScope == token {
                        self.mode = Modes::Parameters(lexer.to_owned().morph());
                    }
                    Some(Token::Block(token, span))
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
            Modes::Parameters(lexer) => {
                let result = lexer.next();
                let span = lexer.span();

                if let Some(token) = result {
                    if Parameters::StringLiteral == token {
                        self.mode =
                            Modes::StringLiteral(lexer.to_owned().morph());
                    } else if Parameters::End == token {
                        self.mode = Modes::Block(lexer.to_owned().morph());
                    }
                    Some(Token::Parameters(token, span))
                } else {
                    None
                }
            }
            Modes::StringLiteral(lexer) => {
                let result = lexer.next();
                let span = lexer.span();

                if let Some(token) = result {
                    if StringLiteral::End == token {
                        self.mode = Modes::Parameters(lexer.to_owned().morph());
                    }
                    Some(Token::StringLiteral(token, span))
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

/// Get a token iterator for the given source template.
///
/// The returned iterator will emit tokens of type `Token`.
pub fn lex(s: &str) -> Lexer {
    Lexer {
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
