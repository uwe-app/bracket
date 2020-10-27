use hbs::{
    error::{ErrorInfo, SourcePos, SyntaxError},
    lexer::parser::ParserOptions,
    Error, Registry, Result,
};
use serde_json::json;

#[test]
fn err_empty_statement() -> Result<'static, ()> {
    let mut registry = Registry::new();
    let name = "mock-template";
    let value = r"{{}}";
    let data = json!({});
    let options: ParserOptions = Default::default();
    match registry.register_template_string(name, value, Default::default()) {
        Ok(_) => panic!("Empty statement error expected"),
        Err(e) => {
            println!("{:?}", e);
            let pos = SourcePos(0, 2);
            let info = ErrorInfo::from((value, &options, pos, vec![]));
            assert_eq!(Error::Syntax(SyntaxError::EmptyStatement(info)), e);
        }
    }
    Ok(())
}
