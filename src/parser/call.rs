use serde_json::{Number, Value};
use std::ops::Range;

use crate::{
    error::{ErrorInfo, SyntaxError},
    lexer::{Lexer, Parameters, Token},
    parser::{
        ast::{Call, CallTarget, Element, Lines, ParameterValue},
        path, string, ParseState,
    },
    SyntaxResult,
};

/// Indicate if this call statement is being parsed
/// in the context of a statement or block which is used
/// to determine if the `else` keyword is allowed.
#[derive(Eq, PartialEq)]
pub(crate) enum CallParseContext {
    /// Parsing as an open block.
    Block,
    /// Parsing as a raw block.
    Raw,
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

/// Parse a JSON literal value.
fn json_literal<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    current: (Parameters, Range<usize>),
    range: &mut Range<usize>,
) -> SyntaxResult<Value> {
    let (lex, span) = current;
    let value = match lex {
        Parameters::Null => Value::Null,
        Parameters::True => Value::Bool(true),
        Parameters::False => Value::Bool(false),
        Parameters::Number => {
            let num: Number = source[span].parse().unwrap();
            Value::Number(num)
        }
        // NOTE: For string literal values we need to add one
        // NOTE: to the end value as the returned span is the
        // NOTE: inner span and we require the outer span
        // NOTE: (including quotes) for the AST data.
        Parameters::DoubleQuoteString => {
            let (value, span) = string::literal(
                source,
                lexer,
                state,
                (lex, span),
                string::RawLiteralType::Double,
            )?;
            range.end = span.end + 1;
            value
        }
        Parameters::SingleQuoteString => {
            let (value, span) = string::literal(
                source,
                lexer,
                state,
                (lex, span),
                string::RawLiteralType::Single,
            )?;
            range.end = span.end + 1;
            value
        }
        Parameters::StartArray => {
            let (value, span) = string::literal(
                source,
                lexer,
                state,
                (lex, span),
                string::RawLiteralType::Array,
            )?;
            range.end = span.end + 1;
            value
        }
        _ => {
            return Err(SyntaxError::TokenJsonLiteral(
                ErrorInfo::from((source, state)).into(),
            ));
        }
    };

    Ok(value)
}

fn value<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    current: (Parameters, Range<usize>),
) -> SyntaxResult<(ParameterValue<'source>, Option<Token>)> {
    let (lex, span) = current;

    match &lex {
        // Path components
        Parameters::ExplicitThisKeyword
        | Parameters::PathDelimiter
        | Parameters::ExplicitThisDotSlash
        | Parameters::Identifier
        | Parameters::LocalIdentifier
        | Parameters::ParentRef => {
            let (mut path, token) =
                path::parse(source, lexer, state, (lex, span))?;
            if let Some(path) = path.take() {
                Ok((ParameterValue::Path(path), token))
            } else {
                Err(SyntaxError::ExpectedPath(
                    ErrorInfo::from((source, state)).into(),
                ))
            }
        }
        // Open a nested call
        Parameters::StartSubExpression => {
            let (call, token) = sub_expr(source, lexer, state, span)?;
            Ok((ParameterValue::SubExpr(call), token))
        }
        // Literal components
        Parameters::DoubleQuoteString
        | Parameters::SingleQuoteString
        | Parameters::StartArray
        | Parameters::Number
        | Parameters::True
        | Parameters::False
        | Parameters::Null => {
            let mut range = span.clone();
            let line_range = state.line_range();
            let value =
                json_literal(source, lexer, state, (lex, span), &mut range)?;
            Ok((
                ParameterValue::Json {
                    source,
                    value,
                    span: range,
                    line: line_range,
                },
                lexer.next(),
            ))
        }
        _ => {
            println!("Value for unknown token {:?}", &lex);
            return Err(SyntaxError::TokenParameter(
                ErrorInfo::from((source, state)).into(),
            ));
        }
    }
}

