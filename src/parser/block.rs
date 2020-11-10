use std::ops::Range;

use crate::{
    lexer::{self, Lexer, Token},
    parser::{
        ast::{Block, Node, Text, TextBlock},
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
) -> Option<Node<'source>> {
    let text = span.end..span.end;
    let open = span;
    let (span, next_token) = until(lexer, state, text, end);
    if let Some(close) = next_token {
        let block = TextBlock::new(
            source,
            Text(source, span),
            open,
            close.span().clone(),
        );
        return Some(wrap(block));
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
    let end = |t: &Token| match t {
        Token::RawBlock(lex, _) => match lex {
            lexer::RawBlock::End => true,
            _ => false,
        },
        _ => false,
    };

    let wrap = |t: TextBlock<'source>| Node::RawBlock(t);
    let maybe_node = text_until(source, lexer, state, span, &end, &wrap);
    if let Some(node) = maybe_node {
        return Ok(node);
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
    if let Some(node) = maybe_node {
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
    if let Some(node) = maybe_node {
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
    if let Some(node) = maybe_node {
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
    let mut block = Block::new(source, span.clone());
    let call =
        call::parse(source, lexer, state, span, CallParseContext::Block)?;
    block.set_call(call);
    Ok(block)
}
