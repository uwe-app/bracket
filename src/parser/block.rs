use std::ops::Range;

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{self, Lexer, Parameters, Token},
    parser::{
        ast::{Block, BlockType, Node, Text},
        statement,
        ParameterCache, ParameterContext, ParseState,
    },
};

/// Parse a raw block `{{{{raw}}}}{{{{/raw}}}}`.
pub(crate) fn raw<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    span: Range<usize>,
) -> Option<Node<'source>> {
    let mut block = Block::new(source, BlockType::RawBlock, Some(span));
    while let Some(t) = lexer.next() {
        match t {
            Token::RawBlock(lex, span) => match lex {
                lexer::RawBlock::End => {
                    block.exit(span);
                    return Some(Node::Block(block));
                }
                _ => {
                    block.push(Node::Text(Text(source, span)));
                }
            },
            _ => {}
        }
    }
    None
}

/// Parse a raw comment `{{!-- comment --}}`.
pub(crate) fn raw_comment<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    span: Range<usize>,
) -> Option<Node<'source>> {
    let mut block = Block::new(source, BlockType::RawComment, Some(span));
    while let Some(t) = lexer.next() {
        match t {
            Token::RawComment(lex, span) => match lex {
                lexer::RawComment::End => {
                    block.exit(span);
                    return Some(Node::Block(block));
                }
                _ => {
                    block.push(Node::Text(Text(source, span)));
                }
            },
            _ => {}
        }
    }
    None
}

/// Parse an escaped statement `\{{escaped}}`.
pub(crate) fn escaped_statement<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    span: Range<usize>,
) -> Option<Node<'source>> {
    let mut block = Block::new(source, BlockType::RawStatement, Some(span));
    while let Some(t) = lexer.next() {
        match t {
            Token::RawStatement(lex, span) => match lex {
                lexer::RawStatement::End => {
                    block.exit(span);
                    return Some(Node::Block(block));
                }
                _ => {
                    block.push(Node::Text(Text(source, span)));
                }
            },
            _ => {}
        }
    }
    None
}

/// Parse a comment block `{{! comment }}`.
pub(crate) fn comment<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    span: Range<usize>,
) -> Option<Node<'source>> {
    let mut block = Block::new(source, BlockType::Comment, Some(span));
    while let Some(t) = lexer.next() {
        match t {
            Token::Comment(lex, span) => match lex {
                lexer::Comment::End => {
                    block.exit(span);
                    return Some(Node::Block(block));
                }
                _ => {
                    block.push(Node::Text(Text(source, span)));
                }
            },
            _ => {}
        }
    }
    None
}

/// Parse block or statement parameters.
pub(crate) fn parameters<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    span: Range<usize>,
    context: ParameterContext,
) -> Result<Option<ParameterCache>, SyntaxError<'source>> {
    let mut params = ParameterCache::new(context, span);
    while let Some(t) = lexer.next() {
        match t {
            Token::StringLiteral(lex, span) => match lex {
                lexer::StringLiteral::Newline => {
                    if let Some((lex, span)) = params.tokens.last() {
                        *state.byte_mut() = span.end - 1;
                    }
                    return Err(SyntaxError::StringLiteralNewline(
                        ErrorInfo::new(
                            source,
                            state.file_name(),
                            SourcePos::from((state.line(), state.byte())),
                        ),
                    ));
                }
                _ => {
                    params.tokens.push((Parameters::StringToken(lex), span));
                }
                _ => {}
            },
            Token::Parameters(lex, span) => match lex {
                lexer::Parameters::End => {
                    params.end = span;
                    return Ok(Some(params));
                }
                _ => {
                    params.tokens.push((lex, span));
                }
            },
            _ => {}
        }
    }
    Ok(None)
}


/// Parse a scoped block `{{# block}}`.
pub(crate) fn scope<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    span: Range<usize>,
) -> Result<Option<Node<'source>>, SyntaxError<'source>> {
    let mut parameters = parameters(
        source,
        lexer,
        state,
        span.clone(),
        ParameterContext::Block,
    )?;

    if let Some(params) = parameters.take() {

        println!("Parse a scoped block node...");

        let mut block = Block::new(
            source,
            BlockType::Scoped,
            Some(span),
        );

        match statement::parse(
            source,
            state,
            params.clone(),
        ) {
            Ok(call) => block.set_call(call),
            Err(e) => return Err(e),
        }

    } else {
        // FIXME: use SyntaxError
        panic!("Statement not terminated");
    }
    Ok(None)
}
