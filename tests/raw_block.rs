use hbs::Result;

#[test]
fn raw_block() -> Result<()> {
    use hbs::lexer::grammar::*;

    let value = "{{{{ raw }}}}foo {{bar}} baz{{{{ / raw }}}}";
    let tokens = lex(value, true);
    let expect = vec![
        BlockToken::Block(Block::StartRawBlock, 0..13),
        BlockToken::Block(Block::Text, 13..28),
        BlockToken::RawBlock(RawBlock::End, 28..43),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}


#[test]
fn raw_block_multiline() -> Result<()> {
    use hbs::lexer::grammar::*;

    let value = "{{{{raw}}}}
foo
{{bar}}
baz
{{{{/raw}}}}";
    let tokens = lex(value, true);

    //println!("{:?}", tokens);

    let expect = vec![
        BlockToken::Block(Block::StartRawBlock, 0..11),
        BlockToken::Block(Block::Text, 11..28),
        BlockToken::RawBlock(RawBlock::End, 28..40),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

