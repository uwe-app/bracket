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

pub mod modes;
pub mod raw_block;
pub mod raw_comment;
pub mod statement;
pub mod string;
