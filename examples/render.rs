extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use std::path::PathBuf;

use hbs::{
    parser::{Parser, ParserOptions},
    Loader, registry::{Registry, Templates}, Result,
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

    //let mut loader = Box::leak(Box::new(Loader::new()));
    let mut loader = Loader::new();
    loader.add("partial", PathBuf::from("examples/partial.md"))?;
    loader.insert(name, content);

    let mut templates = Templates::new();
    let mut registry = Registry::new();


    //let mut registry = Registry::new();
    for (k, v) in loader.sources() {
        println!("register with {:?}", k);
        templates.register(k, Registry::compile(v, Default::default()).unwrap());
    }

    //let child = std::thread::spawn(move || {
    match registry.render(&templates, name, &data) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => log::error!("{:?}", e),
    }
    //});
    //let res = child.join();

    Ok(())
}
