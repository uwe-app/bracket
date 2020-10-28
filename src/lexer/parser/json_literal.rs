use std::vec::IntoIter;

use serde_json::{Number, Value};

use logos::Span;

use crate::{
    error::{SyntaxError},
    lexer::grammar::{Parameters, StringLiteral},
};

use super::ParseState;

/// Parse a JSON literal value.
pub(crate) fn parse<'source>(
    source: &'source str,
    iter: &mut IntoIter<(Parameters, Span)>,
    state: &mut ParseState,
    current: Option<(Parameters, Span)>,
) -> Result<(Option<Value>, Option<(Parameters, Span)>), SyntaxError<'source>> {
    let mut value: Option<Value> = None;
    if let Some((lex, span)) = current {
        //println!("Parameter lex {:?}", lex);

        value = match lex {
            Parameters::Null => Some(Value::Null),
            Parameters::True => Some(Value::Bool(true)),
            Parameters::False => Some(Value::Bool(false)),
            Parameters::Number => {
                let num: Number = source[span].parse().unwrap();
                Some(Value::Number(num))
            }
            Parameters::StringLiteral => {
                let str_start = span.end;
                let mut str_end = span.end;
                while let Some((lex, span)) = iter.next() {
                    match lex {
                        Parameters::StringToken(s) => match s {
                            StringLiteral::End => {
                                break;
                            }
                            _ => {
                                *state.byte_mut() = span.end;
                                str_end = span.end;
                            }
                        },
                        _ => {
                            panic!("Expected string token!");
                        }
                    }
                }

                let str_value = &source[str_start..str_end];
                Some(Value::String(str_value.to_string()))
            }
            _ => {
                // FIXME: how to handle this?
                panic!("Expecting JSON literal token.");
            }
        }
    }

    Ok((value, iter.next()))
}
