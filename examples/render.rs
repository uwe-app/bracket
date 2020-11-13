extern crate log;
extern crate pretty_env_logger;

use std::convert::TryFrom;
use std::path::PathBuf;

use bracket::{
    registry::Registry,
    template::{Loader, Templates},
    Result,
};

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
            "bar": {
                "qux": 42
            }
        },
        "partial-name": "dynamic-partial"
    });

    let mut loader = Loader::new();
    // NOTE: Call load() to use the file path as the name
    loader.load(PathBuf::from(name))?;

    let templates = Templates::try_from(&loader)?;
    let registry = Registry::from(templates);
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
