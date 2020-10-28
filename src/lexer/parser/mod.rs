use std::ops::Range;

use logos::Span;

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{
        ast::{
            Block, BlockType, Node, Text,
        },
        grammar::{self, lex, Parameters, Token},
    },
};

/// Default file name.
static UNKNOWN: &str = "unknown";

mod arguments;
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

#[derive(Debug)]
pub struct Parser<'source> {
    options: ParserOptions,
    stack: Vec<Block<'source>>,
}

impl<'source> Parser<'source> {
    pub fn new(options: ParserOptions) -> Self {
        Self {
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

    fn newline(&self, t: &Token) -> bool {
        match t {
            Token::RawBlock(lex, _) => lex == &grammar::RawBlock::Newline,
            Token::RawComment(lex, _) => lex == &grammar::RawComment::Newline,
            Token::RawStatement(lex, _) => {
                lex == &grammar::RawStatement::Newline
            }
            Token::Comment(lex, _) => lex == &grammar::Comment::Newline,
            Token::Block(lex, _) => lex == &grammar::Block::Newline,
            Token::Parameters(lex, _) => lex == &grammar::Parameters::Newline,
            // NOTE: new lines are not allowed in string literals
            // NOTE: so we have special handling for this case
            Token::StringLiteral(lex, _) => false,
        }
    }

    pub fn parse(
        &mut self,
        source: &'source str,
    ) -> Result<Node<'source>, SyntaxError<'source>> {
        // Consecutive text to normalize
        let mut text: Option<Text> = None;

        let mut parameters: Option<ParameterCache> = None;

        let line = &mut self.options.line_offset.clone();
        let byte_offset = &mut self.options.byte_offset.clone();

        let mut state = ParseState::from(&self.options);

        self.enter_stack(Block::new(source, BlockType::Root, None), &mut text);

        for t in lex(source) {
            if t.is_text() {
                let txt = text.get_or_insert(Text(source, t.span().clone()));
                txt.1.end = t.span().end;
            } else {
                if let Some(txt) = text.take() {
                    let current = self.stack.last_mut().unwrap();
                    current.push(Node::Text(txt));
                }
            }

            if self.newline(&t) {
                *state.line_mut() +=  1;
                *line += 1;
                continue;
            }

            //println!("Parser {:?}", t);

            match t {
                Token::Block(lex, span) => match lex {
                    grammar::Block::StartRawBlock => {
                        self.enter_stack(
                            Block::new(source, BlockType::RawBlock, Some(span)),
                            &mut text,
                        );
                    }
                    grammar::Block::StartRawComment => {
                        self.enter_stack(
                            Block::new(source, BlockType::RawComment, Some(span)),
                            &mut text,
                        );
                    }
                    grammar::Block::StartRawStatement => {
                        self.enter_stack(
                            Block::new(source, BlockType::RawStatement, Some(span)),
                            &mut text,
                        );
                    }
                    grammar::Block::StartComment => {
                        self.enter_stack(
                            Block::new(source, BlockType::Comment, Some(span)),
                            &mut text,
                        );
                    }
                    grammar::Block::StartBlockScope => {
                        parameters = Some(ParameterCache::new(
                            ParameterContext::Block,
                            span.clone(),
                        ));

                        self.enter_stack(
                            Block::new(source, BlockType::Scoped, Some(span)),
                            &mut text,
                        );
                    }
                    grammar::Block::EndBlockScope => {
                        // TODO: check the closing element matches the
                        // TODO: name of the open scope block

                        self.exit_stack(span, &mut text);
                    }
                    grammar::Block::StartStatement => {
                        parameters = Some(ParameterCache::new(
                            ParameterContext::Statement,
                            span,
                        ));
                    }
                    _ => {}
                },
                Token::RawBlock(lex, span) => match lex {
                    grammar::RawBlock::End => {
                        self.exit_stack(span, &mut text);
                    }
                    _ => {}
                },
                Token::RawComment(lex, span) => match lex {
                    grammar::RawComment::End => {
                        self.exit_stack(span, &mut text);
                    }
                    _ => {}
                },
                Token::RawStatement(lex, span) => match lex {
                    grammar::RawStatement::End => {
                        self.exit_stack(span, &mut text);
                    }
                    _ => {}
                },
                Token::Comment(lex, span) => match lex {
                    grammar::Comment::End => {
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
                                source,
                                &mut state,
                                params.clone(),
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
                    grammar::StringLiteral::Newline => {
                        if let Some(params) = parameters.take() {
                            if let Some((lex, span)) = params.tokens.last() {
                                *byte_offset = span.end - 1;
                            }
                        }

                        return Err(SyntaxError::StringLiteralNewline(
                            ErrorInfo::new(
                                source,
                                self.options.file_name.as_str(),
                                SourcePos::from((line, byte_offset)),
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
                *byte_offset = span.end - 1;
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
                source,
                self.options.file_name.as_str(),
                SourcePos::from((line, byte_offset)),
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
