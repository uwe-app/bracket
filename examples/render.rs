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

fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();

    let content = include_str!("files/document.md");
    let name = "document";
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
    loader.add("partial", PathBuf::from("examples/files/partial.md"))?;
    loader.add(
        "dynamic-partial",
        PathBuf::from("examples/files/dynamic-partial.md"),
    )?;
    loader.add(
        "partial-block",
        PathBuf::from("examples/files/partial-block.md"),
    )?;
    loader.insert(name, content);

    let mut templates = Templates::try_from(&loader)?;

    //println!("{:#?}", templates.get(name));

    //let template = templates.get(name)?;
    //for node in template.node() {
    //println!("{:?}", node);
    //}

    let mut registry = Registry::from(templates);
    registry.set_strict(false);

    match registry.render(name, &data) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => log::error!("{}", e),
    }

    Ok(())
}
