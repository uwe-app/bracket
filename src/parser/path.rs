use std::ops::Range;

use crate::{
    error::{ErrorInfo, SyntaxError},
    lexer::{lex, Lexer, Parameters, Token},
    parser::{
        ast::{Component, ComponentType, Path, RawIdType},
        string::{self, RawLiteral},
        ParseState,
    },
    SyntaxResult,
};

fn is_path_component(lex: &Parameters) -> bool {
    match lex {
        Parameters::ExplicitThisKeyword
        | Parameters::ExplicitThisDotSlash
        | Parameters::ParentRef
        | Parameters::Identifier
        | Parameters::LocalIdentifier
        | Parameters::PathDelimiter
        | Parameters::StartArray
        | Parameters::SingleQuoteString
        | Parameters::DoubleQuoteString => true,
        _ => false,
    }
}

fn to_component<'source>(
    source: &'source str,
    state: &mut ParseState,
    lex: &Parameters,
    span: Range<usize>,
    raw_id: Option<RawLiteral>,
) -> SyntaxResult<Component<'source>> {
    let value = if let Some(ref raw) = raw_id {
        if raw.has_escape_sequences() {
            Some(raw.into_owned(&source[span.clone()]))
        } else {
            None
        }
    } else {
        None
    };

    let kind = match &lex {
        Parameters::ExplicitThisKeyword => ComponentType::ThisKeyword,
        Parameters::ExplicitThisDotSlash => ComponentType::ThisDotSlash,
        Parameters::ParentRef => ComponentType::Parent,
        Parameters::Identifier => ComponentType::Identifier,
        Parameters::LocalIdentifier => ComponentType::LocalIdentifier,
        Parameters::PathDelimiter => ComponentType::Delimiter,
        Parameters::SingleQuoteString => {
            ComponentType::RawIdentifier(RawIdType::Single)
        }
        Parameters::DoubleQuoteString => {
            ComponentType::RawIdentifier(RawIdType::Double)
        }
        Parameters::StartArray => {
            ComponentType::RawIdentifier(RawIdType::Array)
        }
        _ => return Err(
            SyntaxError::ComponentType(
                ErrorInfo::from((source, state)).into()))
    };

    Ok(Component::new(source, kind, span, value))
}

fn parents<'source>(
    _state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    path: &mut Path,
) -> Option<Token> {
    path.set_parents(1);
    while let Some(token) = lexer.next() {
        match &token {
            Token::Parameters(lex, _) => match &lex {
                Parameters::ParentRef => {
                    path.set_parents(path.parents() + 1);
                }
                _ => return Some(token),
            },
            _ => return Some(token),
        }
    }
    None
}

pub(crate) fn components<'source>(
    source: &'source str,
    state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    path: &mut Path<'source>,
    mut wants_delimiter: bool,
) -> SyntaxResult<Option<Token>> {
    while let Some(token) = lexer.next() {

        if token.is_newline() {
            *state.line_mut() += 1;
            // Paths are terminated if we hit a newline!
            return Ok(lexer.next())
        }

        match token {
            Token::Parameters(lex, mut span) => {
                *state.byte_mut() = span.start;

                let mut raw_id: Option<RawLiteral> = None;

                if lex == Parameters::End {
                    return Ok(Some(Token::Parameters(lex, span)));
                }

                if is_path_component(&lex) {
                    match &lex {
                        Parameters::ExplicitThisKeyword
                        | Parameters::ExplicitThisDotSlash => {
                            *state.byte_mut() = span.start;
                            return Err(
                                SyntaxError::UnexpectedPathExplicitThis(
                                    ErrorInfo::from((source, state)).into(),
                                ),
                            );
                        }
                        Parameters::ParentRef => {
                            *state.byte_mut() = span.start;
                            return Err(SyntaxError::UnexpectedPathParent(
                                ErrorInfo::from((source, state)).into(),
                            ));
                        }
                        Parameters::LocalIdentifier => {
                            *state.byte_mut() = span.start;
                            return Err(SyntaxError::UnexpectedPathLocal(
                                ErrorInfo::from((source, state)).into(),
                            ));
                        }
                        Parameters::SingleQuoteString => {
                            // Override the span to the inner string value
                            let (inner, flags) = string::parse(
                                source,
                                lexer,
                                state,
                                (lex, span),
                                string::RawLiteralType::Single,
                            )?;

                            span = inner;
                            raw_id = Some(flags);
                        }
                        Parameters::DoubleQuoteString => {
                            // Override the span to the inner string value
                            let (inner, flags) = string::parse(
                                source,
                                lexer,
                                state,
                                (lex, span),
                                string::RawLiteralType::Double,
                            )?;

                            span = inner;
                            raw_id = Some(flags);
                        }
                        Parameters::StartArray => {
                            // Override the span to the inner string value
                            let (inner, flags) = string::parse(
                                source,
                                lexer,
                                state,
                                (lex, span),
                                string::RawLiteralType::Array,
                            )?;

                            span = inner;
                            raw_id = Some(flags);
                        }
                        _ => {}
                    }

                    if wants_delimiter {
                        match &lex {
                            Parameters::PathDelimiter => {
                                wants_delimiter = false;
                                continue;
                            }
                            _ => {
                                *state.byte_mut() = span.start;
                                return Err(
                                    SyntaxError::ExpectedPathDelimiter(
                                        ErrorInfo::from((source, state)).into(),
                                    ),
                                );
                            }
                        }
                    } else {
                        match &lex {
                            Parameters::PathDelimiter => {
                                *state.byte_mut() = span.start;
                                return Err(
                                    SyntaxError::UnexpectedPathDelimiter(
                                        ErrorInfo::from((source, state)).into(),
                                    ),
                                );
                            }
                            _ => {}
                        }
                    }

                    path.add_component(to_component(
                        source, state, &lex, span, raw_id,
                    )?);
                    wants_delimiter = true;
                } else {
                    return Ok(Some(Token::Parameters(lex, span)));
                }
            }
            _ => return Ok(Some(token)),
        }
    }

    Ok(None)
}

