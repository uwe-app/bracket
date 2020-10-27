use std::ops::Range;
use std::vec::IntoIter;

use serde_json::{Number, Value};

use logos::Span;

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{
        ast::{
            Block, BlockType, Call, Component, ComponentType, Node,
            ParameterValue, Path, Text,
        },
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
        byte_offset: &mut usize,
        notes: Option<Vec<&'static str>>,
    ) -> ErrorInfo<'source> {
        let pos = SourcePos(line.clone(), byte_offset.clone());
        ErrorInfo::from((source, &self.options, pos, notes.unwrap_or(vec![])))
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
        let next = self.consume_whitespace(iter, byte_offset, line);
        if let Some((lex, span)) = next {
            //println!("Parameter lex {:?}", lex);

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
                    let str_start = span.end;
                    let mut str_end = span.end;
                    while let Some((lex, span)) = iter.next() {
                        match lex {
                            grammar::Parameters::StringToken(s) => match s {
                                grammar::StringLiteral::End => {
                                    break;
                                }
                                _ => {
                                    *byte_offset = span.end;
                                    str_end = span.end;
                                }
                            },
                            _ => {
                                panic!("Expected string token!");
                            }
                        }
                    }

                    let str_value = &source[str_start..str_end];
                    call.add_argument(ParameterValue::Json(Value::String(
                        str_value.to_string(),
                    )));
                }
                _ => return None,
            }
            return self.parse_arguments(source, iter, byte_offset, line, call);
        }

        iter.next()
    }

    fn starts_path(&self, lex: &Parameters) -> bool {
        match lex {
            Parameters::ExplicitThisRef
            | Parameters::ParentRef
            | Parameters::Identifier
            | Parameters::LocalIdentifier => true,
            _ => false,
        }
    }

    fn is_path_component(&self, lex: &Parameters) -> bool {
        match lex {
            _ => self.starts_path(lex) || lex == &Parameters::PathDelimiter,
        }
    }

    fn component_type(&self, lex: &Parameters) -> ComponentType {
        match lex {
            Parameters::ExplicitThisRef => ComponentType::This,
            Parameters::ParentRef => ComponentType::Parent,
            Parameters::Identifier => ComponentType::Identifier,
            Parameters::LocalIdentifier => ComponentType::LocalIdentifier,
            Parameters::PathDelimiter => ComponentType::Delimiter,
            _ => panic!("Expecting component parameter in parser"),
        }
    }

    fn parse_path(
        &self,
        source: &'source str,
        iter: &mut IntoIter<(Parameters, Span)>,
        byte_offset: &mut usize,
        line: &mut usize,
        current: Option<(Parameters, Span)>,
        path: &mut Path<'source>,
    ) -> Option<(Parameters, Span)> {
        if let Some((mut lex, mut span)) = current {
            if self.starts_path(&lex) {
                *byte_offset = span.end;

                // Consume parent references
                match &lex {
                    Parameters::ParentRef => {
                        let mut parents = 1;
                        while let Some((next_lex, next_span)) = iter.next() {
                            match &next_lex {
                                Parameters::ParentRef => {
                                    parents += 1;
                                }
                                _ => {
                                    lex = next_lex;
                                    span = next_span;
                                    break;
                                }
                            }
                        }
                        path.set_parents(parents);
                    }
                    _ => {}
                }

                let component = Component(source, self.component_type(&lex), span);

                // Flag as a path that should be resolved from the root object
                if path.is_empty() && component.is_root() {
                    path.set_root(true);
                }

                path.add_component(component);
                while let Some((lex, span)) = iter.next() {
                    if self.is_path_component(&lex) {
                        path.add_component(Component(source, self.component_type(&lex), span));
                    } else {
                        return Some((lex, span));
                    }
                }
            }
        }
        None
    }

    fn parse_partial(
        &self,
        source: &'source str,
        iter: &mut IntoIter<(Parameters, Span)>,
        byte_offset: &mut usize,
        line: &mut usize,
        current: Option<(Parameters, Span)>,
    ) -> (bool, Option<(Parameters, Span)>) {
        if let Some((lex, span)) = current {
            match lex {
                Parameters::Partial => {
                    let next = self.consume_whitespace(iter, byte_offset, line);
                    return (true, next);
                }
                _ => {
                    return (false, Some((lex, span)));
                }
            }
        }
        (false, None)
    }

    fn parse_parameters(
        &mut self,
        source: &'source str,
        line: &mut usize,
        byte_offset: &mut usize,
        statement: ParameterCache,
    ) -> Result<Call<'source>, SyntaxError<'source>> {
        let context = statement.context.clone();
        let stmt_start = statement.start.clone();
        let stmt_end = statement.end.clone();
        let mut iter = statement.tokens.into_iter();

        // Position as byte offset for syntax errors
        *byte_offset = stmt_start.end;

        let next = self.consume_whitespace(&mut iter, byte_offset, line);

        //println!("Next {:?}", next);

        if next.is_none() {
            return Err(SyntaxError::EmptyStatement(self.err_info(
                source,
                line,
                byte_offset,
                None,
            )));
        }

        //println!("After leading whitespce {:?}", next);
        let (partial, next) =
            self.parse_partial(source, &mut iter, byte_offset, line, next);
        //println!("After partial parse {:?} {:?}", partial, &next);
        if partial && next.is_none() {
            return Err(SyntaxError::PartialIdentifier(self.err_info(
                source,
                line,
                byte_offset,
                None,
            )));
        }

        let mut call = Call::new(source, partial, stmt_start, stmt_end);

        let next = self.parse_path(
            source,
            &mut iter,
            byte_offset,
            line,
            next,
            call.path_mut(),
        );

        //println!("Got path {:?}", call.path());
        //println!("Got path {:?}", call.path().is_simple());

        if partial && !call.path().is_simple() {
            return Err(SyntaxError::PartialSimpleIdentifier(self.err_info(
                source,
                line,
                byte_offset,
                None,
            )));
        }

        match context {
            ParameterContext::Block => {
                if !call.path().is_simple() {
                    panic!("Blocks require a simple identifier, not a path!");
                }
            }
            ParameterContext::Statement => {
                // TODO: validate statement paths?
            }
        }

        if call.path().is_empty() {
            return Err(SyntaxError::ExpectedIdentifier(self.err_info(
                source,
                line,
                byte_offset,
                None,
            )));
        }

        self.parse_arguments(source, &mut iter, byte_offset, line, &mut call);

        println!("Arguments {:?}", call.arguments());

        Ok(call)
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
        s: &'source str,
    ) -> Result<Node<'source>, SyntaxError<'source>> {
        // Consecutive text to normalize
        let mut text: Option<Text> = None;

        let mut parameters: Option<ParameterCache> = None;
        let mut line = &mut self.options.line_offset.clone();
        let mut byte_offset = &mut self.options.byte_offset.clone();

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
                *line += 1;
                continue;
            }

            println!("Parser {:?}", t);

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
                        println!("Got parameters end!!");
                        if let Some(mut params) = parameters.take() {
                            let ctx = params.context.clone();
                            params.end = span;

                            let call = self.parse_parameters(
                                s,
                                line,
                                byte_offset,
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
                    grammar::StringLiteral::Newline => {
                        if let Some(params) = parameters.take() {
                            if let Some((lex, span)) = params.tokens.last() {
                                *byte_offset = span.end - 1;
                            }
                        }

                        return Err(SyntaxError::StringLiteralNewline(
                            self.err_info(s, line, byte_offset, None),
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

            return Err(SyntaxError::OpenStatement(self.err_info(
                s,
                line,
                byte_offset,
                Some(notes),
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
