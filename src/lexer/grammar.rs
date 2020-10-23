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

/// Parses a handlebars statement into tokens.
pub mod statement {
    use logos::Lexer;
    use logos::Logos;

    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
    pub enum Outer<'source> {
        #[regex(r"\{\{\{?", |lex| lex.slice())]
        Start(&'source str),

        #[error]
        Error,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
    #[logos(subpattern identifier = r#"[^\s"!#%&'()*+,./;<=>@\[/\]^`{|}~]"#)]
    pub enum Inner<'source> {

        #[token(r">", |lex| lex.slice())]
        Partial(&'source str),

        #[regex(r"(?&identifier)+", priority = 2, callback = |lex| lex.slice())]
        Identifier(&'source str),

        #[regex(r"[./]", |lex| lex.slice())]
        PathDelimiter(&'source str),

        #[regex(r"-?[0-9]*\.?[0-9]+", |lex| lex.slice())]
        Number(&'source str),

        #[regex(r"(true|false)", |lex| lex.slice())]
        Bool(&'source str),

        #[token("null", |lex| lex.slice())]
        Null(&'source str),

        #[regex(r"\s+", |lex| lex.slice())]
        WhiteSpace(&'source str),

        #[regex(r"\}?\}\}", |lex| lex.slice())]
        End(&'source str),

        #[error]
        Error,
    }

    enum Modes<'source> {
        Outer(Lexer<'source, Outer<'source>>),
        Inner(Lexer<'source, Inner<'source>>),
    }

    impl<'source> Modes<'source> {
        fn new(s: &'source str) -> Self {
            Self::Outer(Outer::lexer(s))
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum Tokens<'source> {
        InnerToken(Inner<'source>),
        OuterToken(Outer<'source>),
    }

    struct ModeBridge<'source> {
        mode: Modes<'source>,
    }

    // Clones as we switch between modes
    impl<'source> Iterator for ModeBridge<'source> {
        type Item = Tokens<'source>;
        fn next(&mut self) -> Option<Self::Item> {
            use Tokens::*;
            match &mut self.mode {
                Modes::Inner(inner) => {
                    let result = inner.next();
                    if Some(Inner::End("}}")) == result || Some(Inner::End("}}}")) == result {
                        self.mode = Modes::Outer(inner.to_owned().morph());
                    }
                    result.map(InnerToken)
                }
                Modes::Outer(outer) => {
                    let result = outer.next();
                    if Some(Outer::Start("{{")) == result || Some(Outer::Start("{{{")) == result {
                        self.mode = Modes::Inner(outer.to_owned().morph());
                    }
                    result.map(OuterToken)
                }
            }
        }
    }

    pub fn lex(s: &str) -> Vec<Tokens> {
        let moded = ModeBridge {
            mode: Modes::new(s),
        };
        let results: Vec<Tokens> = moded.collect();
        results
    }
}

// Parses a double-quoted JSON-style string into tokens.
pub mod string {
    use logos::Lexer;
    use logos::Logos;

    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
    pub enum Outer<'source> {
        #[token("\"", |lex| lex.slice())]
        Start(&'source str),

        #[error]
        Error,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
    pub enum Inner<'source> {

        #[regex(r#"[^\\"]+"#, |lex| lex.slice())]
        Text(&'source str),

        #[token("\\n", |lex| lex.slice())]
        EscapedNewline(&'source str),

        //#[regex(r"\\u\{[^}]*\}")]
        //EscapedCodepoint,

        #[token(r#"\""#, |lex| lex.slice())]
        EscapedQuote(&'source str),

        #[token("\"", |lex| lex.slice())]
        End(&'source str),

        #[error]
        Error,
    }

    enum Modes<'source> {
        Outer(Lexer<'source, Outer<'source>>),
        Inner(Lexer<'source, Inner<'source>>),
    }

    impl<'source> Modes<'source> {
        fn new(s: &'source str) -> Self {
            Self::Outer(Outer::lexer(s))
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum Tokens<'source> {
        InnerToken(Inner<'source>),
        OuterToken(Outer<'source>),
    }

    struct ModeBridge<'source> {
        mode: Modes<'source>,
    }

    // Clones as we switch between modes
    impl<'source> Iterator for ModeBridge<'source> {
        type Item = Tokens<'source>;
        fn next(&mut self) -> Option<Self::Item> {
            use Tokens::*;
            match &mut self.mode {
                Modes::Inner(inner) => {
                    let result = inner.next();
                    if Some(Inner::End(r#"""#)) == result {
                        self.mode = Modes::Outer(inner.to_owned().morph());
                    }
                    result.map(InnerToken)
                }
                Modes::Outer(outer) => {
                    let result = outer.next();
                    if Some(Outer::Start(r#"""#)) == result {
                        self.mode = Modes::Inner(outer.to_owned().morph());
                    }
                    result.map(OuterToken)
                }
            }
        }
    }

    pub fn lex(s: &str) -> Vec<Tokens> {
        let moded = ModeBridge {
            mode: Modes::new(s),
        };
        let results: Vec<Tokens> = moded.collect();
        results
    }
}
