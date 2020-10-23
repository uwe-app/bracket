/// Parses a handlebars statement into tokens.
use logos::{Lexer, Logos as _, Span};
use logos_derive::Logos;

use super::modes::{self, Extras};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum Outer {
    #[regex(r"\{\{\{?")]
    Start,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
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

pub fn lex(s: &str) -> Vec<(super::modes::Tokens<Outer, Inner>, Span)> {
    modes::lex::<Outer, Inner>(s, Outer::Start, Inner::End)
}
