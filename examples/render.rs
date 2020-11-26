extern crate log;
extern crate pretty_env_logger;

use std::path::PathBuf;

use bracket::{registry::Registry, Result};

use serde_json::json;

fn render() -> Result<String> {
    let name = "examples/files/document.md";
    let data = json!({
        "title": "Handlebars Test Document & Information",
        "list": [1, 2, 3],
        "map": {
            "apples": 1,
            "oranges": 2,
            "pears": 3,
        },
        "foo": {
            "b.r": {
                "qux": 42
            },
            "q'x": {
                "bar": 76
            }
        }
    });

    let mut registry = Registry::new();
    registry.load(PathBuf::from(name))?;
    registry.render(name, &data)
}

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();

    match render() {
        Ok(result) => println!("{}", result),
        // NOTE: Use Debug to print errors with source code snippets
        Err(e) => log::error!("{:?}", e),
    }
}
