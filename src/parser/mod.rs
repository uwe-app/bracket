use logos::Span;

use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{self, lex, Lexer, Parameters, Token},
    parser::ast::{Document, Block, Node, Text, CallTarget},
};

/// Default file name.
static UNKNOWN: &str = "unknown";

mod arguments;
pub mod ast;
mod block;
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

pub struct Parser<'source> {
    source: &'source str,
    lexer: Lexer<'source>,
    state: ParseState,
    options: ParserOptions,
    stack: Vec<(&'source str, Block<'source>)>,
    next_token: Option<Token>,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str, options: ParserOptions) -> Self {
        let lexer = lex(source);
        let state = ParseState::from(&options);
        Self {
            source,
            lexer,
            state,
            options,
            stack: vec![],
            next_token: None,
        }
    }

    /// Parse the entire document into a node tree.
    pub fn parse(&mut self) -> Result<Node<'source>, SyntaxError<'source>> {
        let mut doc = Document(&self.source, vec![]);
        for node in self {
            let node = node?;
            doc.nodes_mut().push(node);
        }
        Ok(Node::Document(doc))
    }

    /// Yield the next token accounting for text normalization which 
    /// saves the next token for further processing.
    fn token(&mut self) -> Option<Token> {
        if let Some(t) = self.next_token.take() {
            self.next_token = None;
            Some(t)
        } else {
            self.lexer.next()
        }
    }

    /// Consume tokens and yield nodes.
    ///
    /// Decoupled from the iterator `next()` implementation as it needs to 
    /// greedily consume tokens and advance again when entering block scopes.
    fn advance(&mut self, next: Token) -> Result<Option<Node<'source>>, SyntaxError<'source>> {

        if next.is_newline() {
            *self.state.line_mut() += 1;
        }

        // Normalize consecutive text nodes
        if next.is_text() {
            let (span, next) = block::until(
                &mut self.lexer,
                &mut self.state,
                next.span().clone(),
                &|t: &Token| !t.is_text(),
            );
            self.next_token = next;
            return Ok(Some(Node::Text(Text(self.source, span))));
        }

        //println!("Advance token {:?}", &next);

        match next {
            Token::Block(lex, span) => match lex {
                lexer::Block::StartRawBlock => {
                    return block::raw(
                        self.source,
                        &mut self.lexer,
                        &mut self.state,
                        span,
                    ).map(Some);
                }
                lexer::Block::StartRawComment => {
                    return block::raw_comment(
                        self.source,
                        &mut self.lexer,
                        &mut self.state,
                        span,
                    ).map(Some);
                }
                lexer::Block::StartRawStatement => {
                    return block::raw_statement(
                        self.source,
                        &mut self.lexer,
                        &mut self.state,
                        span,
                    ).map(Some);
                }
                lexer::Block::StartComment => {
                    return block::comment(
                        self.source,
                        &mut self.lexer,
                        &mut self.state,
                        span,
                    ).map(Some);
                }
                lexer::Block::StartBlockScope => {
                    let block = block::scope(
                        self.source,
                        &mut self.lexer,
                        &mut self.state,
                        span,
                    )?;

                    if let Some(block) = block {

                        let name = block.name().ok_or_else(|| {
                            return SyntaxError::ExpectedIdentifier(
                                ErrorInfo::new(
                                    self.source,
                                    self.state.file_name(),
                                    SourcePos::from((
                                        self.state.line(),
                                        self.state.byte(),
                                    )),
                                ),
                            )
                        })?;

                        match block.call().target() {
                            CallTarget::Path(ref path) => {
                                if !path.is_simple() {
                                    return Err(SyntaxError::ExpectedSimpleIdentifier(
                                        ErrorInfo::new(
                                            self.source,
                                            self.state.file_name(),
                                            SourcePos::from((
                                                self.state.line(),
                                                self.state.byte(),
                                            )),
                                        ),
                                    ))
                                } 
                            } 
                            CallTarget::SubExpr(_) => {
                                if !block.call().is_partial() {
                                    panic!("Sub expression block targets are only evaluated for partials");
                                } 
                            }
                        }

                        //println!("Adding block to the stack...");
                        self.stack.push((name, block));

                        while let Some(t) = self.token() {
                            //println!("Stack is consuming the token {:?}", t);
                            match self.advance(t) {
                                Ok(node) => {
                                    //println!("Got a node to add {:?}", node);
                                    if node.is_none() || self.stack.is_empty() {
                                        return Ok(node);
                                    } else {
                                        let (_, current) = self.stack.last_mut().unwrap();
                                        current.push(node.unwrap());
                                    }
                                }
                                Err(e) => return Err(e),
                            }
                        }
                    } else {
                        // FIXME: use SyntaxError
                        panic!("Block open statement not terminated!");
                    }
                }
                lexer::Block::EndBlockScope => {
                    // Need a temp block to parse the call parameters so we 
                    // can match the tag end name
                    let temp = block::scope(
                        self.source,
                        &mut self.lexer,
                        &mut self.state,
                        span.clone(),
                    )?;

                    if self.stack.is_empty() {
                        let close_name = if let Some(close) = temp {
                            close.name()
                        } else { None };

                        let notes = if let Some(close) = close_name {
                            vec![format!("perhaps open the block '{}'", close)]
                        } else {
                            vec![]
                        };

                        *self.state.byte_mut() = span.start;

                        return Err(SyntaxError::BlockNotOpen(
                            ErrorInfo::new_notes(
                                self.source,
                                self.state.file_name(),
                                SourcePos::from((
                                    self.state.line(),
                                    self.state.byte(),
                                )),
                                notes
                            ),
                        ));
                        //panic!("Got close block with no open block!");
                    }

                    let (open_name, mut block) = self.stack.pop().unwrap();

                    if let Some(close) = temp {
                        if let Some(close_name) = close.name() {
                            if open_name != close_name {
                                return Err(SyntaxError::TagNameMismatch(
                                    ErrorInfo::new_notes(
                                        self.source,
                                        self.state.file_name(),
                                        SourcePos::from((
                                            self.state.line(),
                                            self.state.byte(),
                                        )),
                                        vec![format!("opening name is '{}'", open_name)]
                                    ),
                                ));
                            }

                            // TODO: update span for entire close tag: `{{/name}}`!
                            block.exit(span);

                            return Ok(Some(Node::Block(block)))
                        } else {
                            return Err(SyntaxError::ExpectedIdentifier(
                                ErrorInfo::new(
                                    self.source,
                                    self.state.file_name(),
                                    SourcePos::from((
                                        self.state.line(),
                                        self.state.byte(),
                                    )),
                                ),
                            ))
                        }

                    } else {
                        panic!("Unable to parse call parameters for close block");
                    }
                }
                lexer::Block::StartStatement => {
                    match block::parameters(
                        self.source,
                        &mut self.lexer,
                        &mut self.state,
                        span,
                        ParameterContext::Statement,
                    ) {
                        Ok(mut parameters) => {
                            if let Some(params) = parameters.take() {
                                match statement::parse(
                                    self.source,
                                    &mut self.state,
                                    params,
                                ) {
                                    Ok(call) => {
                                        return Ok(Some(Node::Statement(
                                            call,
                                        )))
                                    }
                                    Err(e) => return Err(e),
                                }
                            } else {
                                // FIXME: use SyntaxError
                                panic!("Statement not terminated");
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                _ => {}
            },
            Token::RawBlock(_, _) => {}
            Token::RawComment(_, _) => {}
            Token::RawStatement(_, _) => {}
            Token::Comment(_, _) => {}
            Token::Parameters(_, _) => {}
            Token::StringLiteral(_, _) => {}
        }
        
        Ok(None)
    }
}

impl<'source> Iterator for Parser<'source> {
    type Item = Result<Node<'source>, SyntaxError<'source>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t) = self.token() {
            match self.advance(t) {
                Ok(node) => return node.map(Ok),
                Err(e) => return Some(Err(e)),
            }
        }

        None
    }
}
