extern crate log;
extern crate pretty_env_logger;

use bracket::{
    registry::Registry,
    template::{Loader, Templates},
    Error, Result,
};

fn render() -> Result<()> {
    let registry = Registry::new();
    let errors = registry
        .lint("examples/files/lint.md", include_str!("files/lint.md"))?;
    for e in errors {
        log::warn!("{:?}", e);
    }
    Ok(())
}

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();

    match render() {
        Err(e) => log::error!("Unexpected lint error: {:?}", e),
        _ => {}
    }
}
