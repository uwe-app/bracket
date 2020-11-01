extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use std::path::PathBuf;

use hbs::{
    parser::{Parser, ParserOptions},
    Registry, Result,
};

use serde_json::json;

fn main() -> Result<'static, ()> {
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();

    let content = include_str!("document.md");
    let name = "document";
    let data = json!({
        "title": "Handlebars Test Document & Information",
        "list": [1, 2, 3],
    });

    let mut registry = Registry::new();
    registry.register_template_file(
        "partial", PathBuf::from("examples/partial.md"))
        .expect("Unable to load partial");

    registry.register_template_string(name, content, Default::default());
    match registry.render(name, &data) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => log::error!("{:?}", e),
    }

    Ok(())
}
