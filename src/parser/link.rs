use std::ops::Range;
use serde_json::{Number, Value};

use crate::{
    error::{ErrorInfo, SyntaxError},
    lexer::{Lexer, Parameters, Token},
    parser::{
        ast::{Link, Lines},
        path, string, ParseState,
    },
    SyntaxResult,
};

pub(crate) fn parse<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    open: Range<usize>,
) -> SyntaxResult<Link<'source>> {
    *state.byte_mut() = open.end;

    let link = Link::new(source, open);

    Ok(link)
}

