use logos::Logos;
use std::fmt;
use std::ops::Range;

use crate::{
    error::{RenderError, SyntaxError},
    lexer::{
        ast::{self, *},
        grammar::{Statement, Token},
        parser, SourceInfo,
    },
    render::{RenderContext, Renderer},
};

#[derive(Debug)]
pub struct Template<'source> {
    ast: Block<'source>,
}

impl<'source> Template<'source> {
    pub fn block(&self) -> &'source Block {
        &self.ast
    }
}

impl fmt::Display for Template<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ast.fmt(f)
    }
}

impl<'reg, 'render> Renderer<'reg, 'render> for Template<'_> {
    fn render(
        &self,
        rc: &mut RenderContext<'reg, 'render>,
    ) -> Result<(), RenderError> {
        self.ast.render(rc)
    }
}

impl<'source> Template<'source> {
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
                    first.info.span.end = last_info.span.end;
                    first.info.line.end = last_info.line.end;
                    first.value =
                        &s[first.info.span.start..first.info.span.end];

                    let item = text.swap_remove(0);
                    current.push(ast::Token::Text(item));
                    text.clear();
                }
            }
        }
    }

    /// Compile a block.
    pub fn compile(s: &'source str) -> Result<Template, SyntaxError> {
        let lex = Token::lexer(s);
        let mut ast = Block::new(BlockType::Root);
        let mut stack: Vec<Block> = vec![];
        let mut line = 0;

        let mut last: Option<Block> = None;

        let mut text: Vec<Text> = vec![];

        for (token, span) in lex.spanned().into_iter() {
            let len = stack.len();
            let current = if stack.is_empty() {
                &mut ast
            } else {
                stack.get_mut(len - 1).unwrap()
            };

            if let Some(last) = last.take() {
                current.push(ast::Token::Block(last));
            }

            //println!("{:?} ({:?})", token, span);

            let info = SourceInfo {
                line: Range {
                    start: line,
                    end: line,
                },
                span,
            };

            // Normalize consecutive characters into a single text block.
            Template::normalize(s, current, &token, &mut text);

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
                        match current.block_type {
                            BlockType::Raw => true,
                            _ => false,
                        }
                    };

                    if !is_raw {
                        let statement = Template::statement(expr.value())?;
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
                    last = stack.pop();
                    if let Some(ref mut block) = last {
                        if !block.is_raw() {
                            return Err(SyntaxError::BadEndRawBlock);
                        }

                        block.close = Some(value);
                    } else {
                        return Err(SyntaxError::BadEndBlock);
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
                            return Err(SyntaxError::BadEndNamedBlock);
                        }

                        let name = parser::block_name(&value);

                        match block.block_type {
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

        // Force text normalization if we end with text
        if !text.is_empty() {
            let len = stack.len();
            let current = if stack.is_empty() {
                &mut ast
            } else {
                stack.get_mut(len - 1).unwrap()
            };

            let token = Token::Error;
            Template::normalize(s, current, &token, &mut text);
        }

        Ok(Template { ast })
    }
}
