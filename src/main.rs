use std::fmt;
use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
#[logos(subpattern path = r"[a-zA-Z0-9._-]+")]
enum Token {

    #[regex(r"[\\]?\{\{\{?>?\s?\w+\s?\}?\}\}", |lex| lex.slice().to_string())]
    Expression(String),

    #[token("{{{{raw}}}}.*{{{{/raw}}}}", |lex| lex.slice().to_string())]
    RawBlock(String),

    #[regex(r"\r?\n", |lex| lex.slice().to_string())]
    Newline(String),

    #[regex(r"\{\{#>?\s?\w+\s?\}\}", |lex| lex.slice().to_string())]
    Block(String),

    #[regex(r"\{\{/\s?\w+\s?\}\}", |lex| lex.slice().to_string())]
    EndBlock(String),

    #[regex(".*", |lex| lex.slice().to_string())]
    Text(String),

    #[error]
    Error,
}

#[derive(Debug)]
struct SourceInfo {
    line: usize, 
    span: logos::Span,
}

#[derive(Debug)]
struct Expression {
    info: SourceInfo,
    value: String,
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.value)
    }
}

#[derive(Debug)]
struct Text {
    info: SourceInfo,
    value: String,
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.value)
    }
}

#[derive(Debug)]
enum AstToken {
    Expression(Expression),
    Text(Text),
    RawBlock(Text),
    Newline(Text),
}

impl fmt::Display for AstToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Expression(ref t) => t.fmt(f),
            Self::Text(ref t) | Self::RawBlock(ref t) | Self::Newline(ref t) => t.fmt(f),
        }
    }
}

#[derive(Debug)]
enum BlockType {
    Root,
    Named(String),
}

#[derive(Debug)]
struct Block {
    block_type: BlockType, 
    tokens: Vec<AstToken>,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self {block_type, tokens: Vec::new()}
    }

    pub fn push(&mut self, token: AstToken) {
        self.tokens.push(token); 
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in self.tokens.iter() {
            t.fmt(f)?;
        }
        Ok(())
    }
}

struct Template {
    block: Block,
}

#[derive(Debug)]
impl Template {
    pub fn compile(s: &str) -> Result<()> {
        Ok(()) 
    }
}

fn main() {
    let lex = Token::lexer(r"\{{expr}}
{{{unescaped}}}

{{var}}

{{{{raw}}}}
This is some raw text.
{{{{/raw}}}}

{{# block}}
This is some block text with an {{inline}}
{{/block}}

{{> partial}}
");

    let mut ast = Block::new(BlockType::Root);

    let mut line = 0;
    for (token, span) in lex.spanned().into_iter() {
        println!("Line number {}", line);
        let info = SourceInfo {line, span};
        match token {
            Token::Expression(value) => {
                ast.push(AstToken::Expression(Expression {info, value}));
            }
            Token::Text(value) => {
                ast.push(AstToken::Text(Text {info, value}));
            }
            Token::RawBlock(value) => {
                ast.push(AstToken::RawBlock(Text {info, value}));
            }
            Token::Newline(value) => {
                ast.push(AstToken::Newline(Text {info, value}));
                line = line + 1; 
            }
            _ => {
                println!("{:?}", token);
            }
        }
        //prev = Some(token);
    }

    println!("{:#?}", ast);
    println!("{}", ast.to_string());
}
