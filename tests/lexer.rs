use hbs::lexer::grammar::*;
use hbs::Result;

#[test]
fn lex_text_only() -> Result<()> {
    let value = "foo bar baz";
    let tokens = lex(value, true);

    let expect = vec![Token::Block(Block::Text, 0..11, 0..0)];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_block_text() -> Result<()> {
    let value = "foo {{bar}} baz";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::Text, 0..4, 0..0),
        Token::Block(Block::StartStatement, 4..6, 0..0),
        Token::Statement(Statement::Identifier, 6..9, 0..0),
        Token::Statement(Statement::End, 9..11, 0..0),
        Token::Block(Block::Text, 11..15, 0..0),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_raw_block() -> Result<()> {
    let value = "{{{{ raw }}}}foo {{bar}} baz{{{{ / raw }}}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartRawBlock, 0..13, 0..0),
        Token::Block(Block::Text, 13..28, 0..0),
        Token::RawBlock(RawBlock::End, 28..43, 0..0),
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
{{{{/raw}}}}
";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartRawBlock, 0..11, 0..0),
        Token::Block(Block::Text, 11..28, 1..4),
        Token::RawBlock(RawBlock::End, 28..40, 4..4),
        Token::Block(Block::Text, 40..41, 5..5),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_raw_comment() -> Result<()> {
    let value = "{{!-- foo {{bar}} baz --}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartRawComment, 0..5, 0..0),
        Token::Block(Block::Text, 5..22, 0..0),
        Token::RawComment(RawComment::End, 22..26, 0..0),
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
        Token::Block(Block::StartRawComment, 0..5, 0..0),
        Token::Block(Block::Text, 5..22, 1..4),
        Token::RawComment(RawComment::End, 22..26, 4..4),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_comment() -> Result<()> {
    let value = "{{! foo }}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartComment, 0..3, 0..0),
        Token::Block(Block::Text, 3..8, 0..0),
        Token::Comment(Comment::End, 8..10, 0..0),
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
        Token::Block(Block::StartComment, 0..3, 0..0),
        Token::Block(Block::Text, 3..16, 1..4),
        Token::Comment(Comment::End, 16..18, 4..4),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_raw_statement() -> Result<()> {
    let value = "\\{{foo}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartRawStatement, 0..3, 0..0),
        Token::Block(Block::Text, 3..6, 0..0),
        Token::RawStatement(RawStatement::End, 6..8, 0..0),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_raw_statement_partial() -> Result<()> {
    let value = "\\{{> foo}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartRawStatement, 0..3, 0..0),
        Token::Block(Block::Text, 3..8, 0..0),
        Token::RawStatement(RawStatement::End, 8..10, 0..0),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_statement_identifier() -> Result<()> {
    let value = "{{foo}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartStatement, 0..2, 0..0),
        Token::Statement(Statement::Identifier, 2..5, 0..0),
        Token::Statement(Statement::End, 5..7, 0..0),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_statement_partial() -> Result<()> {
    let value = "{{> foo}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartStatement, 0..2, 0..0),
        Token::Statement(Statement::Partial, 2..3, 0..0),
        Token::Statement(Statement::WhiteSpace, 3..4, 0..0),
        Token::Statement(Statement::Identifier, 4..7, 0..0),
        Token::Statement(Statement::End, 7..9, 0..0),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}

#[test]
fn lex_statement_path() -> Result<()> {
    let value = "{{foo.bar.baz}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartStatement, 0..2, 0..0),
        Token::Statement(Statement::Identifier, 2..5, 0..0),
        Token::Statement(Statement::PathDelimiter, 5..6, 0..0),
        Token::Statement(Statement::Identifier, 6..9, 0..0),
        Token::Statement(Statement::PathDelimiter, 9..10, 0..0),
        Token::Statement(Statement::Identifier, 10..13, 0..0),
        Token::Statement(Statement::End, 13..15, 0..0),
    ];
    assert_eq!(expect, tokens);

    Ok(())
}
