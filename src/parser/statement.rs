use std::ops::Range;
use std::vec::IntoIter;

use logos::Span;

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::Parameters,
    parser::{
        ast::{Call, CallTarget},
        arguments, path, whitespace, ParameterCache, ParseState
    }
};

/// Collect sub expression tokens.
fn sub_expr(
    iter: &mut IntoIter<(Parameters, Span)>,
) -> (Vec<(Parameters, Span)>, Option<Range<usize>>) {
    //let stmt_end: Option<Range<usize>> = None;

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
                let stmt_start = span.clone();

                let (mut sub_expr, stmt_end) = sub_expr(iter);

                if stmt_end.is_none() {
                    *state.byte_mut() = stmt_start.end;
                    if !sub_expr.is_empty() {
                        let (_, last_span) = sub_expr.pop().unwrap();
                        *state.byte_mut() = last_span.end;
                    }
                    return Err(SyntaxError::OpenSubExpression(
                        ErrorInfo::new_notes(
                            source,
                            state.file_name(),
                            SourcePos::from((state.line(), state.byte())),
                            vec!["requires closing parenthesis ')'"],
                        ),
                    ));
                }

                // NOTE: must advance past the start sub expresion token!
                let next = iter.next();

                let (sub_call, next) = parse_call(
                    source,
                    &mut sub_expr.into_iter(),
                    state,
                    next,
                    false,
                    stmt_start,
                    stmt_end.unwrap(),
                )?;

                call.set_target(CallTarget::SubExpr(Box::new(sub_call)));
                next
            }
            _ => {
                let (mut path, next) = path::parse(
                    source,
                    iter,
                    state,
                    Some((lex, span)),
                )?;

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
    stmt_end: Range<usize>,
) -> Result<(Call<'source>, Option<(Parameters, Span)>), SyntaxError<'source>>
{
    let mut call = Call::new(source, partial, stmt_start, stmt_end);
    let next = parse_call_target(
        source,
        iter,
        state,
        current,
        &mut call,
    )?;

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

pub(crate) fn parse<'source>(
    source: &'source str,
    state: &mut ParseState,
    //file_name: &str,
    //line: &mut usize,
    //byte_offset: &mut usize,
    statement: ParameterCache,
) -> Result<Call<'source>, SyntaxError<'source>> {
    let context = statement.context.clone();
    let stmt_start = statement.start.clone();
    let stmt_end = statement.end.clone();
    let mut iter = statement.tokens.into_iter();

    // Position as byte offset for syntax errors
    *state.byte_mut() = stmt_start.end;

    let next = whitespace::parse(&mut iter, state);

    //println!("Next {:?}", next);

    if next.is_none() {
        return Err(SyntaxError::EmptyStatement(ErrorInfo::new(
            source,
            state.file_name(),
            SourcePos::from((state.line(), state.byte())),
        )));
    }

    //println!("After leading whitespce {:?}", next);
    let (partial, next) =
        partial(source, &mut iter, state, next);
    //println!("After partial parse {:?} {:?}", partial, &next);
    if partial && next.is_none() {
        return Err(SyntaxError::PartialIdentifier(ErrorInfo::new(
            source,
            state.file_name(),
            SourcePos::from((state.line(), state.byte())),
        )));
    }

    let (mut call, next) = parse_call(
        source,
        &mut iter,
        state,
        next,
        partial,
        stmt_start,
        stmt_end,
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
                return Err(SyntaxError::ExpectedIdentifier(
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

    arguments::parse(source, &mut iter, state, &mut call)?;
    println!("Arguments {:?}", call.arguments());

    Ok(call)
}


