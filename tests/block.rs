use hbs::Result;

#[test]
fn block_text() -> Result<()> {
    use hbs::lexer::grammar::*;

    let value = "Some text {{foo}}";
    let tokens = lex(value, true);

    println!("{:#?}", tokens);

    let expect = vec![
        BlockToken::Block(Block::Text, 0..10),
        BlockToken::Block(Block::StartStatement, 10..12),
        BlockToken::Statement(Statement::Identifier, 12..15),
        BlockToken::Statement(Statement::End, 15..17),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

