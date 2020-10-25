use hbs::{error::SyntaxError, Error, Registry, Result};
use serde_json::json;

#[test]
fn err_empty_statement() -> Result<()> {
    let mut registry = Registry::new();
    let name = "mock-template";
    let value = r"{{}}";
    let data = json!({});
    match registry.register_template_string(name, value) {
        Ok(_) => panic!("Empty statement error expected"),
        Err(e) => {
            println!("{}", e.to_string());
            assert_eq!(Error::Syntax(SyntaxError::EmptyStatement(2)), e);
        }
    }
    Ok(())
}
