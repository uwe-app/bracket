/// Parses a raw block into tokens.
use logos::{Logos, Span};

use super::modes::{self, Extras, Tokens};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum Outer {
    #[regex(r"\{\{\{\{\s*raw\s*\}\}\}\}")]
    Start,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
#[logos(extras = Extras)]
pub enum Inner {
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

pub fn lex(s: &str) -> Vec<(Tokens<Outer, Inner>, Span)> {
    let tokens = modes::lex::<Outer, Inner>(s, Outer::Start, Inner::End);
    let is_text = |i: &Inner| {
        i == &Inner::Text || i == &Inner::Newline
    };
    modes::normalize::<Outer, Inner>(tokens, Inner::Text, &is_text)
}
