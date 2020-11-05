extern crate log;
extern crate pretty_env_logger;

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

    let content = include_str!("document.md");
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
    loader.add("partial", PathBuf::from("examples/partial.md"))?;
    loader.add(
        "dynamic-partial",
        PathBuf::from("examples/dynamic-partial.md"),
    )?;
    loader.add(
        "partial-block",
        PathBuf::from("examples/partial-block.md"),
    )?;
    loader.insert(name, content);

    let mut templates = Templates::new();
    templates
        .build(&loader)
        .expect("Failed to compile templates");

    //println!("{:#?}", templates.get(name));

    //let template = templates.get(name)?;
    //for node in template.node() {
    //println!("{:?}", node);
    //}

    let registry = Registry::new_templates(templates);

    match registry.render(name, &data) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => log::error!("{:?}", e),
    }

    Ok(())
}
