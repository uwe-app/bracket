use std::ops::Range;
use std::vec::IntoIter;

use logos::Span;

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{Lexer, Parameters, Token},
    parser::{
        arguments,
        ast::{Call, CallTarget, Path},
        path, path2, whitespace, ParameterCache, ParseState,
    },
};

type SubExpression = Vec<(Parameters, Span)>;

/// Collect sub expression tokens.
fn sub_expr(
    iter: &mut IntoIter<(Parameters, Span)>,
) -> (SubExpression, Option<Span>) {
    let mut sub_expr: Vec<(Parameters, Span)> = Vec::new();
    while let Some((lex, span)) = iter.next() {
        match &lex {
            Parameters::EndSubExpression => {
                return (sub_expr, Some(span));
            }
            _ => {
                sub_expr.push((lex, span));
            }
        }
    }
    (sub_expr, None)
}

/// Parse a sub expression.
pub(crate) fn parse_sub_expr<'source>(
    source: &'source str,
    iter: &mut IntoIter<(Parameters, Span)>,
    state: &mut ParseState,
    current: (Parameters, Span),
) -> Result<(Call<'source>, Option<(Parameters, Span)>), SyntaxError<'source>> {
    let (lex, span) = current;
    let stmt_start = span.clone();
    let (mut sub_expr, stmt_end) = sub_expr(iter);

    if stmt_end.is_none() {
        *state.byte_mut() = stmt_start.end;
        if !sub_expr.is_empty() {
            let (_, last_span) = sub_expr.pop().unwrap();
            *state.byte_mut() = last_span.end;
        }
        return Err(SyntaxError::OpenSubExpression(ErrorInfo::new_notes(
            source,
            state.file_name(),
            SourcePos::from((state.line(), state.byte())),
            vec!["requires closing parenthesis ')'".to_string()],
        )));
    }

    // NOTE: must advance past the start sub expresion token!
    //Ok((iter.next(), stmt_start, sub_expr, stmt_end))
    let next = iter.next();
    parse_call(
        source,
        &mut sub_expr.into_iter(),
        state,
        next,
        false,
        stmt_start,
        stmt_end,
    )
}

fn parse_call_target<'source>(
    source: &'source str,
    iter: &mut IntoIter<(Parameters, Span)>,
    state: &mut ParseState,
    current: Option<(Parameters, Span)>,
    call: &mut Call<'source>,
) -> Result<Option<(Parameters, Span)>, SyntaxError<'source>> {
    if let Some((lex, span)) = current {
        let next = match &lex {
            Parameters::StartSubExpression => {
                let (sub_call, next) =
                    parse_sub_expr(source, iter, state, (lex, span))?;
                call.set_target(CallTarget::SubExpr(Box::new(sub_call)));
                next
            }
            _ => {
                let (mut path, next) =
                    path::parse(source, iter, state, Some((lex, span)))?;

                if let Some(path) = path.take() {
                    call.set_target(CallTarget::Path(path));
                }
                next
            }
        };

        return Ok(next);
    }

    Ok(None)
}

fn parse_call<'source>(
    source: &'source str,
    iter: &mut IntoIter<(Parameters, Span)>,
    state: &mut ParseState,
    current: Option<(Parameters, Span)>,
    partial: bool,
    stmt_start: Range<usize>,
    stmt_end: Option<Range<usize>>,
) -> Result<(Call<'source>, Option<(Parameters, Span)>), SyntaxError<'source>> {
    let mut call = Call::new(source, partial, stmt_start);
    let next = parse_call_target(source, iter, state, current, &mut call)?;
    call.exit(stmt_end);
    Ok((call, next))
}

/// Determine if this statement is a partial reference.
fn partial<'source>(
    source: &'source str,
    iter: &mut IntoIter<(Parameters, Span)>,
    state: &mut ParseState,
    current: Option<(Parameters, Span)>,
) -> (bool, Option<(Parameters, Span)>) {
    if let Some((lex, span)) = current {
        match lex {
            Parameters::Partial => {
                let next = whitespace::parse(iter, state);
                return (true, next);
            }
            _ => {
                return (false, Some((lex, span)));
            }
        }
    }
    (false, None)
}

