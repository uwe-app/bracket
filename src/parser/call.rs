use std::ops::Range;
use std::vec::IntoIter;

use logos::Span;
use serde_json::{Value, Number};

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{Lexer, Parameters, Token, StringLiteral},
    parser::{
        arguments,
        ast::{Call, CallTarget, Path, ParameterValue},
        path2, whitespace, ParseState,
    },
};


/// Parse a quoted string literal value.
fn string_literal<'source>(
    source: &'source str,
    state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    current: (Parameters, Span),
) -> Result<Value, SyntaxError<'source>> {
    let (lex, span) = current;
    let str_start = span.end;
    let mut str_end = span.end;

    while let Some(token) = lexer.next() {
        match token {
            Token::StringLiteral(lex, span) => {
                match &lex {
                    StringLiteral::End => {
                        let str_value = &source[str_start..str_end];
                        return Ok(Value::String(str_value.to_string()))
                    }
                    _ => {
                        *state.byte_mut() = span.end;
                        str_end = span.end;
                    }
                } 
            }
            _ => panic!("Expecting string literal token"),
        } 
    }
    panic!("Failed to parse string literal");
}

/// Parse a JSON literal value.
fn json_literal<'source>(
    source: &'source str,
    state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    current: (Parameters, Span),
) -> Result<Value, SyntaxError<'source>> {

    let (lex, span) = current;
    let value = match lex {
        Parameters::Null => Value::Null,
        Parameters::True => Value::Bool(true),
        Parameters::False => Value::Bool(false),
        Parameters::Number => {
            let num: Number = source[span].parse().unwrap();
            Value::Number(num)
        }
        Parameters::StringLiteral => {
            string_literal(source, state, lexer, (lex, span))?
            //let str_start = span.end;
            //let mut str_end = span.end;
            //let str_value = &source[str_start..str_end];
            //Some(Value::String(str_value.to_string()))
        }
        _ => {
            // FIXME: how to handle this?
            panic!("Expecting JSON literal token.");
        }
    };

    Ok(value)
}

fn value<'source>(
    source: &'source str,
    state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    current: (Parameters, Span),
) -> Result<(ParameterValue<'source>, Option<Token>), SyntaxError<'source>> {

    let (lex, span) = current;

    match &lex {
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
                return Ok((ParameterValue::Path(path), token));
            }
        }
        // Open a nested call
        Parameters::StartSubExpression => {
            todo!("Parse value as sub expression");
        }
        // Literal components
        Parameters::StringLiteral
        | Parameters::Number
        | Parameters::True
        | Parameters::False
        | Parameters::Null => {
            let value = json_literal(source, state, lexer, (lex, span))?;
            return Ok((ParameterValue::Json(value), lexer.next()));
        }
        _ => panic!("Unexpected token while parsing value!"),
    }

    panic!("Expecting value!");
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
                        // Handle path arguments values
                        let (mut value, token) = value(source, state, lexer, (lex, span))?;
                        call.add_argument(value);
                        return arguments(source, state, lexer, call, token);
                    }
                    // Hash parameters
                    Parameters::HashKey => {
                        println!("Got hash parameters to pass...");
                    }
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
                        // Handle json literal argument values
                        let (mut value, token) = value(source, state, lexer, (lex, span))?;
                        call.add_argument(value);
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
    let next = arguments(source, state, lexer, call, next)?;
    Ok(next)
}
