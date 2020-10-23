use hbs::Result;

#[test]
fn block_text() -> Result<()> {
    use hbs::lexer::grammar::block::{self, Outer::*};
    use hbs::lexer::grammar::modes::Tokens::*;

    let value = "Some text {{foo}}";
    let tokens = block::lex(value);

    println!("{:#?}", tokens);

    //let expect = vec![
        //(OuterToken(Start), 0..13),
        //(InnerToken(Text), 13..28),
        //(InnerToken(End), 28..43),
    //];
    //assert_eq!(expect, tokens);

    Ok(())
}

