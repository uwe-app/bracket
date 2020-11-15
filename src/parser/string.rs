use logos::Span;
use serde_json::Value;

use crate::{
    //error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{DoubleQuoteString, Lexer, Parameters, SingleQuoteString, Token},
    parser::ParseState,
    SyntaxResult,
};

pub(crate) enum Type {
    Double,
    Single,
}

/// Parse a quoted string literal and return a span 
/// that matches the inner value without quotes.
pub(crate) fn parse<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    current: (Parameters, Span),
    string_type: Type,
) -> SyntaxResult<Span> {
    let (_lex, span) = current;
    let str_start = span.end;
    let mut str_end = span.end;

    while let Some(token) = lexer.next() {
        match string_type {
            Type::Double => match token {
                Token::DoubleQuoteString(lex, span) => match &lex {
                    DoubleQuoteString::End => {
                        return Ok(str_start..str_end);
                    }
                    _ => {
                        *state.byte_mut() = span.end;
                        str_end = span.end;
                    }
                },
                _ => panic!("Expecting string literal token"),
            },
            Type::Single => match token {
                Token::SingleQuoteString(lex, span) => match &lex {
                    SingleQuoteString::End => {
                        return Ok(str_start..str_end);
                    }
                    _ => {
                        *state.byte_mut() = span.end;
                        str_end = span.end;
                    }
                },
                _ => panic!("Expecting string literal token"),
            },
        }
    }
    panic!("Failed to parse string literal");
}

/// Parse a quoted string literal and return a value.
pub(crate) fn literal<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    current: (Parameters, Span),
    string_type: Type,
) -> SyntaxResult<Value> {
    let span = parse(source, lexer, state, current, string_type)?;
    let str_value = &source[span.start..span.end];
    return Ok(Value::String(str_value.to_string()));
}
