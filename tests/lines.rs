use bracket::{
    parser::ast::{CallTarget, Lines, Node},
    Registry, Result,
};

const NAME: &str = "lines.rs";

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
    let value = r"{{
foo.bar.qux
}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    if let Node::Statement(call) = node {
        assert_eq!(0..3, call.lines().clone());
        let target = call.target();
        assert_eq!(1..2, target.lines().clone());
        if let CallTarget::Path(ref path) = target {
            assert_eq!(1..2, path.lines().clone());
        }
    }
    Ok(())
}

#[test]
fn lines_call_multi() -> Result<()> {
    let registry = Registry::new();
    let value = r#"{{
foo
"message"
true
}}"#;
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    if let Node::Statement(call) = node {
        assert_eq!(0..5, call.lines().clone());
    }
    Ok(())
}

#[test]
fn lines_raw_block() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{{{raw}}}}
This is some text in a raw block.
{{{{/raw}}}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    if let Node::Block(block) = node {
        assert_eq!(0..3, block.lines().clone());
    }
    Ok(())
}

#[test]
fn lines_block() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#block}}
This is some text in a block statement.

If can have other {{foo}} statements.
{{/block}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    if let Node::Block(block) = node {
        assert_eq!(0..5, block.lines().clone());
    }
    Ok(())
}
