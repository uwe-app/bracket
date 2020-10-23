use hbs::Result;

#[test]
fn raw_comment() -> Result<()> {
    use hbs::lexer::grammar::raw_comment::{self, Inner::*, Outer::*};
    use hbs::lexer::grammar::modes::Tokens::*;

    let value = "{{!-- foo {{bar}} baz --}}";
    let tokens = raw_comment::lex(value);

    let expect = vec![
        (OuterToken(Start), 0..5),
        (InnerToken(Text), 5..22),
        (InnerToken(End), 22..26),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn raw_comment_multiline() -> Result<()> {
    use hbs::lexer::grammar::raw_comment::{self, Inner::*, Outer::*};
    use hbs::lexer::grammar::modes::Tokens::*;

    let value = "{{!--
foo
{{bar}}
baz
--}}";
    let tokens = raw_comment::lex(value);

    let expect = vec![
        (OuterToken(Start), 0..5),
        (InnerToken(Text), 5..22),
        (InnerToken(End), 22..26),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}
