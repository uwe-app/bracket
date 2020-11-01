extern crate log;
extern crate pretty_env_logger;

use std::path::PathBuf;

use hbs::{
    registry::Registry,
    template::{Loader, Templates},
    Result,
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

    let mut loader = Loader::new();
    loader.add("partial", PathBuf::from("examples/partial.md"))?;
    loader.insert(name, content);

    let mut templates = Templates::new();
    templates
        .build(&loader)
        .expect("Failed to compile templates");

    let mut registry = Registry::new_templates(templates);

    //let child = std::thread::spawn(move || {
    match registry.render(name, &data) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => log::error!("{:?}", e),
    }
    //});
    //let res = child.join();

    Ok(())
}
