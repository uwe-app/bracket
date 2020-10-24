use std::fmt;
use std::ops::Range;

use super::{
    ast::{Block, BlockType, Node, Text},
    grammar::{self, lex, LineNumber, Span, Token, Statement},
};

use crate::error::SyntaxError;

#[derive(Default, Debug, Eq, PartialEq)]
pub struct LineRange {
    range: LineNumber,
}

impl fmt::Display for LineRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.range.start + 1, self.range.end + 1)
    }
}

impl From<LineNumber> for LineRange {
    fn from(lines: LineNumber) -> Self {
        Self { range: lines }
    }
}

#[derive(Default, Debug)]
struct StatementCache {
    tokens: Vec<(Statement, Span)>,
    start: Span,
    end: Span,
}

#[derive(Debug)]
pub struct Parser<'source> {
    stack: Vec<Block<'source>>,
}

impl<'source> Parser<'source> {
    pub fn new() -> Self {
        Self { stack: vec![] }
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

    fn parse_statement(
        &mut self,
        statement: &mut StatementCache,
    ) -> Result<(), SyntaxError> {
        if !statement.tokens.is_empty() {
            let mut iter = statement.tokens.iter();
            let (first, span) = iter.next().unwrap();
            let mut identifier: Option<(&Statement, &Span)> = None;

            // Find the next token that exists in a list of expected tokens
            // at the next position consuming preceeding whitespace.
            let mut find_until = |expects: Vec<Statement>| -> Option<&(
                Statement,
                Span,
            )> {
                while let Some(item) = iter.next() {
                    if item.0 == Statement::WhiteSpace {
                        continue;
                    } else if expects.contains(&item.0) {
                        return Some(item);
                    }
                    break;
                }
                None
            };

            match first {
                Statement::Identifier => {
                    identifier = Some((first, span));
                }
                Statement::Partial => {
                    if let Some((lex, span)) =
                        find_until(vec![Statement::Identifier])
                    {
                        identifier = Some((lex, span));
                    }
                }
                _ => {}
            }

            if identifier.is_none() {
                return Err(SyntaxError::ExpectedIdentifier);
            }

            println!("Parse statement with identifier {:?}", identifier);

            Ok(())
        } else {
            Err(SyntaxError::EmptyStatement)
        }
    }

    pub fn parse(
        &mut self,
        s: &'source str,
    ) -> Result<Node<'source>, SyntaxError> {
        // Consecutive text to normalize
        let mut text: Option<Text> = None;
        //let mut statement: Vec<Token> = Vec::new();
        let mut statement: StatementCache = Default::default();

        self.enter_stack(Block::new(s, BlockType::Root, None), &mut text);

        for t in lex(s) {
            //println!("Parser {:?}", t);
            match &t {
                Token::Block(lex, span, lines) => match lex {
                    grammar::Block::StartRawBlock => {
                        self.enter_stack(
                            Block::new(
                                s,
                                BlockType::RawBlock,
                                Some(span.clone()),
                            ),
                            &mut text,
                        );
                        continue;
                    }
                    grammar::Block::StartRawComment => {
                        self.enter_stack(
                            Block::new(
                                s,
                                BlockType::RawComment,
                                Some(span.clone()),
                            ),
                            &mut text,
                        );
                        continue;
                    }
                    grammar::Block::StartRawStatement => {
                        self.enter_stack(
                            Block::new(
                                s,
                                BlockType::RawStatement,
                                Some(span.clone()),
                            ),
                            &mut text,
                        );
                        continue;
                    }
                    grammar::Block::StartComment => {
                        self.enter_stack(
                            Block::new(
                                s,
                                BlockType::Comment,
                                Some(span.clone()),
                            ),
                            &mut text,
                        );
                        continue;
                    }
                    grammar::Block::StartStatement => {
                        self.enter_stack(
                            Block::new(
                                s,
                                BlockType::Comment,
                                Some(span.clone()),
                            ),
                            &mut text,
                        );

                        statement = Default::default();
                        statement.start = span.clone();
                        continue;
                    }
                    _ => {}
                }
                Token::RawBlock(lex, span, _line) => match lex {
                    grammar::RawBlock::End => {
                        self.exit_stack(span.clone(), &mut text);
                        continue;
                    }
                    _ => {}
                }
                Token::RawComment(lex, span, _line) => match lex {
                    grammar::RawComment::End => {
                        self.exit_stack(span.clone(), &mut text);
                        continue;
                    }
                    _ => {}
                }
                Token::RawStatement(lex, span, _line) => match lex {
                    grammar::RawStatement::End => {
                        self.exit_stack(span.clone(), &mut text);
                        continue;
                    }
                    _ => {}
                }
                Token::Comment(lex, span, _line) => match lex {
                    grammar::Comment::End => {
                        self.exit_stack(span.clone(), &mut text);
                        continue;
                    }
                    _ => {}
                }
                Token::Statement(lex, span, lines) => match lex {
                    Statement::End => {
                        statement.end = span.clone();
                        self.parse_statement(&mut statement)?;
                        self.exit_stack(span.clone(), &mut text);
                        continue;
                    }
                    _ => {
                        statement.tokens.push((
                            lex.clone(),
                            span.clone(),
                        ));
                        continue;
                    }
                }
            }

            let current = self.stack.last_mut().unwrap();

            if t.is_text() {
                let txt = text.get_or_insert(Text(s, t.span().clone()));
                txt.1.end = t.span().end;
            } else {
                if let Some(txt) = text.take() {
                    current.push(Node::Text(txt));
                }
            }
        }

        // Must append any remaining normalized text!
        if let Some(txt) = text.take() {
            let current = self.stack.last_mut().unwrap();
            current.push(Node::Text(txt));
        }

        //println!("{:#?}", self.stack.first());

        Ok(Node::Block(self.stack.swap_remove(0)))
    }
}
