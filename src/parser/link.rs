use std::ops::Range;

use crate::{
    error::{ErrorInfo, SyntaxError},
    lexer::{self, Lexer, Token},
    parser::{
        ast::{Link, Lines, Element},
        ParseState,
    },
    SyntaxResult,
};

#[derive(Debug, Default)]
struct EscapeFlags {
    newline: bool,
    pipe: bool,
    bracket: bool,
}

impl EscapeFlags {
    fn has_escape_sequences(&self) -> bool {
        self.newline || self.pipe || self.bracket
    }

    fn into_owned<'a>(&self, value: &'a str) -> String {
        let mut val = value.to_string();
        if self.newline {
            val = val.replace("\\n", "\n");
        }
        if self.pipe {
            val = val.replace("\\|", "|");
        }
        if self.bracket {
            val = val.replace("\\]", "]");
        }
        val
    }
}

enum ValueType {
    Label,
    Title,
}

/// Assign an owned value to the link if escape sequences
/// were detected.
fn assign_if_escaped<'source>(
    source: &'source str,
    link: &mut Link<'source>,
    flags: &EscapeFlags,
    span: &Range<usize>,
    value_type: &ValueType) {

    if flags.has_escape_sequences() {
        match value_type {
            ValueType::Label => {
                let value = flags
                    .into_owned(
                        &source[link.label_span().start..span.start]);
                link.set_label(value);
            }
            ValueType::Title => {
                let value = flags
                    .into_owned(
                        &source[link.title_span().start..span.start]);
                link.set_title(value);
            }
        }
    }
}

/// Parse the label and title components.
fn value<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    link: &mut Link<'source>,
    value_type: ValueType,
) -> SyntaxResult<()> {

    let mut flags: EscapeFlags = Default::default();

    while let Some(token) = lexer.next() {
        match token {
            Token::Link(lex, span) => {

                match &lex {
                    _ => *state.byte_mut() = span.end - 1,
                }

                match lex {
                    lexer::Link::Newline => {
                        *state.line_mut() += 1;
                    }
                    lexer::Link::Text => {
                        match value_type {
                            ValueType::Label => {
                                link.label_end(span.end);
                            }
                            ValueType::Title => {
                                link.title_end(span.end);
                            }
                        }
                    }
                    lexer::Link::Pipe => {
                        match value_type {
                            ValueType::Label => {
                                link.label_end(span.start);
                                assign_if_escaped(source, link, &flags, &span, &value_type);

                                link.title_start(span.end);
                                return value(source, lexer, state, link, ValueType::Title)
                            }
                            ValueType::Title => {
                                // Consume any additional pipes until we get till the end.
                                // TODO: make this an error?
                                link.title_end(span.end);
                            }
                        }
                    }
                    lexer::Link::EscapedNewline => {
                        flags.newline = true;
                    }
                    lexer::Link::EscapedPipe => {
                        flags.pipe = true;
                    }
                    lexer::Link::Escaped => {
                        flags.bracket = true;
                    }
                    lexer::Link::End => {
                        assign_if_escaped(source, link, &flags, &span, &value_type);
                        link.exit(span);
                        return Ok(());
                    }
                    _ => return Err(
                        SyntaxError::TokenLink(ErrorInfo::from((source, state)).into()))
                } 
            }
            _ => return Err(
                SyntaxError::TokenLink(ErrorInfo::from((source, state)).into()))
        }
    }

    Ok(())
}

fn href<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    link: &mut Link<'source>,
) -> SyntaxResult<()> {

    let mut flags: EscapeFlags = Default::default();

    while let Some(token) = lexer.next() {
        match token {
            Token::Link(lex, span) => {

                match &lex {
                    _ => *state.byte_mut() = span.end - 1,
                }

                match lex {
                    lexer::Link::Newline => {
                        *state.line_mut() += 1;
                        // NOTE: newlines are not allowed in links
                        // NOTE: by returning early the link will 
                        // NOTE: never be closed and we should generate
                        // NOTE: an unclosed link error.
                        return Ok(())
                    }
                    lexer::Link::Text => {
                        link.href_end(span.end);
                    }
                    lexer::Link::Pipe => {
                        if flags.has_escape_sequences() {
                            let value = flags
                                .into_owned(&source[link.open_span().end..span.start]);
                            link.set_href(value);
                        }
                        link.label_start(span.end);
                        return value(source, lexer, state, link, ValueType::Label);
                    }
                    lexer::Link::EscapedNewline => {
                        flags.newline = true;
                    }
                    lexer::Link::EscapedPipe => {
                        flags.pipe = true;
                    }
                    lexer::Link::Escaped => {
                        flags.bracket = true;
                    }
                    lexer::Link::End => {
                        if flags.has_escape_sequences() {
                            let value = flags
                                .into_owned(&source[link.open_span().end..span.start]);
                            link.set_href(value);
                        }
                        link.exit(span);
                        return Ok(());
                    }
                    _ => return Err(
                        SyntaxError::TokenLink(ErrorInfo::from((source, state)).into()))
                } 
            }
            _ => return Err(
                SyntaxError::TokenLink(ErrorInfo::from((source, state)).into()))
        }
    }

    Ok(())
}

pub(crate) fn parse<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    open: Range<usize>,
) -> SyntaxResult<Link<'source>> {
    *state.byte_mut() = open.end;

    let mut link = Link::new(source, open, state.line_range());
    href(source, lexer, state, &mut link)?;

    if !link.is_closed() {
        Err(SyntaxError::LinkNotTerminated(
            ErrorInfo::from((source, state)).into()))
    } else {
        link.lines_end(state.line());
        Ok(link)
    }
}

