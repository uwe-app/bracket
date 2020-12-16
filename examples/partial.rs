extern crate log;
extern crate pretty_env_logger;

use std::path::PathBuf;
use bracket::{helper::prelude::*, Registry, Result};
use serde_json::json;

pub struct LogFileName;
impl Helper for LogFileName {
    fn call<'render, 'call>(
        &self,
        rc: &mut Render<'render>,
        _ctx: &Context<'call>,
        _template: Option<&'render Node<'render>>,
    ) -> HelperValue {
        log::info!("Partial file name {:?}", rc.current_name());
        Ok(None)
    }
}

fn render() -> Result<String> {
    let name = "examples/files/partial.md";
    let data = json!({
        "title": "Partial Example",
        "partial-name": "partial-dynamic"
    });

    let mut registry = Registry::new();

    registry.helpers_mut().insert("log_file_name", Box::new(LogFileName {}));

    registry.read_dir(PathBuf::from("examples/files/partials/"), "hbs")?;
    registry.load(PathBuf::from(name))?;
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
