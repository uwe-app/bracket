use std::vec::IntoIter;
use logos::Span;

use crate::{
    lexer::grammar::Parameters,
};

/// Consume whitespace tokens.
pub(crate) fn parse(
    iter: &mut IntoIter<(Parameters, Span)>,
    byte_offset: &mut usize,
    line: &mut usize,
) -> Option<(Parameters, Span)> {
    while let Some(item) = iter.next() {
        if item.0 == Parameters::WhiteSpace || item.0 == Parameters::Newline
        {
            *byte_offset = item.1.end;
            if item.0 == Parameters::Newline {
                *line += 1;
            }
        } else {
            return Some(item);
        }
    }
    None
}
