use logos::Span;
use serde_json::{Number, Value};

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{Lexer, Parameters, StringLiteral, Token},
    parser::{
        ast::{Call, CallTarget, ParameterValue},
        path, ParseState,
    },
};

/// Indicate if this call statement is being parsed
/// in the context of a statement or block which is used
/// to determine if the `else` keyword is allowed.
#[derive(Eq, PartialEq)]
pub(crate) enum CallParseContext {
    /// Parsing as an open block.
    Block,
    /// Parsing as a statement out side a block
    Statement,
    /// Parsing as a statement inside a block scope
    /// in which case the `else` keyword should be parsed.
    ScopeStatement,
}

/// Repesents the types of calls that can be parsed.
///
/// Either a top-level call or a sub-expression.
///
/// Sub expressions do not parse partial information and must
/// use a path for the call target.
#[derive(Eq, PartialEq)]
enum CallContext {
    Call,
    SubExpr,
}

/// Parse a quoted string literal value.
fn string_literal<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    current: (Parameters, Span),
) -> Result<Value, SyntaxError<'source>> {
    let (lex, span) = current;
    let str_start = span.end;
    let mut str_end = span.end;

    while let Some(token) = lexer.next() {
        match token {
            Token::StringLiteral(lex, span) => match &lex {
                StringLiteral::End => {
                    let str_value = &source[str_start..str_end];
                    return Ok(Value::String(str_value.to_string()));
                }
                _ => {
                    *state.byte_mut() = span.end;
                    str_end = span.end;
                }
            },
            _ => panic!("Expecting string literal token"),
        }
    }
    panic!("Failed to parse string literal");
}

/// Parse a JSON literal value.
fn json_literal<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
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
            string_literal(source, lexer, state, (lex, span))?
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
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
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
            let (mut path, token) =
                path::parse(source, lexer, state, (lex, span))?;
            if let Some(path) = path.take() {
                return Ok((ParameterValue::Path(path), token));
            }
        }
        // Open a nested call
        Parameters::StartSubExpression => {
            let (call, token) = sub_expr(source, lexer, state, span)?;
            if !call.is_closed() {
                panic!("Sub expression was not terminated");
            }

            return Ok((ParameterValue::SubExpr(call), token));
        }
        // Literal components
        Parameters::StringLiteral
        | Parameters::Number
        | Parameters::True
        | Parameters::False
        | Parameters::Null => {
            let value = json_literal(source, lexer, state, (lex, span))?;
            return Ok((ParameterValue::Json(value), lexer.next()));
        }
        _ => panic!("Unexpected token while parsing value! {:?}", lex),
    }

    panic!("Expecting value!");
}

fn key_value<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    call: &mut Call<'source>,
    current: (Parameters, Span),
) -> Result<Option<Token>, SyntaxError<'source>> {
    let (lex, span) = current;
    let key = &source[span.start..span.end - 1];
    let mut next: Option<Token> = None;

    // Consume the first value
    if let Some(token) = lexer.next() {
        match token {
            Token::Parameters(lex, span) => {
                let (mut value, token) =
                    value(source, lexer, state, (lex, span))?;
                call.add_hash(key, value);
                next = token;
            }
            _ => panic!("Expecting parameter token for key/value pair!"),
        }
    }

    // Read in other key/value pairs
    while let Some(token) = next {
        match token {
            Token::Parameters(lex, span) => match &lex {
                Parameters::WhiteSpace | Parameters::Newline => {
                    if lex == Parameters::Newline {
                        *state.line_mut() += 1;
                    }
                }
                Parameters::HashKey => {
                    return key_value(source, lexer, state, call, (lex, span));
                }
                Parameters::End => {
                    call.exit(span);
                    return Ok(None);
                }
                _ => {
                    panic!("Unexpected parameter token parsing hash parameters")
                }
            },
            _ => panic!("Unexpected token whilst parsing hash parameters"),
        }
        next = lexer.next();
    }
    Ok(None)
}

