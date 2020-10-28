use std::vec::IntoIter;

use logos::Span;

use crate::{
    error::{SyntaxError},
    lexer::{
        ast::{
            Call, ParameterValue, 
        },
        grammar::Parameters,
    },
};

use super::{json_literal, path, whitespace};

fn parse_value<'source>(
    source: &'source str,
    file_name: &str,
    iter: &mut IntoIter<(Parameters, Span)>,
    byte_offset: &mut usize,
    line: &mut usize,
    current: Option<(Parameters, Span)>,
) -> Result<
    (Option<ParameterValue<'source>>, Option<(Parameters, Span)>),
    SyntaxError<'source>,
> {
    let mut value: Option<ParameterValue> = None;
    let mut next: Option<(Parameters, Span)> = None;
    if let Some((lex, span)) = current {
        match &lex {
            Parameters::Null
            | Parameters::True
            | Parameters::False
            | Parameters::Number
            | Parameters::StringLiteral => {
                let (literal, next_token) = json_literal::parse(
                    source,
                    iter,
                    byte_offset,
                    line,
                    Some((lex, span)),
                )?;

                value = literal.map(ParameterValue::Json);
                next = next_token;
            }
            // TODO: parse array literals
            // TODO: parse object literals
            _ => {
                let (path, next_token) = path::parse(
                    source,
                    file_name,
                    iter,
                    byte_offset,
                    line,
                    Some((lex, span)),
                )?;

                value = path.map(ParameterValue::Path);
                next = next_token
            }
        }
    }

    Ok((value, next))
}

fn parse_hash_map<'source>(
    source: &'source str,
    file_name: &str,
    iter: &mut IntoIter<(Parameters, Span)>,
    byte_offset: &mut usize,
    line: &mut usize,
    call: &mut Call<'source>,
    current: (Parameters, Span),
) -> Result<Option<(Parameters, Span)>, SyntaxError<'source>> {
    let (lex, span) = current;

    let key = &source[span.start..span.end - 1];
    if let Some((lex, span)) = iter.next() {
        let (mut value, next) = parse_value(
            source,
            file_name,
            iter,
            byte_offset,
            line,
            Some((lex, span)),
        )?;

        if let Some(arg) = value.take() {
            call.add_hash(key, arg);
        }

        let next = whitespace::parse(iter, byte_offset, line);
        if let Some((lex, span)) = next {
            match &lex {
                Parameters::HashKey => {
                    return parse_hash_map(
                        source,
                        file_name,
                        iter,
                        byte_offset,
                        line,
                        call,
                        (lex, span),
                    );
                }
                _ => {}
            }
        }
    }

    Ok(iter.next())
}

pub(crate) fn parse<'source>(
    source: &'source str,
    file_name: &str,
    iter: &mut IntoIter<(Parameters, Span)>,
    byte_offset: &mut usize,
    line: &mut usize,
    call: &mut Call<'source>,
) -> Result<Option<(Parameters, Span)>, SyntaxError<'source>> {
    let next = whitespace::parse(iter, byte_offset, line);
    if let Some((lex, span)) = next {
        match &lex {
            Parameters::HashKey => {
                return parse_hash_map(
                    source,
                    file_name,
                    iter,
                    byte_offset,
                    line,
                    call,
                    (lex, span),
                );
            }
            _ => {}
        }

        let (mut value, next) = parse_value(
            source,
            file_name,
            iter,
            byte_offset,
            line,
            Some((lex, span)),
        )?;

        if let Some(arg) = value.take() {
            call.add_argument(arg);
        }

        return parse(source, file_name, iter, byte_offset, line, call);
    }

    Ok(iter.next())
}

