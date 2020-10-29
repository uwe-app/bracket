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

/// Consume consecutive tokens into a single span.
pub(crate) fn until<'source>(
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    mut span: Range<usize>,
    end: &Fn(&Token) -> bool,
) -> (Range<usize>, Option<Token>) {
    let mut next_token: Option<Token> = None;
    while let Some(t) = lexer.next() {
        if t.is_newline() { *state.line_mut() += 1; }
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
    block_type: BlockType,
    end: &Fn(&Token) -> bool,
) -> Option<Node<'source>> {
    let text = span.end..span.end;
    let mut block = Block::new(source, block_type, Some(span));
    let (span, next_token) = until(lexer, state, text, end);
    block.push(Node::Text(Text(source, span)));
    if let Some(close) = next_token {
        block.exit(close.span().clone());
    }
    return Some(Node::Block(block));
}

/// Parse a raw block `{{{{raw}}}}{{{{/raw}}}}`.
pub(crate) fn raw<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    span: Range<usize>,
) -> Option<Node<'source>> {
    let end = |t: &Token| {
        match t {
            Token::RawBlock(lex, span) => match lex {
                lexer::RawBlock::End => true,
                _ => false
            },
            _ => false
        }
    };
    text_until(source, lexer, state, span, BlockType::RawBlock, &end)
}

/// Parse a raw comment `{{!-- comment --}}`.
pub(crate) fn raw_comment<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    span: Range<usize>,
) -> Option<Node<'source>> {
    let end = |t: &Token| {
        match t {
            Token::RawComment(lex, span) => match lex {
                lexer::RawComment::End => true,
                _ => false
            },
            _ => false
        }
    };
    text_until(source, lexer, state, span, BlockType::RawComment, &end)
}

/// Parse an escaped statement `\{{escaped}}`.
pub(crate) fn escaped_statement<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    span: Range<usize>,
) -> Option<Node<'source>> {
    let end = |t: &Token| {
        match t {
            Token::RawStatement(lex, span) => match lex {
                lexer::RawStatement::End => true,
                _ => false
            },
            _ => false
        }
    };
    text_until(source, lexer, state, span, BlockType::RawStatement, &end)
}

/// Parse a comment block `{{! comment }}`.
pub(crate) fn comment<'source>(
    source: &'source str,
    lexer: &mut Lexer<'source>,
    state: &mut ParseState,
    span: Range<usize>,
) -> Option<Node<'source>> {
    let end = |t: &Token| {
        match t {
            Token::Comment(lex, span) => match lex {
                lexer::Comment::End => true,
                _ => false
            },
            _ => false
        }
    };
    text_until(source, lexer, state, span, BlockType::Comment, &end)
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
        if t.is_newline() { *state.line_mut() += 1; }
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

        println!("Start tag is {:?}", block.call());

        while let Some(t) = lexer.next() {
            println!("Got block token {:?}", t);
            match t {
                Token::Block(lex, span) => match lex {
                    lexer::Block::EndBlockScope => {
                        println!("Got END block token... {:?}", lex);
                    }
                    _ => {}
                }
                _ => {}
            }
        }

    } else {
        // FIXME: use SyntaxError
        panic!("Statement not terminated");
    }
    Ok(None)
}
