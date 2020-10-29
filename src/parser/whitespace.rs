use logos::Span;
use std::vec::IntoIter;

use crate::lexer::Parameters;
use crate::parser::ParseState;

/// Consume whitespace tokens.
pub(crate) fn parse(
    iter: &mut IntoIter<(Parameters, Span)>,
    state: &mut ParseState,
) -> Option<(Parameters, Span)> {
    while let Some((lex, span)) = iter.next() {
        if lex == Parameters::WhiteSpace || lex == Parameters::Newline {
            *state.byte_mut() = span.end;
            if lex == Parameters::Newline {
                *state.line_mut() += 1;
            }
        } else {
            return Some((lex, span));
        }
    }
    None
}
