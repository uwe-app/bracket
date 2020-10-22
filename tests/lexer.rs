use hbs::Template;
use hbs::{lexer::*, Result};

#[test]
fn escaped_expr() -> Result<()> {
    let value = r"\{{expr}}";
    let tpl = Template::compile(value)?;
    let token = tpl.block().tokens().get(0).unwrap();
    let info = SourceInfo {
        line: 0..0,
        span: 0..9,
    };
    let expected = AstToken::Expression(Expression {
        info,
        value: value.to_string(),
    });

    assert_eq!(1, tpl.block().tokens().len());
    assert_eq!(
        true,
        match token {
            AstToken::Expression(_) => true,
            _ => false,
        }
    );

    assert_eq!(&expected, token);

    assert_eq!(
        true,
        match token {
            AstToken::Expression(ref expr) => expr.is_raw(),
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
    let expected = AstToken::Expression(Expression {
        info,
        value: value.to_string(),
    });

    assert_eq!(1, tpl.block().tokens().len());
    assert_eq!(
        true,
        match token {
            AstToken::Expression(_) => true,
            _ => false,
        }
    );

    assert_eq!(&expected, token);

    assert_eq!(
        false,
        match token {
            AstToken::Expression(ref expr) => expr.is_raw(),
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
    let expected = AstToken::Expression(Expression {
        info,
        value: value.to_string(),
    });

    assert_eq!(1, tpl.block().tokens().len());
    assert_eq!(
        true,
        match token {
            AstToken::Expression(_) => true,
            _ => false,
        }
    );

    assert_eq!(&expected, token);

    assert_eq!(
        false,
        match token {
            AstToken::Expression(ref expr) => expr.is_raw(),
            _ => false,
        }
    );

    assert_eq!(
        false,
        match token {
            AstToken::Expression(ref expr) => expr.escapes(),
            _ => false,
        }
    );

    Ok(())
}
