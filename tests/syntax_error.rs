use bracket::{
    error::{Error, ErrorInfo, SourcePos, SyntaxError},
    Registry, Result,
};
use serde_json::json;

#[test]
fn err_empty_statement() -> Result<()> {
    let mut registry = Registry::new();
    let name = "mock-template";
    let value = r"{{}}";
    let data = json!({});

    match registry.compile(value, Default::default()) {
        Ok(_) => panic!("Empty statement error expected"),
        Err(e) => {
            println!("{:?}", e);
            let pos = SourcePos(0, 2);
            let info = ErrorInfo::new(value, "unknown", pos);
            assert_eq!(
                Error::Syntax(SyntaxError::EmptyStatement(info.into())),
                e
            );
        }
    }
    Ok(())
}
