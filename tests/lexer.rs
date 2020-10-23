use hbs::Result;

#[test]
fn lex_text_only() -> Result<()> {
    use hbs::lexer::grammar::*;

    let value = "Some text";
    let tokens = lex(value, true);

    let expect = vec![
        BlockToken::Block(Block::Text, 0..9),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_block_text() -> Result<()> {
    use hbs::lexer::grammar::*;

    let value = "Some text {{foo}}";
    let tokens = lex(value, true);

    let expect = vec![
        BlockToken::Block(Block::Text, 0..10),
        BlockToken::Block(Block::StartStatement, 10..12),
        BlockToken::Statement(Statement::Identifier, 12..15),
        BlockToken::Statement(Statement::End, 15..17),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_raw_block() -> Result<()> {
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
fn lex_raw_block_multiline() -> Result<()> {
    use hbs::lexer::grammar::*;

    let value = "{{{{raw}}}}
foo
{{bar}}
baz
{{{{/raw}}}}";
    let tokens = lex(value, true);

    let expect = vec![
        BlockToken::Block(Block::StartRawBlock, 0..11),
        BlockToken::Block(Block::Text, 11..28),
        BlockToken::RawBlock(RawBlock::End, 28..40),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_raw_comment() -> Result<()> {
    use hbs::lexer::grammar::*;
    let value = "{{!-- foo {{bar}} baz --}}";
    let tokens = lex(value, true);
    let expect = vec![
        BlockToken::Block(Block::StartRawComment, 0..5),
        BlockToken::Block(Block::Text, 5..22),
        BlockToken::RawComment(RawComment::End, 22..26),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_raw_comment_multiline() -> Result<()> {
    use hbs::lexer::grammar::*;
    let value = "{{!--
foo
{{bar}}
baz
--}}";
    let tokens = lex(value, true);
    let expect = vec![
        BlockToken::Block(Block::StartRawComment, 0..5),
        BlockToken::Block(Block::Text, 5..22),
        BlockToken::RawComment(RawComment::End, 22..26),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}
