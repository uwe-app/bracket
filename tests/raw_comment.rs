use hbs::Result;

#[test]
fn raw_comment() -> Result<()> {
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
fn raw_comment_multiline() -> Result<()> {
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
