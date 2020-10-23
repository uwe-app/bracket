use hbs::Result;

#[test]
fn raw_block() -> Result<()> {
    use hbs::lexer::grammar::raw_block;
    use hbs::lexer::grammar::block::{self, BlockToken};

    let value = "{{{{ raw }}}}foo {{bar}} baz{{{{ / raw }}}}";
    let tokens = block::lex(value);
    let expect = vec![
        BlockToken::Block(block::Outer::StartRawBlock, 0..13),
        BlockToken::Block(block::Outer::Text, 13..28),
        BlockToken::RawBlock(raw_block::Inner::End, 28..43),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

/*
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
*/