pub(crate) fn call<'source>(
    source: &'source str,
    iter: &mut IntoIter<(Parameters, Span)>,
    state: &mut ParseState,
    current: Option<(Parameters, Span)>,
    partial: bool,
    stmt_start: Range<usize>,
    stmt_end: Option<Range<usize>>,
) -> Result<Call<'source>, SyntaxError<'source>> {
    let (mut call, next) = parse_call(
        source, iter, state, current, partial, stmt_start, stmt_end,
    )?;

    // Partials must be simple identifiers or sub expressions.
    if partial {
        match call.target() {
            CallTarget::Path(ref path) => {
                if !path.is_simple() {
                    return Err(SyntaxError::PartialSimpleIdentifier(
                        ErrorInfo::new(
                            source,
                            state.file_name(),
                            SourcePos::from((state.line(), state.byte())),
                        ),
                    ));
                }
            }
            _ => {}
        }
    }

    //if call.is_empty() {
    //return Err(SyntaxError::ExpectedIdentifier(self.err_info(
    //source,
    //line,
    //byte_offset,
    //None,
    //)));
    //}

    // FIXME: check for empty sub expressions too ^^^^^^^^^
    match call.target() {
        CallTarget::Path(ref path) => {
            if path.is_empty() {
                return Err(SyntaxError::ExpectedIdentifier(ErrorInfo::new(
                    source,
                    state.file_name(),
                    SourcePos::from((state.line(), state.byte())),
                )));
            }
        }
        _ => {}
    }

    /*
    match context {
        ParameterContext::Block => {
            if !call.path().is_simple() {
                panic!("Blocks require a simple identifier, not a path!");
            }
        }
        ParameterContext::Statement => {
            // TODO: validate statement paths?
        }
    }
    */

    arguments::parse(source, iter, state, &mut call)?;
    //println!("Arguments {:?}", call.arguments());

    Ok(call)
}

/*
pub(crate) fn parse_path<'source>(
    source: &'source str,
    state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    first: (Parameters, Span),
) -> Result<Path<'source>, SyntaxError<'source>> {
    let path = Path::new(source);

    while let Some(token) = lexer.next() {
        match token {
            Token::Parameters(lex, span) => {
                println!("Parsing path part {:?}", lex);
                //match &lex {
                    ////println!("Parsing path part...");
                //}
            }
            _ => panic!("Unexpected token whilst parsing path"),
        }
    }

    Ok(path)
}
*/

fn advance<'source>(
    source: &'source str,
    state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    mut call: Call<'source>,
    next: Option<Token>,
) -> Result<Option<Call<'source>>, SyntaxError<'source>> {

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
                    Parameters::ElseKeyword => {}
                    // Path components
                    Parameters::ExplicitThisKeyword
                    | Parameters::ExplicitThisDotSlash
                    | Parameters::Identifier
                    | Parameters::LocalIdentifier
                    | Parameters::ParentRef
                    | Parameters::ArrayAccess
                    | Parameters::PathDelimiter => {
                        if !call.has_target() {
                            let (mut path, token) = path2::parse(
                                source,
                                state,
                                lexer,
                                (lex, span),
                            )?;

                            if let Some(path) = path.take() {
                                call.set_target(CallTarget::Path(path));
                            }

                            return advance(source, state, lexer, call, token)
                        }
                    }
                    // Hash parameters
                    Parameters::HashKey => {}
                    // Open a nested call
                    Parameters::StartSubExpression => {}
                    // Literal components
                    Parameters::StringLiteral
                    | Parameters::Number
                    | Parameters::True
                    | Parameters::False
                    | Parameters::Null => {}
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

pub(crate) fn parse<'source>(
    source: &'source str,
    state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    open: Span,
) -> Result<Option<Call<'source>>, SyntaxError<'source>> {
    let mut call = Call::new2(source, open);
    let next = lexer.next();
    Ok(advance(source, state, lexer, call, next)?)
}
