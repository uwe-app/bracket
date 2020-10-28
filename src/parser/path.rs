use std::vec::IntoIter;

use logos::Span;

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::Parameters,
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
        | Parameters::PathDelimiter => true,
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
        _ => panic!("Expecting component parameter in parser"),
    }
}

pub(crate) fn parse<'source>(
    source: &'source str,
    iter: &mut IntoIter<(Parameters, Span)>,
    state: &mut ParseState,
    current: Option<(Parameters, Span)>,
) -> Result<
    (Option<Path<'source>>, Option<(Parameters, Span)>),
    SyntaxError<'source>,
> {
    let mut result: Option<Path> = None;
    let mut next: Option<(Parameters, Span)> = None;

    if let Some((mut lex, mut span)) = current {
        let mut path = Path::new(source);
        if is_path_component(&lex) {
            *state.byte_mut() = span.end;

            // Consume parent references
            match &lex {
                Parameters::ParentRef => {
                    let mut parents = 1;
                    while let Some((next_lex, next_span)) = iter.next() {
                        match &next_lex {
                            Parameters::ParentRef => {
                                parents += 1;
                            }
                            _ => {
                                lex = next_lex;
                                span = next_span;
                                break;
                            }
                        }
                    }
                    path.set_parents(parents);
                }
                _ => {}
            }

            // Cannot start with a path delimiter!
            match &lex {
                Parameters::PathDelimiter => {
                    *state.byte_mut() = span.start;
                    return Err(SyntaxError::UnexpectedPathDelimiter(
                        ErrorInfo::new(
                            source,
                            state.file_name(),
                            SourcePos::from((state.line(), state.byte())),
                        ),
                    ));
                }
                _ => {}
            }

            *state.byte_mut() = span.start;

            let component = Component(source, component_type(&lex), span);
            // Flag as a path that should be resolved from the root object
            if path.is_empty() && component.is_root() {
                path.set_root(true);
            }

            if component.is_explicit() {
                path.set_explicit(true);
            }

            if component.is_local() && path.parents() > 0 {
                return Err(SyntaxError::UnexpectedPathParentWithLocal(
                    ErrorInfo::new(
                        source,
                        state.file_name(),
                        SourcePos::from((state.line(), state.byte())),
                    ),
                ));
            }

            if component.is_explicit() && path.parents() > 0 {
                return Err(SyntaxError::UnexpectedPathParentWithExplicit(
                    ErrorInfo::new(
                        source,
                        state.file_name(),
                        SourcePos::from((state.line(), state.byte())),
                    ),
                ));
            }

            let mut wants_delimiter = !component.is_explicit_dot_slash();

            path.add_component(component);

            while let Some((lex, span)) = iter.next() {
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
                                println!("Lex {:?}", &lex);
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
                    //println!("Adding component for {:?}", &lex);
                    path.add_component(Component(
                        source,
                        component_type(&lex),
                        span,
                    ));
                } else {
                    next = Some((lex, span));
                    break;
                }
            }
            result = Some(path);
        }
    }
    Ok((result, next))
}
