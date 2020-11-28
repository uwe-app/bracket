//! Iterator for grammar tokens.
use logos::{Lexer as Lex, Logos, Span};

/// Identity type for the lexer modes.
#[derive(Clone, Default)]
pub struct Extras;

/// Tokens for the document and nested blocks.
#[derive(Logos, Clone, Debug, Eq, PartialEq)]
#[logos(extras = Extras)]
#[logos(subpattern identifier = r#"[^\s"!#%&'()*+,./;<=>@\[/\]^`{|}~]"#)]
pub enum Block {
    /// Start a raw block.
    #[regex(r"\{\{\{\{~?[\t ]*")]
    StartRawBlock,

    /// Start a raw comment.
    #[regex(r"\{\{!--")]
    StartRawComment,

    /// Start a raw (escaped) statement.
    #[regex(r"\\\{\{\{?")]
    StartRawStatement,

    /// Start a comment.
    #[regex(r"\{\{!")]
    StartComment,

    /// Start a statement.
    #[regex(r"\{\{\{?~?[\t ]*")]
    StartStatement,

    /// Start a block.
    #[regex(r"\{\{\~?[\t ]*#[\t ]*")]
    StartBlockScope,

    /// Start a link.
    #[regex(r"\\?\[\[")]
    StartLink,

    /// End a block.
    #[regex(r"\{\{\~?[\t ]*/")]
    EndBlockScope,

    /// End a raw block.
    #[regex(r"\{\{\{\{~?[\t ]*/")]
    EndRawBlock,

    /// Text token.
    #[regex(r".")]
    Text,

    /// Newline token.
    #[token("\n")]
    Newline,

    /// Error token.
    #[error]
    Error,
}

/// Tokens for raw comments.
///
/// Raw comments can contain statements and blocks which will
/// not be rendered. They begin with `{{!--` and are terminated
/// with `--}}`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum RawComment {
    /// Text token.
    #[regex(r".")]
    Text,

    /// End of raw comment.
    #[regex(r"--\}\}")]
    End,

    /// Newline token.
    #[token("\n")]
    Newline,

    /// Error token.
    #[error]
    Error,
}

/// Tokens for raw statements.
///
/// Raw statements are single-line statements escaped with a
/// backslash, for example: `\{{title}}`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum RawStatement {
    /// Text token.
    #[regex(r".")]
    Text,

    /// End of raw statement.
    #[regex(r"~?\}?\}\}")]
    End,

    /// Newline token.
    #[token("\n")]
    Newline,

    /// Error token.
    #[error]
    Error,
}

/// Tokens for comments.
///
/// Comments may **not** contain statements and blocks.
/// They begin with `{{!` and are terminated with `}}`.
///
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum Comment {
    /// Text token.
    #[regex(r".")]
    Text,

    /// End of comment.
    #[regex(r"\}\}")]
    End,

    /// Newline token.
    #[token("\n")]
    Newline,

    /// Error token.
    #[error]
    Error,
}

/// Tokens for parameters.
///
/// Parameters are converted to a call statement by the parser and must
/// represent all the tokens in a statement (`{{...}}`) and the start
/// of a block (`{{# block}}...{{/block}}`).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
#[logos(subpattern identifier = r#"[^\s"!#%&'()*+,./;<=>@\[/\]^`{|}~]"#)]
pub enum Parameters {
    /// Token for a partial instruction.
    #[token(r">")]
    Partial,

    /// Token for the `else` conditional keyword.
    #[token(r"else")]
    ElseKeyword,

    /// Token for the explicit `this` keyword.
    #[token(r"this")]
    ExplicitThisKeyword,

    /// Token for explicit `this` notation using a dot and a slash.
    #[token("./")]
    ExplicitThisDotSlash,

    /// Token for a reference to a parent scope.
    #[token("../")]
    ParentRef,

    /// Token for a valid identifier.
    #[regex(r"(?&identifier)+", priority = 2)]
    Identifier,

    /// Token for a local identifier (preceeded by an `@` symbol).
    #[regex(r"@(?&identifier)+")]
    LocalIdentifier,

    /// Token for the delimiter between path components.
    #[regex(r"[./]")]
    PathDelimiter,

    /// Token that starts a double-quoted string literal.
    #[token("\"")]
    DoubleQuoteString,

    /// Token that starts a single-quoted string literal.
    #[token("'")]
    SingleQuoteString,

