use std::vec::IntoIter;

use logos::Span;

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{
        ast::{
            Component, ComponentType, Path,
        },
        grammar::Parameters,
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
    file_name: &str,
    iter: &mut IntoIter<(Parameters, Span)>,
    byte_offset: &mut usize,
    line: &mut usize,
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
            *byte_offset = span.end;

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
                    *byte_offset = span.start;
                    return Err(SyntaxError::UnexpectedPathDelimiter(
                        ErrorInfo::new(
                            source,
                            file_name,
                            SourcePos::from((line, byte_offset)),
                        ),
                    ));
                }
                _ => {}
            }

            *byte_offset = span.start;

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
                return Err(SyntaxError::UnexpectedPathParentWithLocal(
                    ErrorInfo::new(
                        source,
                        file_name,
                        SourcePos::from((line, byte_offset)),
                    ),
                ));
            }

            if component.is_explicit() && path.parents() > 0 {
                return Err(SyntaxError::UnexpectedPathParentWithExplicit(
                    ErrorInfo::new(
                        source,
                        file_name,
                        SourcePos::from((line, byte_offset)),
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
                            *byte_offset = span.start;
                            return Err(
                                SyntaxError::UnexpectedPathExplicitThis(
                                    ErrorInfo::new(
                                        source,
                                        file_name,
                                        SourcePos::from((
                                            line,
                                            byte_offset,
                                        )),
                                    ),
                                ),
                            );
                        }
                        Parameters::ParentRef => {
                            *byte_offset = span.start;
                            return Err(SyntaxError::UnexpectedPathParent(
                                ErrorInfo::new(
                                    source,
                                    file_name,
                                    SourcePos::from((line, byte_offset)),
                                ),
                            ));
                        }
                        Parameters::LocalIdentifier => {
                            *byte_offset = span.start;
                            return Err(SyntaxError::UnexpectedPathLocal(
                                ErrorInfo::new(
                                    source,
                                    file_name,
                                    SourcePos::from((line, byte_offset)),
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
                                *byte_offset = span.start;
                                println!("Lex {:?}", &lex);
                                return Err(
                                    SyntaxError::ExpectedPathDelimiter(
                                        ErrorInfo::new(
                                            source,
                                            file_name,
                                            SourcePos::from((
                                                line,
                                                byte_offset,
                                            )),
                                        ),
                                    ),
                                );
                            }
                        }
                    } else {
                        match &lex {
                            Parameters::PathDelimiter => {
                                *byte_offset = span.start;
                                return Err(
                                    SyntaxError::UnexpectedPathDelimiter(
                                        ErrorInfo::new(
                                            source,
                                            file_name,
                                            SourcePos::from((
                                                line,
                                                byte_offset,
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

