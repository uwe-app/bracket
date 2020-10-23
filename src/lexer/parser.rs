use std::ops::Range;

use super::{
    ast::{self, Token, Block, BlockType, Expr, Text, SourceInfo},
    grammar::{self, lex, Token as LexToken},
};

use crate::error::SyntaxError;

#[derive(Debug)]
pub struct Parser<'source> {
    stack: Vec<Block<'source>>,
}

impl<'source> Parser<'source> {

    pub fn new() -> Self {
        Self {stack: vec![Block::new("", BlockType::Root, None)]}
    }

    pub fn parse(&mut self, s: &'source str) -> Result<Block<'source>, SyntaxError> {
        let tokens = lex(s, false);

        // Consecutive text to normalize
        let mut text: Option<Text> = None;

        for t in tokens.into_iter() {
            //println!("Parser {:?}", t);

            match &t {
                LexToken::Block(lex, span) => {
                    match lex {
                        grammar::Block::StartRawComment => {
                            let mut block = Block::new(s, BlockType::Raw, Some(span.clone()));
                            self.stack.push(block);
                            continue;
                        }
                        _ => {}
                    } 
                }
                LexToken::RawComment(lex, span) => {
                    match lex {
                        grammar::RawComment::End => {
                            // Must consume the text now!
                            if let Some(txt) = text.take() {
                                let current = self.stack.last_mut().unwrap();
                                current.push(Token::Text(txt));
                            } 
                            self.stack.pop();
                            continue;
                        }
                        _ => {}
                    } 
                }
                _ => {}
            }

            let current = self.stack.last_mut().unwrap();
            //println!("Current {:?}", current.block_type());

            if t.is_text() {
                let txt = text.get_or_insert(Text(s, t.span().clone()));
                txt.1.end = t.span().end;
            } else {
                if let Some(txt) = text.take() {
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

    /*
    /// Compile a statement.
    fn statement(s: &'source str) -> Result<ast::Statement, SyntaxError> {
        let statement: ast::Statement = Default::default();

        println!("Lex statement: {}", s);

        let lex = Statement::lexer(s);
        for (token, span) in lex.spanned().into_iter() {
            println!("Statement token {:?} {:?}", token, span);
        }

        Ok(statement)
    }

    fn coalesce(
        s: &'source str,
        tokens: &mut Vec<Range<usize>>,
    ) -> &'source str {
        let last = tokens.last().unwrap().clone();
        let first = tokens.get_mut(0).unwrap();
        first.end = last.end;
        &s[first.start..first.end]
    }

    fn range_slice(
        s: &'source str,
        first: &mut SourceInfo,
        last: &SourceInfo,
    ) -> &'source str {
        first.span.end = last.span.end;
        first.line.end = last.line.end;
        &s[first.span.start..first.span.end]
    }

    fn normalize(
        s: &'source str,
        current: &mut Block<'source>,
        token: &Token,
        text: &mut Vec<Text<'source>>,
    ) {
        match token {
            Token::Char(_) | Token::Newline(_) => {}
            _ => {
                if !text.is_empty() {
                    let last_info = text.last().unwrap().info.clone();
                    let first = text.get_mut(0).unwrap();
                    first.value =
                        Parser::range_slice(s, &mut first.info, &last_info);
                    let item = text.swap_remove(0);
                    current.push(ast::Token::Text(item));
                    text.clear();
                }
            }
        }
    }

    pub fn parse(s: &'source str) -> Result<Block, SyntaxError> {
        let lex = Token::lexer(s);
        let mut ast = Block::new(BlockType::Root);
        let mut stack: Vec<Block> = vec![];
        let mut line = 0;

        let mut last: Option<Block> = None;

        let mut text: Vec<Text> = vec![];
        let mut raw: Vec<Range<usize>> = vec![];

        for (token, span) in lex.spanned().into_iter() {
            let len = stack.len();
            let current = if stack.is_empty() {
                &mut ast
            } else {
                stack.get_mut(len - 1).unwrap()
            };

            let mut info = SourceInfo {
                line: Range {
                    start: line,
                    end: line,
                },
                span,
            };

            // Normalize raw blocks into a single string slice
            match current.block_type() {
                BlockType::Raw => match token {
                    Token::Newline(value) => {
                        line = line + 1;
                        raw.push(info.span);
                        continue;
                    }
                    Token::EndRawBlock(value) => {
                        let mut val = if !raw.is_empty() {
                            let value = Parser::coalesce(s, &mut raw);
                            let span = raw.swap_remove(0);
                            Some((span, value))
                        } else {
                            None
                        };

                        if let Some((span, value)) = val.take() {
                            info.set_range(span);
                            current.replace(info, value);
                        }
                        raw.clear();

                        last = stack.pop();

                        if let Some(ref mut block) = last {
                            block.close = Some(value);
                        }

                        continue;
                    }
                    _ => {
                        raw.push(info.span);
                        continue;
                    }
                },
                _ => {}
            }

            if let Some(last) = last.take() {
                current.push(ast::Token::Block(last));
            }

            //println!("{:?} ({:?})", token, span);

            // Normalize consecutive characters into a single text block.
            Parser::normalize(s, current, &token, &mut text);

            match token {
                Token::Char(value) => {
                    text.push(Text { value, info });
                }
                Token::Newline(value) => {
                    text.push(Text { value, info });
                    line = line + 1;
                }
                Token::Expression(value) => {
                    let expr = Expr::new(info, value);

                    // Skip escaped (\) expressions and
                    // those inside raw blocks.
                    let is_raw = expr.is_raw() || {
                        match current.block_type() {
                            BlockType::Raw => true,
                            _ => false,
                        }
                    };

                    if !is_raw {
                        let statement = Parser::statement(expr.value())?;
                        println!("Statement {:?}", statement);
                    }

                    current.push(ast::Token::Expression(expr));
                }
                Token::StartCommentBlock(value) => {
                    let mut block = Block::new(BlockType::Comment);
                    block.open = Some(value);
                    stack.push(block);
                }
                Token::EndCommentBlock(value) => {
                    last = stack.pop();
                    // TODO: check end comment matches the start
                    if let Some(ref mut block) = last {
                        block.close = Some(value);
                    }
                }
                Token::StartRawBlock(value) => {
                    let mut block = Block::new(BlockType::Raw);
                    block.open = Some(value);
                    stack.push(block);
                }
                Token::EndRawBlock(value) => {
                    // NOTE: raw blocks coalesce their content
                    // NOTE: into a single slice and have special
                    // NOTE: handling above
                }
                Token::StartBlock(value) => {
                    let block = Block::new_named(value);
                    stack.push(block);
                }
                Token::EndBlock(value) => {
                    // TODO: check the end block name matches
                    last = stack.pop();
                    if let Some(ref mut block) = last {
                        if !block.is_named() {
                            return Err(SyntaxError::BadEndNamedBlock);
                        }

                        let name = block_name(&value);

                        match block.block_type() {
                            BlockType::Named(ref start_name) => {
                                if start_name != &name {
                                    return Err(SyntaxError::BadBlockEndName(
                                        start_name.to_string(),
                                        name.to_string(),
                                    ));
                                }
                            }
                            _ => {}
                        }

                        block.close = Some(value);
                    } else {
                        return Err(SyntaxError::BadEndBlock);
                    }
                }
                Token::Error => {
                    return Err(SyntaxError::InvalidToken);
                }
            }
        }

        let len = stack.len();
        let current = if stack.is_empty() {
            &mut ast
        } else {
            stack.get_mut(len - 1).unwrap()
        };

        // Force text normalization if we end with text
        if !text.is_empty() {
            let token = Token::Error;
            Parser::normalize(s, current, &token, &mut text);
        }

        if !raw.is_empty() {
            return Err(SyntaxError::RawBlockNotTerminated);
        }

        if let Some(last) = last.take() {
            current.push(ast::Token::Block(last));
        }

        Ok(ast)
    }
    */
}
