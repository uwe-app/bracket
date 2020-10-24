use std::ops::Range;

use super::{
    ast::{self, Block, BlockType, Expr, SourceInfo, Text, Token},
    grammar::{self, lex, Token as LexToken},
};

use crate::error::SyntaxError;

#[derive(Debug)]
pub struct Parser<'source> {
    stack: Vec<Block<'source>>,
}

impl<'source> Parser<'source> {
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    fn enter_stack(&mut self, block: Block<'source>) {
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
            current.push(Token::Text(txt));
        }

        current.exit(close);
        let mut last = self.stack.pop();
        if let Some(block) = last.take() {
            println!("Got last block to add to the current parent...");
            let current = self.stack.last_mut().unwrap();
            current.push(Token::Block(block));
        }
    }

    pub fn parse(
        &mut self,
        s: &'source str,
    ) -> Result<Block<'source>, SyntaxError> {
        let tokens = lex(s, false);

        self.enter_stack(Block::new(s, BlockType::Root, None));

        // Consecutive text to normalize
        let mut text: Option<Text> = None;

        for t in tokens.into_iter() {
            //println!("Parser {:?}", t);

            match &t {
                LexToken::Block(lex, span) => match lex {
                    //grammar::Block::StartRawBlock => {
                        //self.enter_stack(Block::new(
                            //s,
                            //BlockType::RawBlock,
                            //Some(span.clone()),
                        //));
                        //continue;
                    //}
                    grammar::Block::StartRawComment => {
                        self.enter_stack(Block::new(
                            s,
                            BlockType::RawComment,
                            Some(span.clone()),
                        ));
                        continue;
                    }
                    grammar::Block::StartRawStatement => {
                        println!("START RAW STATEMENT");

                        self.enter_stack(Block::new(
                            s,
                            BlockType::RawStatement,
                            Some(span.clone()),
                        ));
                        continue;
                    }
                    _ => {}
                },
                //LexToken::RawBlock(lex, span) => match lex {
                    //grammar::RawBlock::End => {
                        //self.exit_stack(span.clone(), &mut text);
                        //continue;
                    //}
                    //_ => {}
                //},
                LexToken::RawComment(lex, span) => match lex {
                    grammar::RawComment::End => {
                        self.exit_stack(span.clone(), &mut text);
                        continue;
                    }
                    _ => {}
                },
                LexToken::RawStatement(lex, span) => match lex {
                    grammar::RawStatement::End => {
                        println!("END RAW STATEMENT");
                        self.exit_stack(span.clone(), &mut text);
                        continue;
                    }
                    _ => {}
                },
                _ => {}
            }

            let current = self.stack.last_mut().unwrap();
            //println!("Current {:?}", current.block_type());

            if t.is_text() {
                let txt = text.get_or_insert(Text(s, t.span().clone()));
                txt.1.end = t.span().end;
            } else {
                if let Some(txt) = text.take() {
                    println!("Adding text!");
                    current.push(Token::Text(txt));
                }
            }
        }

        if let Some(txt) = text.take() {
            let current = self.stack.last_mut().unwrap();
            current.push(Token::Text(txt));
        }

        Ok(self.stack.swap_remove(0))
    }
}
