use bracket::lexer::{
    collect as lex, Array, Block, Comment, DoubleQuoteString, Parameters,
    RawComment, RawStatement, SingleQuoteString, Token, Link,
};

#[test]
fn lex_text_only() {
    let value = "foo bar baz";
    let tokens = lex(value, true);
    let expect = vec![Token::Block(Block::Text, 0..11)];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_double_quote_string() {
    let value = r#"{{"foo"}}"#;
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartStatement, 0..2),
        Token::Parameters(Parameters::DoubleQuoteString, 2..3),
        Token::DoubleQuoteString(DoubleQuoteString::Text, 3..6),
        Token::DoubleQuoteString(DoubleQuoteString::End, 6..7),
        Token::Parameters(Parameters::End, 7..9),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_single_quote_string() {
    let value = "{{'foo'}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartStatement, 0..2),
        Token::Parameters(Parameters::SingleQuoteString, 2..3),
        Token::SingleQuoteString(SingleQuoteString::Text, 3..6),
        Token::SingleQuoteString(SingleQuoteString::End, 6..7),
        Token::Parameters(Parameters::End, 7..9),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_array_string() {
    let value = "{{this.[1]}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartStatement, 0..2),
        Token::Parameters(Parameters::ExplicitThisKeyword, 2..6),
        Token::Parameters(Parameters::PathDelimiter, 6..7),
        Token::Parameters(Parameters::StartArray, 7..8),
        Token::Array(Array::Text, 8..9),
        Token::Array(Array::End, 9..10),
        Token::Parameters(Parameters::End, 10..12),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_block_text() {
    let value = "foo {{bar}} baz";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::Text, 0..4),
        Token::Block(Block::StartStatement, 4..6),
        Token::Parameters(Parameters::Identifier, 6..9),
        Token::Parameters(Parameters::End, 9..11),
        Token::Block(Block::Text, 11..15),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_raw_block_text() {
    let value = "{{{{ raw }}}}foo {{bar}} baz{{{{ / raw }}}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartRawBlock, 0..5),
        Token::Parameters(Parameters::Identifier, 5..8),
        Token::Parameters(Parameters::WhiteSpace, 8..9),
        Token::Parameters(Parameters::End, 9..13),
        Token::Block(Block::Text, 13..17),
        Token::Block(Block::StartStatement, 17..19),
        Token::Parameters(Parameters::Identifier, 19..22),
        Token::Parameters(Parameters::End, 22..24),
        Token::Block(Block::Text, 24..28),
        Token::Block(Block::EndRawBlock, 28..34),
        Token::Parameters(Parameters::WhiteSpace, 34..35),
        Token::Parameters(Parameters::Identifier, 35..38),
        Token::Parameters(Parameters::WhiteSpace, 38..39),
        Token::Parameters(Parameters::End, 39..43),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_raw_block_multiline() {
    let value = "{{{{raw}}}}
foo
{{bar}}
baz
{{{{/raw}}}}
";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartRawBlock, 0..4),
        Token::Parameters(Parameters::Identifier, 4..7),
        Token::Parameters(Parameters::End, 7..11),
        Token::Block(Block::Text, 11..16),
        Token::Block(Block::StartStatement, 16..18),
        Token::Parameters(Parameters::Identifier, 18..21),
        Token::Parameters(Parameters::End, 21..23),
        Token::Block(Block::Text, 23..28),
        Token::Block(Block::EndRawBlock, 28..33),
        Token::Parameters(Parameters::Identifier, 33..36),
        Token::Parameters(Parameters::End, 36..40),
        Token::Block(Block::Text, 40..41),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_raw_comment() {
    let value = "{{!-- foo {{bar}} baz --}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartRawComment, 0..5),
        Token::Block(Block::Text, 5..22),
        Token::RawComment(RawComment::End, 22..26),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_raw_comment_multiline() {
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
}

#[test]
fn lex_comment() {
    let value = "{{! foo }}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartComment, 0..3),
        Token::Block(Block::Text, 3..8),
        Token::Comment(Comment::End, 8..10),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_comment_multiline() {
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
}

#[test]
fn lex_raw_statement() {
    let value = "\\{{foo}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartRawStatement, 0..3),
        Token::Block(Block::Text, 3..6),
        Token::RawStatement(RawStatement::End, 6..8),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_raw_statement_partial() {
    let value = "\\{{> foo}}";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartRawStatement, 0..3),
        Token::Block(Block::Text, 3..8),
        Token::RawStatement(RawStatement::End, 8..10),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_statement_identifier() {
    let value = "{{foo}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartStatement, 0..2),
        Token::Parameters(Parameters::Identifier, 2..5),
        Token::Parameters(Parameters::End, 5..7),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_statement_partial() {
    let value = "{{> foo}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartStatement, 0..2),
        Token::Parameters(Parameters::Partial, 2..3),
        Token::Parameters(Parameters::WhiteSpace, 3..4),
        Token::Parameters(Parameters::Identifier, 4..7),
        Token::Parameters(Parameters::End, 7..9),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_statement_path() {
    let value = "{{foo.bar.baz}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartStatement, 0..2),
        Token::Parameters(Parameters::Identifier, 2..5),
        Token::Parameters(Parameters::PathDelimiter, 5..6),
        Token::Parameters(Parameters::Identifier, 6..9),
        Token::Parameters(Parameters::PathDelimiter, 9..10),
        Token::Parameters(Parameters::Identifier, 10..13),
        Token::Parameters(Parameters::End, 13..15),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_statement_parent_path() {
    let value = "{{../../foo}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartStatement, 0..2),
        Token::Parameters(Parameters::ParentRef, 2..5),
        Token::Parameters(Parameters::ParentRef, 5..8),
        Token::Parameters(Parameters::Identifier, 8..11),
        Token::Parameters(Parameters::End, 11..13),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_statement_root_path() {
    let value = "{{@root/foo}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartStatement, 0..2),
        Token::Parameters(Parameters::LocalIdentifier, 2..7),
        Token::Parameters(Parameters::PathDelimiter, 7..8),
        Token::Parameters(Parameters::Identifier, 8..11),
        Token::Parameters(Parameters::End, 11..13),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_statement_sub_expr() {
    let value = "{{foo (lookup a b)}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartStatement, 0..2),
        Token::Parameters(Parameters::Identifier, 2..5),
        Token::Parameters(Parameters::WhiteSpace, 5..6),
        Token::Parameters(Parameters::StartSubExpression, 6..7),
        Token::Parameters(Parameters::Identifier, 7..13),
        Token::Parameters(Parameters::WhiteSpace, 13..14),
        Token::Parameters(Parameters::Identifier, 14..15),
        Token::Parameters(Parameters::WhiteSpace, 15..16),
        Token::Parameters(Parameters::Identifier, 16..17),
        Token::Parameters(Parameters::EndSubExpression, 17..18),
        Token::Parameters(Parameters::End, 18..20),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_block_scope() {
    let value = "{{#foo}}bar {{baz}} qux{{/foo}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartBlockScope, 0..3),
        Token::Parameters(Parameters::Identifier, 3..6),
        Token::Parameters(Parameters::End, 6..8),
        Token::Block(Block::Text, 8..12),
        Token::Block(Block::StartStatement, 12..14),
        Token::Parameters(Parameters::Identifier, 14..17),
        Token::Parameters(Parameters::End, 17..19),
        Token::Block(Block::Text, 19..23),
        Token::Block(Block::EndBlockScope, 23..26),
        Token::Parameters(Parameters::Identifier, 26..29),
        Token::Parameters(Parameters::End, 29..31),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_block_scope_partial() {
    let value = "{{#>foo}}{{@partial-block}}{{/foo}}";
    let tokens = lex(value, true);

    let expect = vec![
        Token::Block(Block::StartBlockScope, 0..3),
        Token::Parameters(Parameters::Partial, 3..4),
        Token::Parameters(Parameters::Identifier, 4..7),
        Token::Parameters(Parameters::End, 7..9),
        Token::Block(Block::StartStatement, 9..11),
        Token::Parameters(Parameters::LocalIdentifier, 11..25),
        Token::Parameters(Parameters::End, 25..27),
        Token::Block(Block::EndBlockScope, 27..30),
        Token::Parameters(Parameters::Identifier, 30..33),
        Token::Parameters(Parameters::End, 33..35),
    ];
    assert_eq!(expect, tokens);
}

#[test]
fn lex_link() {
    let value = "[[/foo/bar|label]]";
    let tokens = lex(value, true);
    let expect = vec![
        Token::Block(Block::StartLink, 0..2),
        Token::Link(Link::Text, 2..10),
        Token::Link(Link::Pipe, 10..11),
        Token::Link(Link::Text, 11..16),
        Token::Link(Link::End, 16..18),
    ];
    assert_eq!(expect, tokens);
}

