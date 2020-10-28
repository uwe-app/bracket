use std::ops::Range;

use logos::Span;

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{self, lex, Lexer, Parameters, Token},
    parser::ast::{Block, BlockType, Node, Text},
};

/// Default file name.
static UNKNOWN: &str = "unknown";

mod arguments;
pub mod ast;
mod block;
mod json_literal;
mod path;
mod statement;
mod whitespace;

#[derive(Debug)]
pub struct ParserOptions {
    /// The name of a file for the template source being parsed.
    pub file_name: String,
    /// A line offset into the file for error reporting,
    /// the first line has index zero.
    pub line_offset: usize,
    /// Byte offset into the source file.
    pub byte_offset: usize,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            file_name: UNKNOWN.to_string(),
            line_offset: 0,
            byte_offset: 0,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ParseState {
    file_name: String,
    line: usize,
    byte: usize,
}

impl ParseState {
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn line(&self) -> &usize {
        &self.line
    }

    pub fn line_mut(&mut self) -> &mut usize {
        &mut self.line
    }

    pub fn byte(&self) -> &usize {
        &self.byte
    }

    pub fn byte_mut(&mut self) -> &mut usize {
        &mut self.byte
    }
}

impl From<&ParserOptions> for ParseState {
    fn from(opts: &ParserOptions) -> Self {
        Self {
            file_name: opts.file_name.clone(),
            line: opts.line_offset.clone(),
            byte: opts.byte_offset.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ParameterContext {
    Block,
    Statement,
}

#[derive(Debug, Clone)]
pub(crate) struct ParameterCache {
    context: ParameterContext,
    tokens: Vec<(Parameters, Span)>,
    start: Span,
    end: Span,
}

impl ParameterCache {
    pub fn new(context: ParameterContext, start: Span) -> Self {
        Self {
            context,
            start,
            tokens: Default::default(),
            end: Default::default(),
        }
    }
}

pub struct Parser<'source> {
    source: &'source str,
    lexer: Lexer<'source>,
    state: ParseState,
    options: ParserOptions,
    stack: Vec<Block<'source>>,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str, options: ParserOptions) -> Self {
        let lexer = lex(source);
        let state = ParseState::from(&options);
        Self {
            source,
            lexer,
            state,
            options,
            stack: vec![],
        }
    }

    fn enter_stack(
        &mut self,
        block: Block<'source>,
        text: &mut Option<Text<'source>>,
    ) {
        // Must consume the text now!
        if let Some(txt) = text.take() {
            if let Some(current) = self.stack.last_mut() {
                current.push(Node::Text(txt));
            }
        }
        self.stack.push(block);
    }

    fn exit_stack(
        &mut self,
        close: Range<usize>,
        text: &mut Option<Text<'source>>,
    ) {
        let current = self.stack.last_mut().unwrap();

        // Must consume the text now!
        if let Some(txt) = text.take() {
            current.push(Node::Text(txt));
        }

        current.exit(close);
        let mut last = self.stack.pop();
        if let Some(block) = last.take() {
            // Add the current block to the tree
            let current = self.stack.last_mut().unwrap();
            current.push(Node::Block(block));
        }
    }

    pub fn parse(&mut self) -> Result<Node<'source>, SyntaxError<'source>> {
        //let source = self.source;

        // Consecutive text to normalize
        let mut text: Option<Text> = None;

        let mut parameters: Option<ParameterCache> = None;

        self.enter_stack(
            Block::new(self.source, BlockType::Root, None),
            &mut text,
        );

        while let Some(t) = self.lexer.next() {
            if t.is_text() {
                let txt =
                    text.get_or_insert(Text(self.source, t.span().clone()));
                txt.1.end = t.span().end;
            } else {
                if let Some(txt) = text.take() {
                    let current = self.stack.last_mut().unwrap();
                    current.push(Node::Text(txt));
                }
            }

            if t.is_newline() {
                *self.state.line_mut() += 1;
                continue;
            }

            //println!("Parser {:?}", t);

            match t {
                Token::Block(lex, span) => match lex {
                    lexer::Block::StartRawBlock => {
                        self.enter_stack(
                            Block::new(
                                self.source,
                                BlockType::RawBlock,
                                Some(span),
                            ),
                            &mut text,
                        );
                    }
                    lexer::Block::StartRawComment => {
                        self.enter_stack(
                            Block::new(
                                self.source,
                                BlockType::RawComment,
                                Some(span),
                            ),
                            &mut text,
                        );
                    }
                    lexer::Block::StartRawStatement => {
                        self.enter_stack(
                            Block::new(
                                self.source,
                                BlockType::RawStatement,
                                Some(span),
                            ),
                            &mut text,
                        );
                    }
                    lexer::Block::StartComment => {
                        self.enter_stack(
                            Block::new(
                                self.source,
                                BlockType::Comment,
                                Some(span),
                            ),
                            &mut text,
                        );
                    }
                    lexer::Block::StartBlockScope => {
                        parameters = Some(ParameterCache::new(
                            ParameterContext::Block,
                            span.clone(),
                        ));

                        self.enter_stack(
                            Block::new(
                                self.source,
                                BlockType::Scoped,
                                Some(span),
                            ),
                            &mut text,
                        );
                    }
                    lexer::Block::EndBlockScope => {
                        // TODO: check the closing element matches the
                        // TODO: name of the open scope block

                        self.exit_stack(span, &mut text);
                    }
                    lexer::Block::StartStatement => {
                        parameters = Some(ParameterCache::new(
                            ParameterContext::Statement,
                            span,
                        ));
                    }
                    _ => {}
                },
                Token::RawBlock(lex, span) => match lex {
                    lexer::RawBlock::End => {
                        self.exit_stack(span, &mut text);
                    }
                    _ => {}
                },
                Token::RawComment(lex, span) => match lex {
                    lexer::RawComment::End => {
                        self.exit_stack(span, &mut text);
                    }
                    _ => {}
                },
                Token::RawStatement(lex, span) => match lex {
                    lexer::RawStatement::End => {
                        self.exit_stack(span, &mut text);
                    }
                    _ => {}
                },
                Token::Comment(lex, span) => match lex {
                    lexer::Comment::End => {
                        self.exit_stack(span, &mut text);
                    }
                    _ => {}
                },
                Token::Parameters(lex, span) => match lex {
                    Parameters::End => {
                        if let Some(mut params) = parameters.take() {
                            let ctx = params.context.clone();
                            params.end = span;

                            let call = statement::parse(
                                self.source,
                                &mut self.state,
                                params,
                            )?;

                            let current = self.stack.last_mut().unwrap();
                            match ctx {
                                ParameterContext::Statement => {
                                    current.push(Node::Statement(call));
                                }
                                ParameterContext::Block => {
                                    current.set_call(call);
                                }
                            }
                        }
                    }
                    _ => {
                        if let Some(params) = parameters.as_mut() {
                            params.tokens.push((lex, span));
                        }
                    }
                },
                Token::StringLiteral(lex, span) => match lex {
                    lexer::StringLiteral::Newline => {
                        if let Some(params) = parameters.take() {
                            if let Some((lex, span)) = params.tokens.last() {
                                *self.state.byte_mut() = span.end - 1;
                            }
                        }

                        return Err(SyntaxError::StringLiteralNewline(
                            ErrorInfo::new(
                                self.source,
                                self.state.file_name(),
                                SourcePos::from((
                                    self.state.line(),
                                    self.state.byte(),
                                )),
                            ),
                        ));
                    }
                    _ => {
                        if let Some(params) = parameters.as_mut() {
                            params
                                .tokens
                                .push((Parameters::StringToken(lex), span));
                        }
                    }
                },
            }
        }

        if let Some(mut params) = parameters.take() {
            if !params.tokens.is_empty() {
                let (lex, span) = params.tokens.pop().unwrap();
                *self.state.byte_mut() = span.end - 1;
            }

            let str_literal = params
                .tokens
                .iter()
                .find(|(t, _)| &Parameters::StringLiteral == t);

            let mut notes: Vec<&'static str> = Vec::new();
            if str_literal.is_some() {
                notes.push("string literal was not closed");
            }

            return Err(SyntaxError::OpenStatement(ErrorInfo::new_notes(
                self.source,
                self.state.file_name(),
                SourcePos::from((self.state.line(), self.state.byte())),
                notes,
            )));
        }

        // Must append any remaining normalized text!
        if let Some(txt) = text.take() {
            let current = self.stack.last_mut().unwrap();
            current.push(Node::Text(txt));
        }

        Ok(Node::Block(self.stack.swap_remove(0)))
    }
}

impl<'source> Iterator for Parser<'source> {
    type Item = Result<Node<'source>, SyntaxError<'source>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t) = self.lexer.next() {
            if t.is_text() {
                return Some(Ok(Node::Text(Text(
                    self.source,
                    t.span().clone(),
                ))));
            }

            if t.is_newline() {
                *self.state.line_mut() += 1;
                return Some(Ok(Node::Text(Text(
                    self.source,
                    t.span().clone(),
                ))));
            }

            match t {
                Token::Block(lex, span) => match lex {
                    lexer::Block::StartRawBlock => {
                        return block::raw(self.source, &mut self.lexer, span)
                            .map(Ok);
                    }
                    lexer::Block::StartRawComment => {
                        return block::raw_comment(
                            self.source,
                            &mut self.lexer,
                            span,
                        )
                        .map(Ok);
                    }
                    lexer::Block::StartRawStatement => {
                        return block::escaped_statement(
                            self.source,
                            &mut self.lexer,
                            span,
                        )
                        .map(Ok);
                    }
                    lexer::Block::StartComment => {
                        return block::comment(
                            self.source,
                            &mut self.lexer,
                            span,
                        )
                        .map(Ok);
                    }
                    lexer::Block::StartBlockScope => {
                        return block::scope(
                            self.source,
                            &mut self.lexer,
                            &mut self.state,
                            span,
                        ).ok().map(|o| o.ok_or_else(|| {
                            todo!("Handle no block node!")   
                        }));
                    }
                    lexer::Block::EndBlockScope => {
                        // TODO: check the closing element matches the
                        // TODO: name of the open scope block
                    }
                    lexer::Block::StartStatement => {
                        match block::parameters(
                            self.source,
                            &mut self.lexer,
                            &mut self.state,
                            span,
                            ParameterContext::Statement,
                        ) {
                            Ok(mut parameters) => {

                                if let Some(params) = parameters.take() {
                                    match statement::parse(
                                        self.source,
                                        &mut self.state,
                                        params,
                                    ) {
                                        Ok(call) => {
                                            return Some(Ok(
                                                Node::Statement(call),
                                            ))
                                        }
                                        Err(e) => return Some(Err(e)),
                                    } 
                                } else {
                                    // FIXME: use SyntaxError
                                    panic!("Statement not terminated");
                                }
                            }
                            Err(e) => return Some(Err(e))
                        }
                    }
                    _ => {}
                },
                _ => panic!("Unexpected token {:?}", t),
            }
        }
        None
    }
}
