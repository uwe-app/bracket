// Parses a double-quoted JSON-style string into tokens.
use logos::{Span, Logos};

use super::modes::{self, Extras};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum Outer {
    #[token("\"")]
    Start,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum Inner {
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

pub fn lex(s: &str) -> Vec<(super::modes::Tokens<Outer, Inner>, Span)> {
    modes::lex::<Outer, Inner>(s, Outer::Start, Inner::End)
}