pub(crate) fn parse<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    current: (Parameters, Range<usize>),
) -> SyntaxResult<(Option<Path<'source>>, Option<Token>)> {
    let (lex, span) = current;
    let mut path = Path::new(source, span.clone(), state.line_range());

    let mut next: Option<Token> = Some(Token::Parameters(lex, span));
    match &lex {
        // Cannot start with a path delimiter
        Parameters::PathDelimiter => {
            return Err(SyntaxError::UnexpectedPathDelimiter(
                ErrorInfo::from((source, state)).into(),
            ));
        }
        // Count parent references
        Parameters::ParentRef => {
            next = parents(state, lexer, &mut path);
        }
        _ => {}
    }

    while let Some(token) = next {

        match token {
            Token::Parameters(lex, span) => {
                *state.byte_mut() = span.start;

                if is_path_component(&lex) {
                    let component = to_component(source, state, &lex, span, None)?;
                    // Flag as a path that should be resolved from the root object
                    if path.is_empty() && component.is_root() {
                        path.set_root(true);
                    }

                    if component.is_explicit() {
                        path.set_explicit(true);
                    }

                    if component.is_local() && path.parents() > 0 {
                        return Err(
                            SyntaxError::UnexpectedPathParentWithLocal(
                                ErrorInfo::from((source, state)).into(),
                            ),
                        );
                    }

                    if component.is_explicit() && path.parents() > 0 {
                        return Err(
                            SyntaxError::UnexpectedPathParentWithExplicit(
                                ErrorInfo::from((source, state)).into(),
                            ),
                        );
                    }

                    let wants_delimiter = !component.is_explicit_dot_slash();
                    path.add_component(component);

                    let next = components(
                        source,
                        state,
                        lexer,
                        &mut path,
                        wants_delimiter,
                    )?;

                    if path.is_empty() {
                        return Err(
                            SyntaxError::EmptyPath(
                                ErrorInfo::from((source, state)).into()))
                    }

                    return Ok((Some(path), next));
                }
            }
            _ => return Err(
                SyntaxError::TokenParameterPath(
                    ErrorInfo::from((source, state)).into()))
        }

        next = lexer.next();
    }

    Ok((None, next))
}

pub(crate) fn from_str<'source>(
    source: &'source str,
) -> SyntaxResult<Option<Path<'source>>> {
    let mut lexer = lex(source);
    lexer.set_parameters_mode();

    let mut state: ParseState = ParseState::new();

    if let Some(token) = lexer.next() {
        match token {
            Token::Parameters(lex, span) => {
                let (path, _) =
                    parse(source, &mut lexer, &mut state, (lex, span))?;
                return Ok(path);
            }
            _ => return Err(
                SyntaxError::TokenParameterPath(
                    ErrorInfo::from((source, &mut state)).into()))
        }
    }

    Ok(None)
}
