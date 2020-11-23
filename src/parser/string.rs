use serde_json::Value;
use std::ops::Range;

use crate::{
    error::{ErrorInfo, SyntaxError},
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

    pub fn into_owned<'a>(&self, value: &'a str) -> String {
        let mut val = value.to_string();
        if self.newline {
            val = val.replace("\\n", "\n");
        }
        if self.delimiter {
            match self.literal_type {
                RawLiteralType::Double => {
                    val = val.replace(r#"\""#, r#"""#);
                }
                RawLiteralType::Single => {
                    val = val.replace(r"\'", "'");
                }
                RawLiteralType::Array => {
                    val = val.replace(r"\]", "]");
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
    current: (Parameters, Range<usize>),
    string_type: RawLiteralType,
) -> SyntaxResult<(Range<usize>, RawLiteral)> {
    let (_lex, span) = current;
    let str_start = span.end;
    let mut str_end = span.end;

    let mut flags = RawLiteral {
        literal_type: string_type,
        newline: false,
        delimiter: false,
    };

    while let Some(token) = lexer.next() {
        match string_type {
            RawLiteralType::Double => match token {
                Token::DoubleQuoteString(lex, span) => match &lex {
                    DoubleQuoteString::Newline => {
                        return Err(SyntaxError::LiteralNewline(
                            ErrorInfo::from((source, state)).into(),
                        ))
                    }
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
                        *state.byte_mut() = span.end - 1;
                        str_end = span.end;
                    }
                },
                _ => {
                    return Err(SyntaxError::TokenDoubleQuoteLiteral(
                        ErrorInfo::from((source, state)).into(),
                    ));
                }
            },
            RawLiteralType::Single => match token {
                Token::SingleQuoteString(lex, span) => match &lex {
                    SingleQuoteString::Newline => {
                        return Err(SyntaxError::LiteralNewline(
                            ErrorInfo::from((source, state)).into(),
                        ))
                    }
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
                        *state.byte_mut() = span.end - 1;
                        str_end = span.end;
                    }
                },
                _ => {
                    return Err(SyntaxError::TokenSingleQuoteLiteral(
                        ErrorInfo::from((source, state)).into(),
                    ));
                }
            },
            RawLiteralType::Array => match token {
                Token::Array(lex, span) => match &lex {
                    Array::Newline => {
                        return Err(SyntaxError::LiteralNewline(
                            ErrorInfo::from((source, state)).into(),
                        ))
                    }
                    Array::Escaped => {
                        flags.delimiter = true;
                    }
                    Array::End => {
                        return Ok((str_start..str_end, flags));
                    }
                    _ => {
                        *state.byte_mut() = span.end - 1;
                        str_end = span.end;
                    }
                },
                _ => {
                    return Err(SyntaxError::TokenArrayLiteral(
                        ErrorInfo::from((source, state)).into(),
                    ));
                }
            },
        }
    }

    return Err(SyntaxError::TokenRawLiteral(
        ErrorInfo::from((source, state)).into(),
    ));
}

/// Parse a quoted string literal and return a value.
pub(crate) fn literal<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    current: (Parameters, Range<usize>),
    string_type: RawLiteralType,
) -> SyntaxResult<(Value, Range<usize>)> {
    let (span, flags) = parse(source, lexer, state, current, string_type)?;
    let value = if flags.has_escape_sequences() {
        flags.into_owned(&source[span.start..span.end])
    } else {
        source[span.start..span.end].to_string()
    };
    return Ok((Value::String(value), span));
}
