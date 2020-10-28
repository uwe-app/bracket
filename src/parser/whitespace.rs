use std::vec::IntoIter;
use logos::Span;

use crate::lexer::Parameters;
use crate::parser::ParseState;

/// Consume whitespace tokens.
pub(crate) fn parse(
    iter: &mut IntoIter<(Parameters, Span)>,
    state: &mut ParseState,
) -> Option<(Parameters, Span)> {
    while let Some(item) = iter.next() {
        if item.0 == Parameters::WhiteSpace || item.0 == Parameters::Newline
        {
            *state.byte_mut() = item.1.end;
            if item.0 == Parameters::Newline {
                *state.line_mut() += 1;
            }
        } else {
            return Some(item);
        }
    }
    None
}
