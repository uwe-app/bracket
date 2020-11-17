use bracket::{parser::ast::{Node, Lines}, Registry, Result};

static NAME: &str = "lines.rs";

#[test]
fn lines_text() -> Result<()> {
    let registry = Registry::new();
    let value = r"This is some text
that spans multiple lines
so we can check the line range.";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    if let Node::Text(text) = node {
        assert_eq!(0..3, text.lines().clone());
    }
    Ok(())
}
