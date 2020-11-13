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

fn render () -> Result<String> {
    let name = "examples/files/partial-document.md";
    let data = json!({
        "title": "Partial Example",
        "partial-name": "dynamic-partial"
    });

    let mut loader = Loader::new();
    loader.add("partial-named", PathBuf::from("examples/files/partial-named.md"))?;
    loader.add(
        "dynamic-partial",
        PathBuf::from("examples/files/dynamic-partial.md"),
    )?;
    loader.add(
        "partial-block",
        PathBuf::from("examples/files/partial-block.md"),
    )?;

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
