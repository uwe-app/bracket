use bracket::{
    error::{Error, ErrorInfo, SourcePos, SyntaxError},
    Registry, Result,
};

static NAME: &str = "syntax_error.rs";

#[test]
fn syntax_err_empty_statement() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{}}";
    match registry.parse(NAME, value) {
        Ok(_) => panic!("Empty statement error expected"),
        Err(e) => {
            println!("{:?}", e);
            let pos = SourcePos(0, 2);
            let info = ErrorInfo::new(value, NAME, pos, vec![]);
            assert_eq!(
                Error::Syntax(SyntaxError::ExpectedIdentifier(info.into())),
                e
            );
        }
    }
    Ok(())
}

#[test]
fn syntax_err_identifier_expected() -> Result<()> {
    let registry = Registry::new();
    let value = r#"{{# }}"#;
    match registry.parse(NAME, value) {
        Ok(_) => panic!("Identifier expected error expected"),
        Err(e) => {
            println!("{:?}", e);
            let pos = SourcePos(0, 4);
            let info = ErrorInfo::new(value, NAME, pos, vec![]);
            assert_eq!(
                Error::Syntax(SyntaxError::ExpectedIdentifier(info.into())),
                e
            );
        }
    }
    Ok(())
}

#[test]
fn syntax_err_sub_expr() -> Result<()> {
    let registry = Registry::new();
    let value = r#"{{#> (foo}}"#;
    match registry.parse(NAME, value) {
        Ok(_) => panic!("Sub expression not terminated error expected"),
        Err(e) => {
            println!("{:?}", e);
            let pos = SourcePos(0, 9);
            let info = ErrorInfo::new(value, NAME, pos, vec![]);
            assert_eq!(
                Error::Syntax(
                    SyntaxError::SubExpressionNotTerminated(info.into())),
                e
            );
        }
    }
    Ok(())
}

#[test]
fn syntax_err_link() -> Result<()> {
    let registry = Registry::new();
    let value = r#"[[SomeLink|Page"#;
    match registry.parse(NAME, value) {
        Ok(_) => panic!("Link not terminated error expected"),
        Err(e) => {
            println!("{:?}", e);
            let pos = SourcePos(0, 14);
            let info = ErrorInfo::new(value, NAME, pos, vec![]);
            assert_eq!(
                Error::Syntax(
                    SyntaxError::LinkNotTerminated(info.into())),
                e
            );
        }
    }
    Ok(())
}

#[test]
fn syntax_err_raw_block_open() -> Result<()> {
    let registry = Registry::new();
    let value = r#"{{{{raw"#;
    match registry.parse(NAME, value) {
        Ok(_) => panic!("Raw block open error expected"),
        Err(e) => {
            println!("{:?}", e);
            let pos = SourcePos(0, 4);
            let info = ErrorInfo::new(value, NAME, pos, vec![]);
            assert_eq!(
                Error::Syntax(
                    SyntaxError::RawBlockOpenNotTerminated(info.into())),
                e
            );
        }
    }
    Ok(())
}
