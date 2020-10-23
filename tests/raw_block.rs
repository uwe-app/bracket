use hbs::Result;

#[test]
fn raw_block() -> Result<()> {
    use hbs::lexer::grammar::raw_block::{self, Inner::*, Outer::*};
    use hbs::lexer::grammar::modes::Tokens::*;

    let value = "{{{{ raw }}}}foo {{bar}} baz{{{{ / raw }}}}";
    let tokens = raw_block::lex(value);

    let expect = vec![
        (OuterToken(Start), 0..13),
        (InnerToken(Text), 13..28),
        (InnerToken(End), 28..43),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn raw_block_multiline() -> Result<()> {
    use hbs::lexer::grammar::raw_block::{self, Inner::*, Outer::*};
    use hbs::lexer::grammar::modes::Tokens::*;

    let value = "{{{{raw}}}}
foo
{{bar}}
baz
{{{{/raw}}}}";
    let tokens = raw_block::lex(value);

    //println!("{:?}", tokens);

    let expect = vec![
        (OuterToken(Start), 0..11),
        (InnerToken(Text), 11..28),
        (InnerToken(End), 28..40),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}
