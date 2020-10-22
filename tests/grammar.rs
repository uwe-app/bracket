use std::ops::Range;

use hbs::Template;
use hbs::{lexer::{SourceInfo, ast::{self, *}}, Result};

fn assert_text(token: &ast::Token, value: &str, line: Range<usize>, span: Range<usize>) {
    let info = SourceInfo { line, span };
    let expected = ast::Token::Text(Text {
        info,
        value,
    });
    assert_eq!(&expected, token);
}

#[test]
fn text() -> Result<()> {
    let value = r"Some text";
    let tpl = Template::compile(value)?;
    let token = tpl.block().tokens().get(0).unwrap();
    let info = SourceInfo {
        line: 0..0,
        span: 0..9,
    };
    let expected = ast::Token::Text(Text {
        info,
        value: value,
    });

    assert_eq!(1, tpl.block().tokens().len());
    assert_eq!(&expected, token);

    Ok(())
}

#[test]
fn mixed() -> Result<()> {
    let value = r"Some {{var}} text";
    let tpl = Template::compile(value)?;

    assert_eq!(3, tpl.block().tokens().len());

    assert_text(tpl.block().tokens().get(0).unwrap(), "Some ", 0..0, 0..5);

    let info = SourceInfo {
        line: 0..0,
        span: 5..12,
    };
    let expected = ast::Token::Expression(Expr::new(
        info,
        "{{var}}"
    ));
    assert_eq!(&expected, tpl.block().tokens().get(1).unwrap());

    assert_text(tpl.block().tokens().get(2).unwrap(), " text", 0..0, 12..17);

    Ok(())
}

#[test]
fn escaped_expr() -> Result<()> {
    let value = r"\{{expr}}";
    let tpl = Template::compile(value)?;
    let token = tpl.block().tokens().get(0).unwrap();
    let info = SourceInfo {
        line: 0..0,
        span: 0..9,
    };
    let expected = ast::Token::Expression(Expr::new(
        info,
        value,
    ));

    assert_eq!(1, tpl.block().tokens().len());
    assert_eq!(
        true,
        match token {
            ast::Token::Expression(_) => true,
            _ => false,
        }
    );

    assert_eq!(&expected, token);

    assert_eq!(
        true,
        match token {
            ast::Token::Expression(ref expr) => expr.is_raw(),
            _ => false,
        }
    );

    Ok(())
}

#[test]
fn simple_expr() -> Result<()> {
    let value = r"{{var}}";
    let tpl = Template::compile(value)?;
    let token = tpl.block().tokens().get(0).unwrap();
    let info = SourceInfo {
        line: 0..0,
        span: 0..7,
    };
    let expected = ast::Token::Expression(Expr::new(
        info,
        value,
    ));

    assert_eq!(1, tpl.block().tokens().len());
    assert_eq!(
        true,
        match token {
            ast::Token::Expression(_) => true,
            _ => false,
        }
    );

    assert_eq!(&expected, token);

    assert_eq!(
        false,
        match token {
            ast::Token::Expression(ref expr) => expr.is_raw(),
            _ => false,
        }
    );

    Ok(())
}

#[test]
fn unescaped_expr() -> Result<()> {
    let value = r"{{{var}}}";
    let tpl = Template::compile(value)?;
    let token = tpl.block().tokens().get(0).unwrap();
    let info = SourceInfo {
        line: 0..0,
        span: 0..9,
    };
    let expected = ast::Token::Expression(Expr::new(
        info,
        value,
    ));

    assert_eq!(1, tpl.block().tokens().len());
    assert_eq!(
        true,
        match token {
            ast::Token::Expression(_) => true,
            _ => false,
        }
    );

    assert_eq!(&expected, token);

    assert_eq!(
        false,
        match token {
            ast::Token::Expression(ref expr) => expr.is_raw(),
            _ => false,
        }
    );

    assert_eq!(
        false,
        match token {
            ast::Token::Expression(ref expr) => expr.escapes(),
            _ => false,
        }
    );

    Ok(())
}
