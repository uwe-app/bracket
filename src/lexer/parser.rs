use std::ops::Range;
use std::vec::IntoIter;

use serde_json::{Value, Number};

use logos::Span;

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{
        ast::{Block, BlockType, Call, Node, Path, Text, ParameterValue},
        grammar::{self, lex, Parameters, Token},
    },
};

/// Default file name.
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
        Self {
            file_name: UNKNOWN.to_string(),
            line_offset: 0,
        }
    }
}

#[derive(Clone, Debug)]
enum ParameterContext {
    Block,
    Statement,
}


#[derive(Debug, Clone)]
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
        Self {
            options,
            stack: vec![],
        }
    }

    /// Helper to generate error information with source position.
    fn err_info(
        &self,
        source: &'source str,
        line: &mut usize,
        byte_offset: usize) -> ErrorInfo<'source> {
        let pos = SourcePos(line.clone(), byte_offset.clone());
        ErrorInfo::from((source, &self.options, pos))
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

    /// Consume whitespace tokens.
    fn consume_whitespace(
        &self,
        iter: &mut IntoIter<(Parameters, Span)>,
        byte_offset: &mut usize,
        line: &mut usize,
    ) -> Option<(Parameters, Span)> {
        while let Some(item) = iter.next() {
            if item.0 == Parameters::WhiteSpace || item.0 == Parameters::Newline
            {
                *byte_offset = item.1.end;
                if item.0 == Parameters::Newline {
                    *line += 1;
                }
            } else {
                return Some(item);
            }
        }
        None
    }

    fn parse_arguments(
        &self,
        source: &'source str,
        iter: &mut IntoIter<(Parameters, Span)>,
        byte_offset: &mut usize,
        line: &mut usize,
        call: &mut Call<'source>,
    ) -> Option<(Parameters, Span)> {
        let next =
            self.consume_whitespace(iter, byte_offset, line);
        if let Some((lex, span)) = next {

            println!("Parameter lex {:?}", lex);

            match lex {
                grammar::Parameters::Null => {
                    call.add_argument(ParameterValue::Json(Value::Null));
                }
                grammar::Parameters::True => {
                    call.add_argument(ParameterValue::Json(Value::Bool(true)));
                }
                grammar::Parameters::False => {
                    call.add_argument(ParameterValue::Json(Value::Bool(false)));
                }
                grammar::Parameters::Number => {
                    let num: Number = source[span].parse().unwrap();
                    call.add_argument(ParameterValue::Json(Value::Number(num)));
                }
                grammar::Parameters::StringLiteral => {
                    println!("Got string literal start...");
                    let str_start = span.end;
                    let mut str_end = span.end;
                    let mut terminated = false;
                    while let Some((lex, span)) = iter.next() {
                        match lex {
                            grammar::Parameters::StringToken(s) => {
                                println!("Got string literal token...");
                                match s {
                                    grammar::StringLiteral::End => {
                                        terminated = true;
                                        break;
                                    }
                                    _ => {
                                        *byte_offset = span.end;
                                        str_end = span.end; 
                                    }
                                }
                            }
                            _ => {
                                panic!("Expected string token!");
                            }
                        }
                    }

                    // FIXME: error on non-terminated string literal

                    let str_value = &source[str_start..str_end];
                    call.add_argument(ParameterValue::Json(Value::String(str_value.to_string())));
                }
                _ => return None
            }
            return self.parse_arguments(source, iter, byte_offset, line, call)
        }

        iter.next()
    }

    fn parse_parameters(
        &mut self,
        source: &'source str,
        line: &mut usize,
        statement: ParameterCache,
    ) -> Result<Call<'source>, SyntaxError<'source>> {
        let context = statement.context.clone();
        let stmt_start = statement.start.clone();
        let stmt_end = statement.end.clone();
        let mut iter = statement.tokens.into_iter();

        // Position as byte offset for syntax errors
        let mut byte_offset = stmt_start.end.clone();

        let next = self.consume_whitespace(&mut iter, &mut byte_offset, line);

        if let Some((mut first, mut span)) = next {
            let mut identifier: Option<(Parameters, Span)> = None;

            let partial = match first {
                Parameters::Partial => true,
                _ => false,
            };

            if partial {
                let next =
                    self.consume_whitespace(&mut iter, &mut byte_offset, line);
                if let Some((lex, next_span)) = next {
                    first = lex;
                    span = next_span;
                }
            }

            let mut call = Call::new(
                source,
                partial,
                stmt_start,
                stmt_end,
                Path(vec![]),
            );

            match context {
                ParameterContext::Block => {
                    if Parameters::Identifier != first {
                        return Err(SyntaxError::BlockIdentifier(self.err_info(
                            source,
                            line,
                            byte_offset,
                        )));
                    }
                    identifier = Some((first, span));
                }
                ParameterContext::Statement => {
                    match first {
                        Parameters::Identifier
                        | Parameters::LocalIdentifier => {
                            identifier = Some((first, span));
                        }
                        Parameters::Partial => {
                            let next = self.consume_whitespace(
                                &mut iter,
                                &mut byte_offset,
                                line,
                            );

                            if let Some((lex, span)) = next {
                                let expects = vec![
                                    Parameters::Identifier,
                                    Parameters::LocalIdentifier,
                                ];
                                if expects.contains(&lex) {
                                    identifier = Some((lex, span));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            println!("Parse statement with identifier {:?}", identifier);

            if identifier.is_none() {
                return Err(SyntaxError::ExpectedIdentifier(self.err_info(
                    source,
                    line,
                    byte_offset,
                )));
            }

            self.parse_arguments(source, &mut iter, &mut byte_offset, line, &mut call);

            println!("Arguments {:?}", call.arguments());

            Ok(call)
        } else {
            Err(SyntaxError::EmptyStatement(self.err_info(source, line, byte_offset)))
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
            Token::StringLiteral(lex, _) => lex == &grammar::StringLiteral::Newline,
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
                            Block::new(s, BlockType::RawBlock, Some(span)),
                            &mut text,
                        );
                    }
                    grammar::Block::StartRawComment => {
                        self.enter_stack(
                            Block::new(s, BlockType::RawComment, Some(span)),
                            &mut text,
                        );
                    }
                    grammar::Block::StartRawStatement => {
                        self.enter_stack(
                            Block::new(s, BlockType::RawStatement, Some(span)),
                            &mut text,
                        );
                    }
                    grammar::Block::StartComment => {
                        self.enter_stack(
                            Block::new(s, BlockType::Comment, Some(span)),
                            &mut text,
                        );
                    }
                    grammar::Block::StartBlockScope => {
                        parameters = Some(ParameterCache::new(
                            ParameterContext::Block,
                            span.clone(),
                        ));

                        self.enter_stack(
                            Block::new(s, BlockType::Scoped, Some(span)),
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

                            let call = self.parse_parameters(
                                s,
                                &mut line,
                                params.clone(),
                            )?;
                            match ctx {
                                ParameterContext::Statement => {
                                    let current =
                                        self.stack.last_mut().unwrap();
                                    current.push(Node::Statement(call));
                                }
                                ParameterContext::Block => {
                                    let current =
                                        self.stack.last_mut().unwrap();
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
                    _ => {
                        println!("GOT STRING LITERAL TOKEN {:?}", lex);
                        if let Some(params) = parameters.as_mut() {
                            params.tokens.push((Parameters::StringToken(lex), span));
                        }
                    }
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