fn key_value<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    call: &mut Call<'source>,
    current: (Parameters, Range<usize>),
    context: CallContext,
) -> SyntaxResult<Option<Token>> {
    let (_lex, span) = current;
    let key = &source[span.start..span.end - 1];
    let mut next: Option<Token> = None;

    // FIXME: support absolute path values (paths with leading slash) for hash parameters

    // Consume the first value
    if let Some(token) = lexer.next() {
        match token {
            Token::Parameters(lex, span) => {
                let (value, token) = value(source, lexer, state, (lex, span))?;
                call.add_parameter(key, value);
                next = token;
            }
            _ => {
                return Err(SyntaxError::TokenParameter(
                    ErrorInfo::from((source, state)).into(),
                ));
            }
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
                    return key_value(
                        source,
                        lexer,
                        state,
                        call,
                        (lex, span),
                        context,
                    );
                }
                Parameters::End => {
                    call.exit(span);
                    return Ok(None);
                }
                Parameters::EndSubExpression => {
                    if context == CallContext::SubExpr {
                        call.exit(span);
                        return Ok(lexer.next());
                    } else {
                        return Err(SyntaxError::SubExprNotOpen(
                            ErrorInfo::from((source, state)).into(),
                        ));
                    }
                }
                _ => {
                    return Err(SyntaxError::TokenHashKeyValue(
                        ErrorInfo::from((source, state)).into(),
                    ));
                }
            },
            _ => {
                return Err(SyntaxError::TokenParameter(
                    ErrorInfo::from((source, state)).into(),
                ));
            }
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
) -> SyntaxResult<Option<Token>> {
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
                        return Err(SyntaxError::PartialPosition(
                            ErrorInfo::from((source, state)).into(),
                        ))
                    }
                    Parameters::ElseKeyword => {}
                    // Path components
                    Parameters::ExplicitThisKeyword
                    | Parameters::PathDelimiter
                    | Parameters::ExplicitThisDotSlash
                    | Parameters::Identifier
                    | Parameters::LocalIdentifier
                    | Parameters::StartArray
                    | Parameters::ParentRef => {
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
                            context,
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
                    Parameters::DoubleQuoteString
                    | Parameters::SingleQuoteString
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
                    /*
                    Parameters::PathDelimiter => {
                        let leading_delimiter = &source[span.start..span.end];
                        if leading_delimiter == "/" {
                            // Consume as an absolute path
                            let (value, token) =
                                value(source, lexer, state, (lex, span), true)?;
                            call.add_argument(value);
                            return arguments(
                                source, lexer, state, call, token, context,
                            );
                        } else {
                            return Err(SyntaxError::PathDelimiterNotAllowed(
                                ErrorInfo::from((source, state)).into(),
                            ))
                        }
                    }
                    */
                    Parameters::EndSubExpression => {
                        if context == CallContext::SubExpr {
                            call.exit(span);
                            return Ok(lexer.next());
                        } else {
                            return Err(SyntaxError::SubExprNotOpen(
                                ErrorInfo::from((source, state)).into(),
                            ));
                        }
                    }
                    Parameters::Error => {
                        return Err(SyntaxError::TokenError(
                            String::from("parameters"),
                            ErrorInfo::from((source, state)).into(),
                        ))
                    }
                    Parameters::End => {
                        if context != CallContext::SubExpr {
                            call.exit(span);
                        }
                        return Ok(None);
                    }
                }
            }
            _ => {
                return Err(SyntaxError::TokenParameter(
                    ErrorInfo::from((source, state)).into(),
                ));
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
) -> SyntaxResult<Option<Token>> {
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
                        return Err(SyntaxError::ElseNotAllowed(
                            ErrorInfo::from((source, state)).into(),
                        ));
                    }
                    // Path components
                    Parameters::ExplicitThisKeyword
                    | Parameters::ExplicitThisDotSlash
                    | Parameters::Identifier
                    | Parameters::LocalIdentifier
                    | Parameters::ParentRef
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
                            return Err(SyntaxError::SubExprTargetNotAllowed(
                                ErrorInfo::from((source, state)).into(),
                            ));
                        }

                        let (sub_call, token) =
                            sub_expr(source, lexer, state, span)?;
                        call.set_target(CallTarget::SubExpr(Box::new(
                            sub_call,
                        )));
                        return Ok(token);
                    }
                    Parameters::End => {
                        if !call.has_target() && !call.is_conditional() {
                            return Err(SyntaxError::ExpectedIdentifier(
                                ErrorInfo::from((source, state)).into(),
                            ));
                        }
                        if context != CallContext::SubExpr {
                            call.exit(span);
                        }
                        return Ok(None);
                    }
                    _ => {
                        return Err(SyntaxError::TokenCallTarget(
                            ErrorInfo::from((source, state)).into(),
                        ));
                    }
                }
            }
            _ => {
                return Err(SyntaxError::TokenParameter(
                    ErrorInfo::from((source, state)).into(),
                ));
            }
        }

        next = lexer.next();
    }
    Ok(None)
}

/// Parse the partial and conditional flags.
fn flags<'source>(
    _source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    call: &mut Call<'source>,
    mut next: Option<Token>,
) -> SyntaxResult<Option<Token>> {
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
    open: Range<usize>,
) -> SyntaxResult<(Call<'source>, Option<Token>)> {
    *state.byte_mut() = open.end;

    let mut call = Call::new(source, open, state.line_range());
    let next = lexer.next();
    let next =
        target(source, lexer, state, &mut call, next, CallContext::SubExpr)?;
    let next =
        arguments(source, lexer, state, &mut call, next, CallContext::SubExpr)?;
    if !call.is_closed() {
        return Err(SyntaxError::SubExpressionNotTerminated(
            ErrorInfo::from((source, state)).into(),
        ));
    }

    call.lines_end(state.line());

    Ok((call, next))
}

pub(crate) fn parse<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    open: Range<usize>,
    // TODO: use this to determine whether `else` keyword is legal
    _parse_context: CallParseContext,
) -> SyntaxResult<Call<'source>> {
    *state.byte_mut() = open.end;

    let mut call = Call::new(source, open, state.line_range());
    let next = lexer.next();
    let next = flags(source, lexer, state, &mut call, next)?;

    if call.is_partial() && call.is_conditional() {
        return Err(SyntaxError::MixedPartialConditional(
            ErrorInfo::from((source, state)).into(),
        ));
    }

    let next =
        target(source, lexer, state, &mut call, next, CallContext::Call)?;
    let _next =
        arguments(source, lexer, state, &mut call, next, CallContext::Call)?;

    // FIXME: should we return the next token here so it is consumed ???

    call.lines_end(state.line());

    Ok(call)
}
