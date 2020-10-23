/// Parses a handlebars statement into tokens.
use logos::{Lexer, Logos, Span};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
pub enum Outer {
    #[regex(r"\{\{\{?")]
    Start,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(subpattern identifier = r#"[^\s"!#%&'()*+,./;<=>@\[/\]^`{|}~]"#)]
pub enum Inner {
    #[token(r">")]
    Partial,

    #[regex(r"(?&identifier)+", priority = 2)]
    Identifier,

    #[regex(r"[./]")]
    PathDelimiter,

    #[regex(r"-?[0-9]*\.?[0-9]+")]
    Number,

    #[regex(r"(true|false)")]
    Bool,

    #[token("null")]
    Null,

    #[regex(r"\s+")]
    WhiteSpace,

    #[regex(r"\}?\}\}")]
    End,

    #[error]
    Error,
}

enum Modes<'source> {
    Outer(Lexer<'source, Outer>),
    Inner(Lexer<'source, Inner>),
}

impl<'source> Modes<'source> {
    fn new(s: &'source str) -> Self {
        Self::Outer(Outer::lexer(s))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Tokens {
    InnerToken(Inner),
    OuterToken(Outer),
}

struct ModeBridge<'source> {
    mode: Modes<'source>,
}

// Clones as we switch between modes
impl<'source> Iterator for ModeBridge<'source> {
    type Item = (Tokens, Span);
    fn next(&mut self) -> Option<Self::Item> {
        use Tokens::*;
        match &mut self.mode {
            Modes::Inner(inner) => {
                let result = inner.next();
                let span = inner.span();
                //println!("Inner span {:?}", span);
                if let Some(token) = result {
                    if Inner::End == token {
                        self.mode = Modes::Outer(inner.to_owned().morph());
                    }
                    Some((InnerToken(token), span))
                } else {
                    None
                }
            }
            Modes::Outer(outer) => {
                let result = outer.next();
                let span = outer.span();
                //println!("Outer span {:?}", span);
                if let Some(token) = result {
                    if Outer::Start == token {
                        self.mode = Modes::Inner(outer.to_owned().morph());
                    }
                    Some((OuterToken(token), span))
                } else {
                    None
                }
            }
        }
    }
}

pub fn lex(s: &str) -> Vec<(Tokens, Span)> {
    let moded = ModeBridge {
        mode: Modes::new(s),
    };
    let results: Vec<(Tokens, Span)> = moded.collect();
    results
}
