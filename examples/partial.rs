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
    let name = "examples/files/partial-document.md";
    let data = json!({
        "title": "Partial Example",
        "partial-name": "partial-dynamic"
    });

    let mut loader = Loader::new();
    loader.read_dir(PathBuf::from("examples/files/partials/"), "hbs")?;
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
