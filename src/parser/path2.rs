use std::vec::IntoIter;

use logos::Span;

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{Lexer, Parameters, Token},
    parser::{
        ast::{Component, ComponentType, Path},
        ParseState,
    },
};

fn is_path_component(lex: &Parameters) -> bool {
    match lex {
        Parameters::ExplicitThisKeyword
        | Parameters::ExplicitThisDotSlash
        | Parameters::ParentRef
        | Parameters::Identifier
        | Parameters::LocalIdentifier
        | Parameters::PathDelimiter
        | Parameters::ArrayAccess => true,
        _ => false,
    }
}

fn component_type(lex: &Parameters) -> ComponentType {
    match lex {
        Parameters::ExplicitThisKeyword => ComponentType::ThisKeyword,
        Parameters::ExplicitThisDotSlash => ComponentType::ThisDotSlash,
        Parameters::ParentRef => ComponentType::Parent,
        Parameters::Identifier => ComponentType::Identifier,
        Parameters::LocalIdentifier => ComponentType::LocalIdentifier,
        Parameters::PathDelimiter => ComponentType::Delimiter,
        Parameters::ArrayAccess => ComponentType::ArrayAccess,
        _ => panic!("Expecting component parameter in parser"),
    }
}

fn parents<'source>(
    state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    path: &mut Path,
) -> Option<Token> {
    path.set_parents(1);
    while let Some(token) = lexer.next() {
        match &token {
            Token::Parameters(lex, span) => match &lex {
                Parameters::ParentRef => {
                    path.add_parent();
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
) -> Result<Option<Token>, SyntaxError<'source>> {
    while let Some(token) = lexer.next() {
        match token {
            Token::Parameters(lex, span) => {
                *state.byte_mut() = span.start;

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
                                    ErrorInfo::new(
                                        source,
                                        state.file_name(),
                                        SourcePos::from((
                                            state.line(),
                                            state.byte(),
                                        )),
                                    ),
                                ),
                            );
                        }
                        Parameters::ParentRef => {
                            *state.byte_mut() = span.start;
                            return Err(SyntaxError::UnexpectedPathParent(
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
                        Parameters::LocalIdentifier => {
                            *state.byte_mut() = span.start;
                            return Err(SyntaxError::UnexpectedPathLocal(
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
                                        ErrorInfo::new(
                                            source,
                                            state.file_name(),
                                            SourcePos::from((
                                                state.line(),
                                                state.byte(),
                                            )),
                                        ),
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
                                        ErrorInfo::new(
                                            source,
                                            state.file_name(),
                                            SourcePos::from((
                                                state.line(),
                                                state.byte(),
                                            )),
                                        ),
                                    ),
                                );
                            }
                            _ => {}
                        }
                    }
                    path.add_component(Component(
                        source,
                        component_type(&lex),
                        span,
                    ));
                    wants_delimiter = true;
                } else {
                    break;
                }
            }
            _ => return Ok(Some(token)),
        }
    }

    Ok(None)
}

pub(crate) fn parse<'source>(
    source: &'source str,
    state: &mut ParseState,
    lexer: &mut Lexer<'source>,
    current: (Parameters, Span),
) -> Result<(Option<Path<'source>>, Option<Token>), SyntaxError<'source>> {
    let (lex, span) = current;
    let mut path = Path::new(source);

    let mut next: Option<Token> = Some(Token::Parameters(lex, span));
    match &lex {
        // Cannot start with a path delimiter
        Parameters::PathDelimiter => {
            return Err(SyntaxError::UnexpectedPathDelimiter(ErrorInfo::new(
                source,
                state.file_name(),
                SourcePos::from((state.line(), state.byte())),
            )));
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
                    let component =
                        Component(source, component_type(&lex), span);
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
                                ErrorInfo::new(
                                    source,
                                    state.file_name(),
                                    SourcePos::from((
                                        state.line(),
                                        state.byte(),
                                    )),
                                ),
                            ),
                        );
                    }

                    if component.is_explicit() && path.parents() > 0 {
                        return Err(
                            SyntaxError::UnexpectedPathParentWithExplicit(
                                ErrorInfo::new(
                                    source,
                                    state.file_name(),
                                    SourcePos::from((
                                        state.line(),
                                        state.byte(),
                                    )),
                                ),
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
                    return Ok((Some(path), next));
                }
            }
            _ => panic!("Expected parameter token"),
        }
        next = lexer.next();
    }

    Ok((None, next))
}
