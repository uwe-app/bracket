use hbs::lexer::grammar::*;
use hbs::Result;

#[test]
fn lex_text_only() -> Result<()> {
    let value = "foo bar baz";
    let tokens = lex(value, true);

    let expect = vec![Token::Block(Block::Text, 0..11)];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_block_text() -> Result<()> {
    let value = "foo {{bar}} baz";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::Text, 0..4),
        Token::Block(Block::StartStatement, 4..6),
        Token::Statement(Statement::Identifier, 6..9),
        Token::Statement(Statement::End, 9..11),
        Token::Block(Block::Text, 11..15),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_raw_block() -> Result<()> {
    let value = "{{{{ raw }}}}foo {{bar}} baz{{{{ / raw }}}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartRawBlock, 0..13),
        Token::Block(Block::Text, 13..28),
        Token::RawBlock(RawBlock::End, 28..43),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_raw_block_multiline() -> Result<()> {
    let value = "{{{{raw}}}}
foo
{{bar}}
baz
{{{{/raw}}}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartRawBlock, 0..11),
        Token::Block(Block::Text, 11..28),
        Token::RawBlock(RawBlock::End, 28..40),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_raw_comment() -> Result<()> {
    let value = "{{!-- foo {{bar}} baz --}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartRawComment, 0..5),
        Token::Block(Block::Text, 5..22),
        Token::RawComment(RawComment::End, 22..26),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_raw_comment_multiline() -> Result<()> {
    let value = "{{!--
foo
{{bar}}
baz
--}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartRawComment, 0..5),
        Token::Block(Block::Text, 5..22),
        Token::RawComment(RawComment::End, 22..26),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_comment() -> Result<()> {
    let value = "{{! foo }}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartComment, 0..3),
        Token::Block(Block::Text, 3..8),
        Token::Comment(Comment::End, 8..10),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_comment_multiline() -> Result<()> {
    let value = "{{!
foo
bar
baz
}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartComment, 0..3),
        Token::Block(Block::Text, 3..16),
        Token::Comment(Comment::End, 16..18),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_raw_statement() -> Result<()> {
    let value = "\\{{foo}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartRawStatement, 0..3),
        Token::Block(Block::Text, 3..6),
        Token::RawStatement(RawStatement::End, 6..8),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_raw_statement_partial() -> Result<()> {
    let value = "\\{{> foo}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartRawStatement, 0..3),
        Token::Block(Block::Text, 3..8),
        Token::RawStatement(RawStatement::End, 8..10),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_statement_identifier() -> Result<()> {
    let value = "{{foo}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartStatement, 0..2),
        Token::Statement(Statement::Identifier, 2..5),
        Token::Statement(Statement::End, 5..7),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_statement_partial() -> Result<()> {
    let value = "{{> foo}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartStatement, 0..2),
        Token::Statement(Statement::Partial, 2..3),
        Token::Statement(Statement::WhiteSpace, 3..4),
        Token::Statement(Statement::Identifier, 4..7),
        Token::Statement(Statement::End, 7..9),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_statement_path() -> Result<()> {
    let value = "{{foo.bar.baz}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartStatement, 0..2),
        Token::Statement(Statement::Identifier, 2..5),
        Token::Statement(Statement::PathDelimiter, 5..6),
        Token::Statement(Statement::Identifier, 6..9),
        Token::Statement(Statement::PathDelimiter, 9..10),
        Token::Statement(Statement::Identifier, 10..13),
        Token::Statement(Statement::End, 13..15),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}
