extern crate log;
extern crate pretty_env_logger;

use std::path::PathBuf;

use bracket::{registry::Registry, Result};

use serde_json::json;

fn render() -> Result<String> {
    let name = "examples/files/with.md";
    let data = json!({
        "title": "With Example",
        "list": [1, 2, 3]
    });

    let mut registry = Registry::new();
    registry.load(PathBuf::from(name))?;
    registry.build(registry.sources())?;
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
