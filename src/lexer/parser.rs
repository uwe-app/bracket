use std::fmt;
use std::ops::Range;

use logos::Span;

use crate::{
    error::{SyntaxError, ErrorInfo, SourcePos},
    lexer::{
        ast::{Block, BlockType, Node, Text, Call, Path},
        grammar::{self, lex, Parameters, Token},
    },
};

static UNKNOWN: &str = "unknown";

#[derive(Debug)]
pub struct ParserOptions {
    /// The name of a file for the template source being parsed.
    pub file_name: String,
    /// A line offset into the file for error reporting, 
    /// the first line has index zero.
    pub line_offset: usize,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {file_name: UNKNOWN.to_string(), line_offset: 0} 
    } 
}

#[derive(Clone, Debug)]
enum ParameterContext {
    Block,
    Statement,
}

#[derive(Debug)]
struct ParameterCache {
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
        Self { options, stack: vec![] }
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

    fn parse_parameters(
        &mut self,
        s: &'source str,
        line: &usize,
        statement: &mut ParameterCache,
    ) -> Result<Call<'source>, SyntaxError<'source>> {
        // Position as byte offset for syntax errors
        let mut byte_offset = statement.start.end;

        if !statement.tokens.is_empty() {
            let mut iter = statement.tokens.iter();
            let (first, span) = iter.next().unwrap();
            let mut identifier: Option<(&Parameters, &Span)> = None;

            // Find the next token that exists in a list of expected tokens
            // at the next position consuming preceeding whitespace.
            let mut find_until =
                |expects: Vec<Parameters>| -> Option<&(Parameters, Span)> {
                    while let Some(item) = iter.next() {
                        if item.0 == Parameters::WhiteSpace {
                            continue;
                        } else if expects.contains(&item.0) {
                            return Some(item);
                        }
                        break;
                    }
                    None
                };

            match first {
                Parameters::Identifier | Parameters::LocalIdentifier => {
                    identifier = Some((first, span));
                }
                Parameters::Partial => {
                    byte_offset = span.end;
                    if let Some((lex, span)) =
                        find_until(vec![Parameters::Identifier, Parameters::LocalIdentifier])
                    {
                        identifier = Some((lex, span));
                    }
                }
                _ => {}
            }

            println!("Parse statement with identifier {:?}", identifier);

            if identifier.is_none() {
                let pos = SourcePos(line.clone(), byte_offset);
                let info = ErrorInfo::from((s, &self.options, pos));
                return Err(SyntaxError::ExpectedIdentifier(info));
            }

            let range = statement.start.end..statement.end.start;
            let call = Call::new(s, range, Path(vec![]), None, None);
            Ok(call)
        } else {
            let pos = SourcePos(line.clone(), byte_offset);
            let info = ErrorInfo::from((s, &self.options, pos));
            Err(SyntaxError::EmptyStatement(info))
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
            Token::BlockScope(lex, _) => lex == &grammar::BlockScope::Newline,
            Token::Parameters(lex, _) => lex == &grammar::Parameters::Newline,
        }
    }

    pub fn parse(
        &mut self,
        s: &'source str,
    ) -> Result<Node<'source>, SyntaxError<'source>> {

        // Consecutive text to normalize
        let mut text: Option<Text> = None;

        let mut parameters: Option<ParameterCache> = None;
        let mut line: usize = self.options.line_offset.clone();

        self.enter_stack(Block::new(s, BlockType::Root, None), &mut text);

        for t in lex(s) {
            if t.is_text() {
                let txt = text.get_or_insert(Text(s, t.span().clone()));
                txt.1.end = t.span().end;
            } else {
                if let Some(txt) = text.take() {
                    let current = self.stack.last_mut().unwrap();
                    current.push(Node::Text(txt));
                }
            }

            if self.newline(&t) {
                line += 1;
                continue;
            }

            //println!("Parser {:?}", t);
            
            match t {
                Token::Block(lex, span) => match lex {
                    grammar::Block::StartRawBlock => {
                        self.enter_stack(
                            Block::new(
                                s,
                                BlockType::RawBlock,
                                Some(span),
                            ),
                            &mut text,
                        );
                    }
                    grammar::Block::StartRawComment => {
                        self.enter_stack(
                            Block::new(
                                s,
                                BlockType::RawComment,
                                Some(span),
                            ),
                            &mut text,
                        );
                    }
                    grammar::Block::StartRawStatement => {
                        self.enter_stack(
                            Block::new(
                                s,
                                BlockType::RawStatement,
                                Some(span),
                            ),
                            &mut text,
                        );
                    }
                    grammar::Block::StartComment => {
                        self.enter_stack(
                            Block::new(
                                s,
                                BlockType::Comment,
                                Some(span),
                            ),
                            &mut text,
                        );
                    }
                    grammar::Block::StartBlockScope => {
                        self.enter_stack(
                            Block::new(
                                s,
                                BlockType::Scoped,
                                Some(span),
                            ),
                            &mut text,
                        );
                    }
                    grammar::Block::EndBlockScope => {

                        // TODO: check the closing element matches the
                        // TODO: name of the open scope block

                        self.exit_stack(span, &mut text);
                    }
                    grammar::Block::StartStatement => {
                        parameters = Some(
                            ParameterCache::new(
                                ParameterContext::Statement, span));
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

                            let call = self.parse_parameters(s, &line, &mut params)?;
                            match ctx {
                                ParameterContext::Statement => {
                                    let current = self.stack.last_mut().unwrap();
                                    current.push(Node::Statement(call));
                                }
                                ParameterContext::Block => {
                                    let current = self.stack.last_mut().unwrap();
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
                Token::BlockScope(lex, span) => match lex {
                    _ => {}
                },
            }
        }

        // Must append any remaining normalized text!
        if let Some(txt) = text.take() {
            let current = self.stack.last_mut().unwrap();
            current.push(Node::Text(txt));
        }

        Ok(Node::Block(self.stack.swap_remove(0)))
    }
}
