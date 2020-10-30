use hbs::{
    parser::{Parser, ParserOptions},
    Result,
};

/// Demonstrates how to get a document tree of nodes.
fn main() -> Result<'static, ()> {
    let content = include_str!("document.md");
    let options = ParserOptions {
        file_name: String::from("document.md"),
        line_offset: 0,
        byte_offset: 0,
    };
    let mut parser = Parser::new(content, options);
    let doc = parser.parse()?;
    println!("{:#?}", doc);
    Ok(())
}
