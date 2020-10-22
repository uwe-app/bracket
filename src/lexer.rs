use std::ops::Range;

pub mod parser {
    use regex::Regex;

    pub fn block_name(value: &str) -> String {
        let re = Regex::new(r"\{\{\{?#?>?/?\s*([^}]*)\s*\}?\}\}").unwrap();
        let cap = re.captures_iter(value).next().unwrap();
        cap[1].to_string()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceInfo {
    pub line: Range<usize>,
    pub span: logos::Span,
}

pub mod grammar {
    use logos::Logos;

    #[derive(Logos, Debug, PartialEq)]
    // SEE: https://handlebarsjs.com/guide/expressions.html#literal-segments
    #[logos(subpattern identifier = r#"[^\s"!#%&'()*+,./;<=>@\[/\]^`{|}~]"#)]
    pub(crate) enum Statement<'source> {

        #[regex(r"\{\{\{?>?\s*", |lex| lex.slice())]
        Open(&'source str),

        #[regex(r"((?&identifier)[.])*(?&identifier)+", priority = 2, callback = |lex| lex.slice())]
        Path(&'source str),

        #[regex(r"-?[0-9]*\.?[0-9]+", |lex| lex.slice())]
        Number(&'source str),

        #[regex(r"(true|false)", |lex| lex.slice())]
        Bool(&'source str),

        #[token("null", |lex| lex.slice())]
        Null(&'source str),

        #[regex(r"\s*\}?\}\}", |lex| lex.slice())]
        Close(&'source str),

        #[regex(r"\s+", |lex| lex.slice())]
        WhiteSpace(&'source str),

        #[error]
        Error,
    }

    #[derive(Logos, Debug, PartialEq)]
    #[logos(subpattern simple_name = r"[a-zA-Z0-9_-]+")]
    pub(crate) enum Token<'source> {
        #[regex(r"[\\]?\{\{\{?[^!]>?\s*[^}]+\s*\}?\}\}", |lex| lex.slice())]
        Expression(&'source str),

        #[regex(r"\{\{\{\{\s*raw\s*\}\}\}\}", |lex| lex.slice())]
        StartRawBlock(&'source str),

        #[regex(r"\{\{\{\{\s*/raw\s*\}\}\}\}", |lex| lex.slice())]
        EndRawBlock(&'source str),

        #[regex(r"\r?\n", |lex| lex.slice())]
        Newline(&'source str),

        #[regex(r"\{\{#>?\s*(?&simple_name)\s*\}\}", |lex| lex.slice())]
        StartBlock(&'source str),

        #[regex(r"\{\{/\s*(?&simple_name)\s*\}\}", |lex| lex.slice())]
        EndBlock(&'source str),

        #[regex(r"(\{\{!(--)?|<!--)", |lex| lex.slice())]
        StartCommentBlock(&'source str),

        #[regex(r"((--)?\}\}|-->)", |lex| lex.slice())]
        EndCommentBlock(&'source str),

        #[regex(r"[^\n{]", |lex| lex.slice())]
        Char(&'source str),

        #[error]
        Error,
    }
}

pub mod ast {

    #[derive(Debug, Eq, PartialEq, Default)]
    pub struct Statement<'source> {
        tokens: Vec<Token<'source>>,
    }

    impl Statement<'_> {
        pub fn tokens(&self) -> &Vec<Token> {
            &self.tokens 
        } 
    }

    use std::fmt;
    use super::{SourceInfo, parser};

    #[derive(Debug, Eq, PartialEq)]
    pub struct Expr<'source> {
        info: SourceInfo,
        value: &'source str,
    }

    impl<'source> Expr<'source> {

        pub fn new(info: SourceInfo, value: &'source str) -> Self {
            Self {info, value} 
        }

        pub fn is_raw(&self) -> bool {
            if !self.value.is_empty() {
                let first = self.value.chars().nth(0).unwrap();
                return first == '\\';
            }
            false
        }

        pub fn escapes(&self) -> bool {
            !self.value.starts_with("{{{") 
        }

        pub fn value(&self) -> &str {
            &self.value
        }

        pub fn info(&self) -> &SourceInfo {
            &self.info
        }
    }

    impl fmt::Display for Expr<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value)
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct Text<'source> {
        pub info: SourceInfo,
        pub value: &'source str,
    }

    impl fmt::Display for Text<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value)
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum Token<'source> {
        Expression(Expr<'source>),
        Text(Text<'source>),
        Block(Block<'source>),
        //Newline(Text),
    }

    impl fmt::Display for Token<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                Self::Expression(ref t) => t.fmt(f),
                Self::Block(ref t) => t.fmt(f),
                Self::Text(ref t) => t.fmt(f),
            }
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum BlockType {
        Root,
        Raw,
        Comment,
        // TODO: use &'source ref
        Named(String),
    }

    impl Default for BlockType {
        fn default() -> Self {
            Self::Root
        }
    }

    #[derive(Debug, Default, Eq, PartialEq)]
    pub struct Block<'source> {
        pub(crate) block_type: BlockType,
        tokens: Vec<Token<'source>>,
        pub(crate) open: Option<&'source str>,
        pub(crate) close: Option<&'source str>,
    }

    impl<'source> Block<'source> {
        pub fn new(block_type: BlockType) -> Self {
            Self {
                block_type,
                tokens: Vec::new(),
                open: None,
                close: None,
            }
        }

        pub fn new_named(value: &'source str) -> Self {
            let name = parser::block_name(&value);
            let mut block = Block::new(BlockType::Named(name));
            block.open = Some(value);
            block
        }

        pub fn push(&mut self, token: Token<'source>) {
            self.tokens.push(token);
        }

        pub fn tokens(&self) -> &'source Vec<Token> {
            &self.tokens 
        }

        pub fn tokens_mut(&mut self) -> &'source mut Vec<Token> {
            &mut self.tokens 
        }

        pub fn is_raw(&self) -> bool {
            match self.block_type {
                BlockType::Raw => true,
                _ => false,
            }
        }

        pub fn is_named(&self) -> bool {
            match self.block_type {
                BlockType::Named(_) => true,
                _ => false,
            }
        }
    }

    impl fmt::Display for Block<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if let Some(ref s) = self.open {
                write!(f, "{}", s)?;
            }
            for t in self.tokens.iter() {
                t.fmt(f)?;
            }
            if let Some(ref s) = self.close {
                write!(f, "{}", s)?;
            }
            Ok(())
        }
    }

}
