use std::fmt;
use std::ops::Range;

use super::{
    ast::{self, Block, BlockType, Text},
    grammar::{self, lex, Token, LineNumber, Span},
};

use crate::error::SyntaxError;

#[derive(Default, Debug, Eq, PartialEq)]
pub struct LineRange {
    range: LineNumber
}

impl fmt::Display for LineRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.range.start + 1, self.range.end + 1)
    }
}

impl From<LineNumber> for LineRange {
    fn from(lines: LineNumber) -> Self {
        Self {range: lines}
    }
}

#[derive(Default, Debug)]
struct Statement {
    tokens: Vec<Token>,
    start: (Span, LineNumber),
    end: (Span, LineNumber),
}

impl Statement {
    pub fn lines(&self) -> LineRange {
        LineRange {range: self.start.1.start..self.end.1.end }
    }
}

#[derive(Debug)]
pub struct Parser<'source> {
    stack: Vec<Block<'source>>,
}

impl<'source> Parser<'source> {
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    fn enter_stack(&mut self, block: Block<'source>, text: &mut Option<Text<'source>>) {
        // Must consume the text now!
        if let Some(txt) = text.take() {
            if let Some(current) = self.stack.last_mut() {
                current.push(Block::from(txt));
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
            current.push(Block::from(txt));
        }

        current.exit(close);
        let mut last = self.stack.pop();
        if let Some(block) = last.take() {
            // Add the current block to the tree
            let current = self.stack.last_mut().unwrap();
            current.push(block);
        }
    }

    fn parse_statement(&mut self, statement: &mut Statement) -> Result<(), SyntaxError> {
        println!("Parse statement {:?}", statement);
        if !statement.tokens.is_empty() {
            let first = statement.tokens.swap_remove(0);
            Ok(())
        } else {
            Err(SyntaxError::EmptyStatement { lines: LineRange::from(statement.lines()) })
        }

    }

    pub fn parse(
        &mut self,
        s: &'source str,
    ) -> Result<Block<'source>, SyntaxError> {
        let tokens = lex(s, false);

        // Consecutive text to normalize
        let mut text: Option<Text> = None;
        //let mut statement: Vec<Token> = Vec::new();
        let mut statement: Statement = Default::default();

        self.enter_stack(Block::new(s, BlockType::Root, None), &mut text);

        for t in tokens.into_iter() {
            //println!("Parser {:?}", t);

            match &t {
                Token::Block(lex, span, lines) => match lex {
                    grammar::Block::StartRawBlock => {
                        self.enter_stack(Block::new(
                            s,
                            BlockType::RawBlock,
                            Some(span.clone()),
                        ), &mut text);
                        continue;
                    }
                    grammar::Block::StartRawComment => {
                        self.enter_stack(Block::new(
                            s,
                            BlockType::RawComment,
                            Some(span.clone()),
                        ), &mut text);
                        continue;
                    }
                    grammar::Block::StartRawStatement => {
                        self.enter_stack(Block::new(
                            s,
                            BlockType::RawStatement,
                            Some(span.clone()),
                        ), &mut text);
                        continue;
                    }
                    grammar::Block::StartComment => {
                        self.enter_stack(Block::new(
                            s,
                            BlockType::Comment,
                            Some(span.clone()),
                        ), &mut text);
                        continue;
                    }
                    grammar::Block::StartStatement => {
                        self.enter_stack(Block::new(
                            s,
                            BlockType::Comment,
                            Some(span.clone()),
                        ), &mut text);

                        statement = Default::default();
                        statement.start = (span.clone(), lines.clone());
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
                    grammar::Statement::End => {
                        statement.end = (span.clone(), lines.clone());
                        self.parse_statement(&mut statement)?;
                        self.exit_stack(span.clone(), &mut text);
                        continue;
                    }
                    _ => {
                        statement.tokens.push(t);
                        continue;
                    }
                }
                _ => {}
            }

            let current = self.stack.last_mut().unwrap();

            if t.is_text() {
                let txt = text.get_or_insert(Text(s, t.span().clone()));
                txt.1.end = t.span().end;
            } else {
                if let Some(txt) = text.take() {
                    current.push(Block::from(txt));
                }
            }
        }

        // Must append any remaining normalized text!
        if let Some(txt) = text.take() {
            let current = self.stack.last_mut().unwrap();
            current.push(Block::from(txt));
        }

        //println!("{:#?}", self.stack.first());

        Ok(self.stack.swap_remove(0))
    }
}
