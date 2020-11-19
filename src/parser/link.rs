use std::ops::Range;
use serde_json::{Number, Value};

use crate::{
    error::{ErrorInfo, SyntaxError},
    lexer::{self, Lexer, Parameters, Token},
    parser::{
        ast::{Link, Lines, Element},
        path, string, ParseState,
    },
    SyntaxResult,
};

#[derive(Debug, Default)]
struct EscapeFlags {
    pipe: bool,
    bracket: bool,
}

impl EscapeFlags {
    fn has_escape_sequences(&self) -> bool {
        self.pipe || self.bracket
    }

    fn into_owned<'a>(&self, value: &'a str) -> String {
        let mut val = value.to_string();
        if self.pipe {
            val = val.replace("\\|", "|");
        }
        if self.bracket {
            val = val.replace("\\]", "]");
        }
        val
    }
}

fn label<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    link: &mut Link<'source>,
) -> SyntaxResult<()> {

    let mut flags: EscapeFlags = Default::default();

    while let Some(token) = lexer.next() {
        match token {
            Token::Link(lex, span) => {
                match lex {
                    lexer::Link::Newline => {
                        *state.line_mut() += 1;
                    }
                    lexer::Link::Text => {
                        link.label_end(span.end);
                    }
                    lexer::Link::Pipe => {
                        // NOTE: for now subsequent pipes just become 
                        // NOTE: part of the label, later we may want to support
                        // NOTE: a `title` using another pipe.
                        link.label_end(span.end);
                    }
                    lexer::Link::EscapedPipe => {
                        flags.pipe = true;
                    }
                    lexer::Link::Escaped => {
                        flags.bracket = true;
                    }
                    lexer::Link::End => {
                        link.exit(span);
                        return Ok(());
                    }
                    _ => panic!("Unexpected link token"),
                } 
            }
            _ => panic!("Expecting a link token"),
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
                        return label(source, lexer, state, link);
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
                    _ => panic!("Unexpected link token"),
                } 
            }
            _ => panic!("Expecting a link token"),
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
        panic!("Link was not closed...");
    }

    link.lines_end(state.line());

    Ok(link)
}

