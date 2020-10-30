use hbs::{
    parser::{Parser, ParserOptions},
    Registry, Result,
};

use serde_json::json;

fn main() {
    let content = include_str!("document.md");
    let name = "document";
    let data = json!({"title": "Handlebars Test Document & Information"});
    let mut registry = Registry::new();
    registry.register_template_string(name, content, Default::default());
    let result = registry.render(name, &data).unwrap();
    println!("{}", result);
}
