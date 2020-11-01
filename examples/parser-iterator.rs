use hbs::{
    parser::{Parser, ParserOptions},
    Result,
};

/// Demonstrates how to get nodes by iterating a parser.
fn main() -> Result<'static, ()> {
    let content = include_str!("document.md");
    let options = ParserOptions {
        file_name: String::from("document.md"),
        line_offset: 0,
        byte_offset: 0,
    };
    let mut parser = Parser::new(content, options);
    for node in parser {
        let node = node?;
        println!("{:#?}", node);
    }
    Ok(())
}