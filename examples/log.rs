extern crate log;
extern crate pretty_env_logger;

use std::convert::TryFrom;
use std::path::PathBuf;
use std::io::Write;

use bracket::{
    registry::Registry,
    template::{Loader, Templates},
    Result,
};

use serde_json::json;

fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();

    let content = include_str!("files/log.md");
    let data = json!({"title": "Log"});
    let mut registry = Registry::new();
    match registry.once("log.md", content, &data) {
        Ok(result) => {
            write!(std::io::stdout(), "{}", result)?;
        }
        Err(e) => log::error!("{}", e),
    }

    Ok(())
}
