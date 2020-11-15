use logos::Span;
use serde_json::Value;
use std::borrow::Cow;

use crate::{
    //error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{
        Array, DoubleQuoteString, Lexer, Parameters, SingleQuoteString, Token,
    },
    parser::ParseState,
    SyntaxResult,
};

#[derive(Copy, Clone, Debug)]
pub enum RawLiteralType {
    Double,
    Single,
    Array,
}

#[derive(Debug)]
pub struct RawLiteral {
    /// Escaped newline was encountered during parsing.
    pub newline: bool,
    /// Escaped delimiter was encountered during parsing.
    pub delimiter: bool,
    /// The type of literal statement.
    pub literal_type: RawLiteralType,
}

impl RawLiteral {

    pub fn has_escape_sequences(&self) -> bool {
        self.newline || self.delimiter 
    }

    pub fn into_value<'a>(&self, value: &'a str) -> Cow<'a, str> {
        let mut val = Cow::from(value);
        if self.newline {
            val = Cow::from(val.to_mut().replace("\\n", "\n"));
        }
        if self.delimiter {
            match self.literal_type {
                RawLiteralType::Double => {
                    val = Cow::from(val.to_mut().replace(r#"\""#, r#"""#));
                }
                RawLiteralType::Single => {
                    val = Cow::from(val.to_mut().replace(r"\'", "'"));
                }
                RawLiteralType::Array => {
                    val = Cow::from(val.to_mut().replace(r"\]", "]"));
                }
            }
        }
        val
    }
}

/// Parse a quoted string literal and return a span
/// that matches the inner value without quotes.
pub(crate) fn parse<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    current: (Parameters, Span),
    string_type: RawLiteralType,
) -> SyntaxResult<(Span, RawLiteral)> {
    let (_lex, span) = current;
    let str_start = span.end;
    let mut str_end = span.end;

    let mut flags = RawLiteral {
        literal_type: string_type,
        newline: false,
        delimiter: false
    };

    while let Some(token) = lexer.next() {
        match string_type {
            RawLiteralType::Double => match token {
                Token::DoubleQuoteString(lex, span) => match &lex {
                    DoubleQuoteString::EscapedNewline => {
                        flags.newline = true;
                    }
                    DoubleQuoteString::Escaped => {
                        flags.delimiter = true;
                    }
                    DoubleQuoteString::End => {
                        return Ok((str_start..str_end, flags));
                    }
                    _ => {
                        *state.byte_mut() = span.end;
                        str_end = span.end;
                    }
                },
                _ => panic!("Expecting string literal token"),
            },
            RawLiteralType::Single => match token {
                Token::SingleQuoteString(lex, span) => match &lex {
                    SingleQuoteString::EscapedNewline => {
                        flags.newline = true;
                    }
                    SingleQuoteString::Escaped => {
                        flags.delimiter = true;
                    }
                    SingleQuoteString::End => {
                        return Ok((str_start..str_end, flags));
                    }
                    _ => {
                        *state.byte_mut() = span.end;
                        str_end = span.end;
                    }
                },
                _ => panic!("Expecting string literal token"),
            },
            RawLiteralType::Array => match token {
                Token::Array(lex, span) => match &lex {
                    Array::Escaped => {
                        flags.delimiter = true;
                    }
                    Array::End => {
                        return Ok((str_start..str_end, flags));
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
    string_type: RawLiteralType,
) -> SyntaxResult<Value> {
    let (span, flags) = parse(source, lexer, state, current, string_type)?;
    let value = flags.into_value(&source[span.start..span.end]);
    return Ok(Value::String(value.into_owned().to_string()));
}
