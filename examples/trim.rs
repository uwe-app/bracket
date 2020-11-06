extern crate log;
extern crate pretty_env_logger;

use bracket::{
    registry::Registry,
    Result,
};

use serde_json::json;

/// Demonstrates how to iterate a template nodes and include 
/// trim state information.
fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();

    let content = include_str!("files/whitespace.md");
    let data = json!({
        "foo": "bar",
    });

    let registry = Registry::new();
    let template = registry.parse("trim.rs", content)?;

    for node in template.node().iter().trim() {
        println!("{:#?}", node);
    }

    Ok(())
}
