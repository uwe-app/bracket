extern crate log;
extern crate pretty_env_logger;

use std::path::PathBuf;

use bracket::{
    registry::Registry,
    Result,
};

use serde_json::json;

fn render() -> Result<String> {
    let name = "examples/files/each.md";
    let data = json!({
        "title": "Each Example",
        "list": [1, 2, 3],
        "map": {
            "apples": 1,
            "oranges": 2,
            "pears": 3,
        }
    });

    let mut registry = Registry::new();
    registry.load(PathBuf::from(name))?;
    registry.build()?;

    registry.render(name, &data)
}

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();
    match render() {
        Ok(result) => println!("{}", result),
        Err(e) => log::error!("{:?}", e),
    }
}
