use std::fmt;
use std::ops::Range;
use logos::Logos;

use crate::{
    Error,
    Result,
    lexer::{
        self,
        Block,
        BlockType,
        AstToken,
        Token,
        Expression,
        Text,
        SourceInfo,
    }
};

#[derive(Debug)]
pub struct Template {
    ast: Block,
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ast.fmt(f)
    }
}

impl Template {

    pub fn compile(s: &str) -> Result<Template> {
        let lex = Token::lexer(s);
        let mut ast = Block::new(BlockType::Root);
        let mut stack: Vec<Block> = vec![];
        let mut line = 0;

        let mut last: Option<Block> = None;

        for (token, span) in lex.spanned().into_iter() {

            let len = stack.len();
            let current = if stack.is_empty() {
                &mut ast
            } else {
                stack.get_mut(len - 1).unwrap()
            };

            if let Some(last) = last.take() {
                current.push(AstToken::Block(last));
            }

            println!("{:?}", token);

            let info = SourceInfo {line: Range {start: line, end: line}, span};
            match token {
                Token::Text(value) => {
                    current.add_text(info, value);
                }
                Token::Newline(value) => {
                    current.add_text(info, value);
                    line = line + 1; 
                }
                Token::Expression(value) => {
                    current.push(AstToken::Expression(Expression {info, value}));
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
                    last = stack.pop();
                    if let Some(ref mut block) = last {
                        if !block.is_raw() {
                            return Err(Error::BadEndRawBlock)
                        }

                        block.close = Some(value);
                    } else {
                        return Err(Error::BadEndBlock)
                    }
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
                            return Err(Error::BadEndNamedBlock)
                        }

                        let name = lexer::parse_block_name(&value);

                        match block.block_type {
                            BlockType::Named(ref start_name) => {
                                if start_name != &name {
                                    return Err(Error::BadBlockEndName(start_name.to_string(), name))
                                }
                            }
                            _ => {}
                        }

                        block.close = Some(value);
                    } else {
                        return Err(Error::BadEndBlock)
                    }
                }
                Token::Error => {
                    return Err(Error::InvalidToken);
                }
            }
        }

        Ok(Template {ast})
    }
}

