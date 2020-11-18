use std::ops::Range;

use crate::{
    lexer::{self, Lexer, Token},
    parser::{
        ast::{Block, Element, Node, Slice, Text, TextBlock, Lines},
        call::{self, CallParseContext},
        ParseState,
    },
    SyntaxResult,
};

/// Consume consecutive tokens into a single span.
pub(crate) fn until<'source>(
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    mut span: Range<usize>,
    end: &dyn Fn(&Token) -> bool,
) -> (Range<usize>, Option<Token>) {
    let mut next_token: Option<Token> = None;
    while let Some(t) = lexer.next() {
        if t.is_newline() {
            *state.line_mut() += 1;
        }
        if !end(&t) {
            span.end = t.span().end;
        } else {
            next_token = Some(t);
            break;
        }
    }
    return (span, next_token);
}

/// Parse text until a test indicates the end of the block.
pub(crate) fn text_until<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    span: Range<usize>,
    end: &dyn Fn(&Token) -> bool,
    wrap: &dyn Fn(TextBlock<'source>) -> Node<'source>,
) -> Option<(Node<'source>, Option<Token>)> {
    let text = span.end..span.end;
    let open = span;
    let line_range = state.line_range();
    let (span, next_token) = until(lexer, state, text, end);
    if let Some(ref close) = next_token {
        let mut text = Text::new(source, span, line_range);
        text.lines_end(state.line());
        let block = TextBlock::new(
            source,
            text,
            open,
            close.span().clone(),
        );
        return Some((wrap(block), next_token));
    }
    None
}

/// Parse a raw block `{{{{raw}}}}{{{{/raw}}}}`.
pub(crate) fn raw<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    span: Range<usize>,
) -> SyntaxResult<Node<'source>> {
    let mut block = Block::new(source, span.clone(), true, state.line_range());

    let call =
        call::parse(source, lexer, state, span.clone(), CallParseContext::Raw)?;

    if !call.is_closed() {
        panic!("Raw block start tag was not terminated");
    }

    let start_name = call.target().as_str();

    // NOTE: must have an accurate end span before reading the Text chunk!
    let end_span = call.close_span().clone().unwrap();

    block.set_call(call);

    let end = |t: &Token| match t {
        Token::Block(lex, _) => match lex {
            lexer::Block::EndRawBlock => true,
            _ => false,
        },
        _ => false,
    };

    let wrap = |t: TextBlock<'source>| Node::Text(t.into());

    let maybe_node = text_until(source, lexer, state, end_span, &end, &wrap);
    if let Some((node, next_token)) = maybe_node {
        //let string = &node.source()[span.clone()];

        //println!("Got end raw block slice {:?}", string);

        let span = if let Some(token) = next_token {
            match token {
                Token::Block(lex, span) => match lex {
                    lexer::Block::EndRawBlock => span,
                    _ => panic!("Expecting end raw block!"),
                },
                _ => panic!("Expecting block token in end raw block!"),
            }
        } else {
            panic!("Unable to get end raw block span")
        };

        block.push(node);

        let end_tag =
            call::parse(source, lexer, state, span, CallParseContext::Raw)?;

        if let Some(close_span) = end_tag.close_span() {
            let exit_span = end_tag.open_span().start..close_span.end;
            block.exit(exit_span);
        } else {
            panic!("Raw block was not terminated correctly!");
        }

        let end_name = end_tag.target().as_str();

        if start_name != end_name {
            // FIXME: return an error here!
            panic!(
                "Raw block start '{}' does not match end name '{}'",
                start_name, end_name
            );
        }

        block.lines_end(state.line());

        Ok(Node::Block(block))
    } else {
        // FIXME:
        panic!("Raw block was not terminated");
    }
}

/// Parse a raw comment `{{!-- comment --}}`.
pub(crate) fn raw_comment<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    span: Range<usize>,
) -> SyntaxResult<Node<'source>> {
    let end = |t: &Token| match t {
        Token::RawComment(lex, _) => match lex {
            lexer::RawComment::End => true,
            _ => false,
        },
        _ => false,
    };

    let wrap = |t: TextBlock<'source>| Node::RawComment(t);
    let maybe_node = text_until(source, lexer, state, span, &end, &wrap);
    if let Some((node, _)) = maybe_node {
        return Ok(node);
    } else {
        // FIXME:
        panic!("Raw comment was not terminated");
    }
}

/// Parse an escaped statement `\{{escaped}}`.
pub(crate) fn raw_statement<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    span: Range<usize>,
) -> SyntaxResult<Node<'source>> {
    let end = |t: &Token| match t {
        Token::RawStatement(lex, _) => match lex {
            lexer::RawStatement::End => true,
            _ => false,
        },
        _ => false,
    };

    let wrap = |t: TextBlock<'source>| Node::RawStatement(t);
    let maybe_node = text_until(source, lexer, state, span, &end, &wrap);
    if let Some((node, _)) = maybe_node {
        return Ok(node);
    } else {
        // FIXME:
        panic!("Raw statement was not terminated");
    }
}

/// Parse a comment block `{{! comment }}`.
pub(crate) fn comment<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    span: Range<usize>,
) -> SyntaxResult<Node<'source>> {
    let end = |t: &Token| match t {
        Token::Comment(lex, _) => match lex {
            lexer::Comment::End => true,
            _ => false,
        },
        _ => false,
    };

    let wrap = |t: TextBlock<'source>| Node::Comment(t);
    let maybe_node = text_until(source, lexer, state, span, &end, &wrap);
    if let Some((node, _)) = maybe_node {
        return Ok(node);
    } else {
        // FIXME:
        panic!("Comment was not terminated");
    }
}

/// Open a scoped block `{{# block}}`.
pub(crate) fn scope<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    span: Range<usize>,
) -> SyntaxResult<Block<'source>> {
    let mut block = Block::new(source, span.clone(), false, state.line_range());
    let call =
        call::parse(source, lexer, state, span, CallParseContext::Block)?;
    block.set_call(call);
    Ok(block)
}
