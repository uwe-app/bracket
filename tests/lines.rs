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

#[test]
fn lines_comment() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{!
This is a comment that spans multiple lines.
}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    if let Node::Comment(text) = node {
        assert_eq!(0..3, text.lines().clone());
    }
    Ok(())
}

#[test]
fn lines_raw_comment() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{!--
This is a raw comment that spans multiple lines.
--}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    if let Node::RawComment(text) = node {
        assert_eq!(0..3, text.lines().clone());
    }
    Ok(())
}

#[test]
fn lines_call_single() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{foo}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    if let Node::Statement(call) = node {
        assert_eq!(0..1, call.lines().clone());
    }
    Ok(())
}

#[test]
fn lines_call_multi() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{foo
bar
qux}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    if let Node::Statement(call) = node {
        assert_eq!(0..3, call.lines().clone());
    }
    Ok(())
}
