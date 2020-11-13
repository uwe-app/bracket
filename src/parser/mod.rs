//! Convert the lexer token stream to AST nodes.
use crate::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::{self, lex, Lexer, Token},
    parser::{
        ast::{Block, CallTarget, Condition, Document, Element, Node, Text},
        call::CallParseContext,
    },
    SyntaxResult,
};

/// Default file name.
static UNKNOWN: &str = "unknown";

pub mod ast;
mod block;
mod call;
mod iter;
pub(crate) mod path;
pub mod trim;

/// Set the file name used in error messages.
///
/// It is also possible to set the line and byte offsets if your template
/// is being extracted from a larger document.
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

impl ParserOptions {
    /// Create parser options using the given `file_name`.
    pub fn new(file_name: String) -> Self {
        Self {
            file_name,
            line_offset: 0,
            byte_offset: 0,
        }
    }
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

    /// Create a parser state with an unknown file name.
    pub fn new() -> Self {
        Self {
            file_name: UNKNOWN.to_string(),
            line: 0,
            byte: 0,
        }
    }

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

/// Convert a lexer token stream to AST nodes.
///
/// The `Parser` is an `Iterator` that yields `Node`:
///
/// ```ignore
/// let content = "A {{var}} template.";
/// let parser = Parser::new(content, Default::default());
/// for node in parser {
///     println!("{:#?}", node.unwrap());
/// }
/// ```
pub struct Parser<'source> {
    source: &'source str,
    lexer: Lexer<'source>,
    state: ParseState,
    stack: Vec<(&'source str, Block<'source>)>,
    next_token: Option<Token>,
}

impl<'source> Parser<'source> {
    /// Create a new Parser for the given source template.
    ///
    /// This will prepare a lexer and initial state for the iterator.
    pub fn new(source: &'source str, options: ParserOptions) -> Self {
        let lexer = lex(source);
        let state = ParseState::from(&options);
        Self {
            source,
            lexer,
            state,
            stack: vec![],
            next_token: None,
        }
    }

    /// Parse the entire document into a node tree.
    ///
    /// This iterates the parser until completion and adds
    /// each node to a `Document` node which is returned.
    pub fn parse(&mut self) -> SyntaxResult<Node<'source>> {
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
    fn advance(&mut self, next: Token) -> SyntaxResult<Option<Node<'source>>> {
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
            Token::Block(lex, mut span) => match lex {
                lexer::Block::StartRawBlock => {
                    return block::raw(
                        self.source,
                        &mut self.lexer,
                        &mut self.state,
                        span,
                    )
                    .map(Some);
                }
                lexer::Block::StartRawComment => {
                    return block::raw_comment(
                        self.source,
                        &mut self.lexer,
                        &mut self.state,
                        span,
                    )
                    .map(Some);
                }
                lexer::Block::StartRawStatement => {
                    return block::raw_statement(
                        self.source,
                        &mut self.lexer,
                        &mut self.state,
                        span,
                    )
                    .map(Some);
                }
                lexer::Block::StartComment => {
                    return block::comment(
                        self.source,
                        &mut self.lexer,
                        &mut self.state,
                        span,
                    )
                    .map(Some);
                }
                lexer::Block::StartBlockScope => {
                    let block = block::scope(
                        self.source,
                        &mut self.lexer,
                        &mut self.state,
                        span,
                    )?;

                    let name = block.name().ok_or_else(|| {
                        return SyntaxError::ExpectedIdentifier(
                            ErrorInfo::new(
                                self.source,
                                self.state.file_name(),
                                SourcePos::from((
                                    self.state.line(),
                                    self.state.byte(),
                                )),
                            )
                            .into(),
                        );
                    })?;

                    match block.call().target() {
                        CallTarget::Path(ref path) => {
                            if !path.is_simple() {
                                return Err(
                                    SyntaxError::ExpectedSimpleIdentifier(
                                        ErrorInfo::new(
                                            self.source,
                                            self.state.file_name(),
                                            SourcePos::from((
                                                self.state.line(),
                                                self.state.byte(),
                                            )),
                                        )
                                        .into(),
                                    ),
                                );
                            }
                        }
                        CallTarget::SubExpr(_) => {
                            if !block.call().is_partial() {
                                panic!("Sub expression block targets are only evaluated for partials");
                            }
                        }
                    }

                    self.stack.push((name, block));

                    while let Some(t) = self.token() {
                        match self.advance(t) {
                            Ok(mut node) => {
                                if node.is_none() || self.stack.is_empty() {
                                    return Ok(node);
                                } else {
                                    let (_, current) =
                                        self.stack.last_mut().unwrap();

                                    if let Some(node) = node.take() {
                                        match node {
                                            // NOTE: The push() implementation on Block
                                            // NOTE: will add to the last conditional if
                                            // NOTE: any conditions are present.
                                            Node::Statement(call) => {
                                                if call.is_conditional() {
                                                    let condition =
                                                        Condition::new(
                                                            self.source,
                                                            call,
                                                        );
                                                    current.add_condition(
                                                        condition,
                                                    );
                                                } else {
                                                    current.push(
                                                        Node::Statement(call),
                                                    );
                                                }
                                            }
                                            _ => {
                                                current.push(node);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => return Err(e),
                        }
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
                        let notes = if let Some(close) = temp.name() {
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
                                notes,
                            )
                            .into(),
                        ));
                        //panic!("Got close block with no open block!");
                    }

                    let (open_name, mut block) = self.stack.pop().unwrap();

                    if let Some(close_name) = temp.name() {
                        if open_name != close_name {
                            return Err(SyntaxError::TagNameMismatch(
                                ErrorInfo::new_notes(
                                    self.source,
                                    self.state.file_name(),
                                    SourcePos::from((
                                        self.state.line(),
                                        self.state.byte(),
                                    )),
                                    vec![format!(
                                        "opening name is '{}'",
                                        open_name
                                    )],
                                )
                                .into(),
                            ));
                        }

                        // Update span for entire close tag: `{{/name}}`!
                        if temp.call().is_closed() {
                            let end_tag_close = temp.call().span();
                            span.end = end_tag_close.end;
                        }
                        block.exit(span);

                        return Ok(Some(Node::Block(block)));
                    } else {
                        return Err(SyntaxError::ExpectedIdentifier(
                            ErrorInfo::new(
                                self.source,
                                self.state.file_name(),
                                SourcePos::from((
                                    self.state.line(),
                                    self.state.byte(),
                                )),
                            )
                            .into(),
                        ));
                    }
                }
                lexer::Block::StartStatement => {
                    let context = if self.stack.is_empty() {
                        CallParseContext::Statement
                    } else {
                        CallParseContext::ScopeStatement
                    };
                    let call = call::parse(
                        self.source,
                        &mut self.lexer,
                        &mut self.state,
                        span,
                        context,
                    )?;
                    return Ok(Some(Node::Statement(call)));
                }
                _ => {}
            },
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
    type Item = SyntaxResult<Node<'source>>;

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
