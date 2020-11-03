use std::ops::Range;
use std::vec::IntoIter;

use logos::Span;
use serde_json::{Value, Number};

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{Lexer, Parameters, Token},
    parser::{
        arguments,
        ast::{Call, CallTarget, Path, ParameterValue},
        path2, whitespace, ParseState,
    },
};

/// Parse a JSON literal value.
pub(crate) fn json_literal<'source>(
    source: &'source str,
    state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    current: Option<Token>,
) -> Result<(Option<Value>, Option<Token>), SyntaxError<'source>> {
    let mut value: Option<Value> = None;
    if let Some(token) = current {
        match token {
            Token::Parameters(lex, span) => {
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
                        /*
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
                        */

                        let str_value = &source[str_start..str_end];
                        Some(Value::String(str_value.to_string()))
                    }
                    _ => {
                        // FIXME: how to handle this?
                        panic!("Expecting JSON literal token.");
                    }
                }
            }
            _ => panic!("Unexpected json literal token"),
        }
    }

    Ok((value, lexer.next()))
}

fn arguments<'source>(
    source: &'source str,
    state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    mut call: Call<'source>,
    next: Option<Token>,
) -> Result<Option<Call<'source>>, SyntaxError<'source>> {

    println!("Arguments {:?}", next);

    if let Some(token) = next {
        match token {
            Token::Parameters(lex, span) => {
                match &lex {
                    Parameters::WhiteSpace | Parameters::Newline => {
                        if lex == Parameters::Newline {
                            *state.line_mut() += 1;
                        }
                        let next = lexer.next();
                        return arguments(source, state, lexer, call, next);
                    }
                    Parameters::Partial => {
                        panic!("Partial indicator (>) must be the first part of a call statement");
                    }
                    Parameters::ElseKeyword => {}
                    // Path components
                    Parameters::ExplicitThisKeyword
                    | Parameters::ExplicitThisDotSlash
                    | Parameters::Identifier
                    | Parameters::LocalIdentifier
                    | Parameters::ParentRef
                    | Parameters::ArrayAccess => {
                        let (mut path, token) = path2::parse(
                            source,
                            state,
                            lexer,
                            (lex, span),
                        )?;

                        if let Some(path) = path.take() {
                            call.add_argument(ParameterValue::Path(path)) 
                        }

                        return arguments(source, state, lexer, call, token);
                    }
                    // Hash parameters
                    Parameters::HashKey => {}
                    // Open a nested call
                    Parameters::StartSubExpression => {
                        todo!("Parse argument as sub expression");
                    }
                    // Literal components
                    Parameters::StringLiteral
                    | Parameters::Number
                    | Parameters::True
                    | Parameters::False
                    | Parameters::Null => {
                        println!("Parse out a json literal...");
                        let token = Token::Parameters(lex, span);
                        let (mut value, token) = json_literal(source, state, lexer, Some(token))?;
                        if let Some(value) = value.take() {
                            call.add_argument(ParameterValue::Json(value));
                        }
                        return arguments(source, state, lexer, call, token);
                    }
                    Parameters::PathDelimiter => {
                        panic!("Unexpected path delimiter");
                    }
                    Parameters::EndSubExpression => {
                        panic!("Unexpected end of sub expression");
                    }
                    Parameters::Error => {
                        panic!("Unexpected token");
                    }
                    Parameters::End => {
                        call.exit(span);
                        return Ok(Some(call))
                    }
                }
            }
            _ => {
                panic!("Expecting parameter token");
            }
        }
    }

    Ok(None)
}

/// Parse the call target and handle partial flag.
fn target<'source>(
    source: &'source str,
    state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    call: &mut Call<'source>,
    next: Option<Token>,
) -> Result<Option<Token>, SyntaxError<'source>> {

    if let Some(token) = next {
        match token {
            Token::Parameters(lex, span) => {
                match &lex {
                    Parameters::WhiteSpace => {
                        *state.byte_mut() = span.end;
                    }
                    Parameters::Newline => {
                        *state.byte_mut() = span.end;
                        *state.line_mut() += 1;
                    }
                    Parameters::Partial => {
                        // TODO: ensure partial is set before
                        // TODO: any call target/args/hash etc
                        call.set_partial(true);
                    }
                    Parameters::ElseKeyword => {
                        todo!("Got else keyword parsing call target");
                    }
                    // Path components
                    Parameters::ExplicitThisKeyword
                    | Parameters::ExplicitThisDotSlash
                    | Parameters::Identifier
                    | Parameters::LocalIdentifier
                    | Parameters::ParentRef
                    | Parameters::ArrayAccess
                    | Parameters::PathDelimiter => {
                        let (mut path, token) = path2::parse(
                            source,
                            state,
                            lexer,
                            (lex, span),
                        )?;

                        if let Some(path) = path.take() {
                            call.set_target(CallTarget::Path(path));
                            return Ok(token)
                        }
                    }
                    Parameters::StartSubExpression => {
                        todo!("Parse sub expression for call target");
                    }
                    _ => {
                        panic!("Unexpected token parsing call target!");
                    }
                }
            }
            _ => {
                panic!("Expecting parameter token");
            }
        }
    }
    Ok(None)
}

pub(crate) fn parse<'source>(
    source: &'source str,
    state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    open: Span,
) -> Result<Option<Call<'source>>, SyntaxError<'source>> {
    let mut call = Call::new2(source, open);
    let next = lexer.next();
    let next = target(source, state, lexer, &mut call, next)?;
    Ok(arguments(source, state, lexer, call, next)?)
}
