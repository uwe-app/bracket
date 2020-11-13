extern crate log;
extern crate pretty_env_logger;

use std::io::Write;

use bracket::{
    registry::Registry,
    Result,
};

use serde_json::json;

fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();

    let content = include_str!("files/log.md");
    let data = json!({"title": "Log Example"});
    let registry = Registry::new();
    match registry.once("log.md", content, &data) {
        Ok(result) => {
            write!(std::io::stdout(), "{}", result)?;
        }
        Err(e) => log::error!("{}", e),
    }

    Ok(())
}