fn arguments<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    call: &mut Call<'source>,
    next: Option<Token>,
    context: CallContext,
) -> Result<Option<Token>, SyntaxError<'source>> {
    //println!("Arguments {:?}", next);

    if let Some(token) = next {
        match token {
            Token::Parameters(lex, span) => {
                match &lex {
                    Parameters::WhiteSpace | Parameters::Newline => {
                        if lex == Parameters::Newline {
                            *state.line_mut() += 1;
                        }
                        let next = lexer.next();
                        return arguments(
                            source, lexer, state, call, next, context,
                        );
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
                        let (value, token) =
                            value(source, lexer, state, (lex, span))?;
                        call.add_argument(value);
                        return arguments(
                            source, lexer, state, call, token, context,
                        );
                    }
                    // Hash parameters
                    Parameters::HashKey => {
                        return key_value(
                            source,
                            lexer,
                            state,
                            call,
                            (lex, span),
                        );
                    }
                    // Open a nested call
                    Parameters::StartSubExpression => {
                        let (value, token) =
                            value(source, lexer, state, (lex, span))?;
                        call.add_argument(value);
                        return arguments(
                            source, lexer, state, call, token, context,
                        );
                    }
                    // Literal components
                    Parameters::StringLiteral
                    | Parameters::Number
                    | Parameters::True
                    | Parameters::False
                    | Parameters::Null => {
                        // Handle json literal argument values
                        let (value, token) =
                            value(source, lexer, state, (lex, span))?;
                        call.add_argument(value);
                        return arguments(
                            source, lexer, state, call, token, context,
                        );
                    }
                    Parameters::PathDelimiter => {
                        panic!("Unexpected path delimiter");
                    }
                    Parameters::EndSubExpression => {
                        if context == CallContext::SubExpr {
                            call.exit(span);
                            return Ok(lexer.next());
                        } else {
                            panic!("Unexpected end of sub expression");
                        }
                    }
                    Parameters::Error => {
                        panic!("Unexpected token");
                    }
                    Parameters::End => {
                        call.exit(span);
                        return Ok(None);
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

/// Parse the call target.
fn target<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    call: &mut Call<'source>,
    mut next: Option<Token>,
    context: CallContext,
) -> Result<Option<Token>, SyntaxError<'source>> {
    while let Some(token) = next {
        match token {
            Token::Parameters(lex, span) => {
                match &lex {
                    Parameters::WhiteSpace | Parameters::Newline => {
                        if lex == Parameters::Newline {
                            *state.line_mut() += 1;
                        }
                    }
                    Parameters::ElseKeyword => {
                        panic!("Got else keyword parsing call target");
                    }
                    // Path components
                    Parameters::ExplicitThisKeyword
                    | Parameters::ExplicitThisDotSlash
                    | Parameters::Identifier
                    | Parameters::LocalIdentifier
                    | Parameters::ParentRef
                    | Parameters::ArrayAccess
                    | Parameters::PathDelimiter => {
                        let (mut path, token) =
                            path::parse(source, lexer, state, (lex, span))?;

                        if let Some(path) = path.take() {
                            call.set_target(CallTarget::Path(path));
                        }

                        return Ok(token);
                    }
                    Parameters::StartSubExpression => {
                        if context == CallContext::SubExpr {
                            panic!("Sub expressions must use a path or identifier for the target");
                        }

                        let (sub_call, token) =
                            sub_expr(source, lexer, state, span)?;
                        call.set_target(CallTarget::SubExpr(Box::new(
                            sub_call,
                        )));
                        return Ok(token);
                    }
                    Parameters::End => {
                        if !call.has_target() {
                            //panic!("Got end of statement with no call target...");
                            return Err(SyntaxError::EmptyStatement(
                                ErrorInfo::new(
                                    source,
                                    state.file_name(),
                                    SourcePos::from((
                                        state.line(),
                                        state.byte(),
                                    )),
                                ),
                            ));
                        }
                        call.exit(span);
                        return Ok(None);
                    }
                    _ => {
                        panic!(
                            "Unexpected token parsing call target {:?}",
                            lex
                        );
                    }
                }
            }
            _ => {
                panic!("Expecting parameter token");
            }
        }

        next = lexer.next();
    }
    Ok(None)
}

/// Parse the partial and conditional flags.
fn flags<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    call: &mut Call<'source>,
    mut next: Option<Token>,
) -> Result<Option<Token>, SyntaxError<'source>> {
    while let Some(token) = next {
        match token {
            Token::Parameters(lex, span) => match &lex {
                Parameters::WhiteSpace | Parameters::Newline => {
                    if lex == Parameters::Newline {
                        *state.line_mut() += 1;
                    }
                }
                Parameters::Partial => {
                    call.set_partial(true);
                    return Ok(lexer.next());
                }
                Parameters::ElseKeyword => {
                    call.set_conditional(true);
                    return Ok(lexer.next());
                }
                _ => return Ok(Some(Token::Parameters(lex, span))),
            },
            _ => return Ok(Some(token)),
        }
        next = lexer.next();
    }
    Ok(None)
}

pub(crate) fn sub_expr<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    open: Span,
    //parse_context: CallParseContext,
) -> Result<(Call<'source>, Option<Token>), SyntaxError<'source>> {
    *state.byte_mut() = open.end;

    let mut call = Call::new(source, open);
    let next = lexer.next();
    let next =
        target(source, lexer, state, &mut call, next, CallContext::SubExpr)?;
    let next =
        arguments(source, lexer, state, &mut call, next, CallContext::SubExpr)?;
    if !call.is_closed() {
        panic!("Sub expression statement was not terminated");
    }
    Ok((call, next))
}

pub(crate) fn parse<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    open: Span,
    parse_context: CallParseContext,
) -> Result<Call<'source>, SyntaxError<'source>> {
    *state.byte_mut() = open.end;

    let mut call = Call::new(source, open);
    let next = lexer.next();
    let next = flags(source, lexer, state, &mut call, next)?;

    if call.is_partial() && call.is_conditional() {
        panic!("Partials and conditionals may not be combined.");
    }

    let next =
        target(source, lexer, state, &mut call, next, CallContext::Call)?;
    let next =
        arguments(source, lexer, state, &mut call, next, CallContext::Call)?;
    // FIXME: we should return the next token here so it is consumed ???
    if !call.is_closed() {
        //println!("{:?}", call);
        panic!("Call statement was not terminated");
    }
    Ok(call)
}
