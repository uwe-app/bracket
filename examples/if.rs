extern crate log;
extern crate pretty_env_logger;

use std::path::PathBuf;

use bracket::{
    registry::Registry,
    template::{Loader, Templates},
    Result,
};

use serde_json::json;

fn render() -> Result<String> {
    let name = "examples/files/if.md";
    let data = json!({
        "title": "If Example",
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
