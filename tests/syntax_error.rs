use bracket::{
    error::{Error, ErrorInfo, SourcePos, SyntaxError},
    Registry, Result,
};

#[test]
fn err_empty_statement() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{}}";

    match registry.compile(value, Default::default()) {
        Ok(_) => panic!("Empty statement error expected"),
        Err(e) => {
            println!("{:?}", e);
            let pos = SourcePos(0, 2);
            let info = ErrorInfo::new(value, "unknown", pos, vec![]);
            assert_eq!(
                Error::Syntax(SyntaxError::EmptyStatement(info.into())),
                e
            );
        }
    }
    Ok(())
}