    /// Token that starts a raw literal using square brackets.
    #[token("[")]
    StartArray,

    /// Token that starts a sub-expression.
    #[token("(", priority = 3)]
    StartSubExpression,

    /// Token that ends a sub-expression.
    #[token(")")]
    EndSubExpression,

    /// Token for key/value pairs (hash parameters).
    #[regex(r"(?&identifier)+=")]
    HashKey,

    /// Token for numeric values.
    // NOTE: Must have higher priority than identifier
    // NOTE: otherwise numbers become identifiers
    #[regex(r"-?([0-9]+\.)?[0-9]+((e|E)[+-]?[0-9]+)?", priority = 3)]
    Number,

    /// Token for the `true` keyword.
    #[token("true")]
    True,

    /// Token for the `false` keyword.
    #[token("false")]
    False,

    /// Token for the `null` keyword.
    #[token("null")]
    Null,

    /// Token for whitespace delimiters.
    #[regex(r"[ \t]+")]
    WhiteSpace,

    /// Token for the end of a statement or block open tag.
    #[regex(r"~?\}?\}?\}\}")]
    End,

    /// Newline token.
    #[token("\n")]
    Newline,

    /// Error token.
    #[error]
    Error,
}

/// Tokens for double-quoted string literals.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum DoubleQuoteString {
    /// Text token.
    #[regex(r#"[^\\"\n]+"#)]
    Text,

    /// Escaped newline token.
    #[token("\\n")]
    EscapedNewline,

    /// Escaped quote.
    #[token(r#"\""#)]
    Escaped,

    /// End of the string literal.
    #[token("\"")]
    End,

    /// Newline token.
    #[token("\n")]
    Newline,

    /// Error token.
    #[error]
    Error,
}

/// Tokens for single-quoted string literals.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum SingleQuoteString {
    /// Text token.
    #[regex(r#"[^\\'\n]+"#)]
    Text,

    /// Escaped newline token.
    #[token("\\n")]
    EscapedNewline,

    /// Escaped quote.
    #[token(r#"\'"#)]
    Escaped,

    /// End of the string literal.
    #[token("'")]
    End,

    /// Newline token.
    #[token("\n")]
    Newline,

    /// Error token.
    #[error]
    Error,
}

/// Tokens for square bracket raw literals.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum Array {
    /// Text token.
    #[regex(r#"[^\]\n]+"#)]
    Text,

    //#[token("\\n")]
    //EscapedNewline,
    /// Escaped bracket.
    #[token(r#"\]"#)]
    Escaped,

    /// End of the raw literal.
    #[token("]")]
    End,

    /// Newline token.
    #[token("\n")]
    Newline,

    /// Error token.
    #[error]
    Error,
}

/// Tokens for links.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum Link {
    /// Text token.
    #[regex(r#"[^\\|\]]+"#)]
    Text,

    /// Pipe delimiter token.
    #[token("|")]
    Pipe,

    /// Escaped newline token.
    #[token("\\n")]
    EscapedNewline,

    /// Escaped pipe token.
    #[token(r#"\|"#)]
    EscapedPipe,

    /// Escaped bracket token.
    #[token(r#"\]"#)]
    Escaped,

    /// End of square bracket literal.
    #[token(r"]]")]
    End,

    /// Newline token.
    #[token("\n")]
    Newline,

    /// Error token.
    #[error]
    Error,
}

/// Enumeration of the token types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    /// Block token.
    Block(Block, Span),
    /// Raw comment token.
    RawComment(RawComment, Span),
    /// Raw statement token.
    RawStatement(RawStatement, Span),
    /// Comment token.
    Comment(Comment, Span),
    /// Token for call parameters.
    Parameters(Parameters, Span),
    /// Token for a double-quoted string literal.
    DoubleQuoteString(DoubleQuoteString, Span),
    /// Token for a single-quoted string literal.
    SingleQuoteString(SingleQuoteString, Span),
    /// Token for a raw square bracket literal.
    Array(Array, Span),
    /// Token for links.
    Link(Link, Span),
}

impl Token {
    /// Get the span for a token.
    pub fn span(&self) -> &Span {
        match self {
            Token::Block(_, ref span) => span,
            Token::RawComment(_, ref span) => span,
            Token::RawStatement(_, ref span) => span,
            Token::Comment(_, ref span) => span,
            Token::Parameters(_, ref span) => span,
            Token::DoubleQuoteString(_, ref span) => span,
            Token::SingleQuoteString(_, ref span) => span,
            Token::Array(_, ref span) => span,
            Token::Link(_, ref span) => span,
        }
    }

    /// Determine if a token should be treated as text.
    pub fn is_text(&self) -> bool {
        match self {
            Token::Block(ref t, _) => t == &Block::Text || t == &Block::Newline,
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
            Token::DoubleQuoteString(_, _) => false,
            Token::SingleQuoteString(_, _) => false,
            Token::Array(_, _) => false,
            Token::Link(_, _) => false,
        }
    }

    /// Determine if a token is the newline token.
    pub fn is_newline(&self) -> bool {
        match *self {
            Token::RawComment(ref lex, _) => lex == &RawComment::Newline,
            Token::RawStatement(ref lex, _) => lex == &RawStatement::Newline,
            Token::Comment(ref lex, _) => lex == &Comment::Newline,
            //Token::RawBlock(ref lex, _) => lex == &Block::Newline,
            Token::Block(ref lex, _) => lex == &Block::Newline,
            Token::Parameters(ref lex, _) => lex == &Parameters::Newline,
            Token::DoubleQuoteString(ref lex, _) => {
                lex == &DoubleQuoteString::Newline
            }
            Token::SingleQuoteString(ref lex, _) => {
                lex == &SingleQuoteString::Newline
            }
            Token::Array(ref lex, _) => lex == &Array::Newline,
            Token::Link(ref lex, _) => lex == &Link::Newline,
        }
    }
}

enum Modes<'source> {
    Block(Lex<'source, Block>),
    RawComment(Lex<'source, RawComment>),
    RawStatement(Lex<'source, RawStatement>),
    Comment(Lex<'source, Comment>),
    Parameters(Lex<'source, Parameters>),
    DoubleQuoteString(Lex<'source, DoubleQuoteString>),
    SingleQuoteString(Lex<'source, SingleQuoteString>),
    Array(Lex<'source, Array>),
    Link(Lex<'source, Link>),
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

    /// Consume nodes until we can return to the top-level mode.
    ///
    /// This is used during *lint* mode to move back to the top-level
    /// parsing mode.
    pub(crate) fn until_mode(&mut self) -> Option<Token> {
        while let Some(token) = self.next() {
            match token {
                Token::Block(_, _) => return Some(token),
                _ => {}
            }
        }
        None
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
                    } else if Block::StartLink == token {
                        self.mode = Modes::Link(lexer.to_owned().morph());
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
                    if Parameters::DoubleQuoteString == token {
                        self.mode =
                            Modes::DoubleQuoteString(lexer.to_owned().morph());
                    } else if Parameters::SingleQuoteString == token {
                        self.mode =
                            Modes::SingleQuoteString(lexer.to_owned().morph());
                    } else if Parameters::StartArray == token {
                        self.mode = Modes::Array(lexer.to_owned().morph());
                    } else if Parameters::End == token {
                        self.mode = Modes::Block(lexer.to_owned().morph());
                    }
                    Some(Token::Parameters(token, span))
                } else {
                    None
                }
            }
            Modes::DoubleQuoteString(lexer) => {
                let result = lexer.next();
                let span = lexer.span();

                if let Some(token) = result {
                    if DoubleQuoteString::End == token {
                        self.mode = Modes::Parameters(lexer.to_owned().morph());
                    }
                    Some(Token::DoubleQuoteString(token, span))
                } else {
                    None
                }
            }
            Modes::SingleQuoteString(lexer) => {
                let result = lexer.next();
                let span = lexer.span();

                if let Some(token) = result {
                    if SingleQuoteString::End == token {
                        self.mode = Modes::Parameters(lexer.to_owned().morph());
                    }
                    Some(Token::SingleQuoteString(token, span))
                } else {
                    None
                }
            }
            Modes::Array(lexer) => {
                let result = lexer.next();
                let span = lexer.span();

                if let Some(token) = result {
                    if Array::End == token {
                        self.mode = Modes::Parameters(lexer.to_owned().morph());
                    }
                    Some(Token::Array(token, span))
                } else {
                    None
                }
            }
            Modes::Link(lexer) => {
                let result = lexer.next();
                let span = lexer.span();

                if let Some(token) = result {
                    if Link::End == token {
                        self.mode = Modes::Block(lexer.to_owned().morph());
                    }
                    Some(Token::Link(token, span))
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
