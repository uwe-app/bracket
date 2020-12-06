use bracket::{
    parser::ast::{CallTarget, Node, Slice},
    Registry, Result,
};

static NAME: &str = "slice.rs";

#[test]
fn slice_call() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{foo}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    assert_eq!(value, node.as_str());
    Ok(())
}

#[test]
fn slice_escaped_call() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{{foo}}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    assert_eq!(value, node.as_str());
    Ok(())
}

#[test]
fn slice_trim_call() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{~foo~}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    assert_eq!(value, node.as_str());
    Ok(())
}

#[test]
fn slice_block() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{#if true}}{{foo}}{{/if}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    assert_eq!(value, node.as_str());
    Ok(())
}

#[test]
fn slice_raw_block() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{{{raw}}}}{{foo}}{{{{/raw}}}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    assert_eq!(value, node.as_str());
    Ok(())
}

#[test]
fn slice_comment() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{!foo}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    assert_eq!(value, node.as_str());
    Ok(())
}

#[test]
fn slice_raw_comment() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{!-- foo --}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    assert_eq!(value, node.as_str());
    Ok(())
}

#[test]
fn slice_call_interspersed() -> Result<()> {
    let registry = Registry::new();
    let value = r"bar {{foo}} qux";
    let template = registry.parse(NAME, value)?;
    let mut it = template.node().into_iter();
    let node = it.next().unwrap();
    assert_eq!("bar ", node.as_str());
    let node = it.next().unwrap();
    assert_eq!("{{foo}}", node.as_str());
    let node = it.next().unwrap();
    assert_eq!(" qux", node.as_str());
    Ok(())
}

#[test]
fn slice_path_identifier() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{foo}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    if let Node::Statement(ref call) = node {
        if let CallTarget::Path(ref path) = call.target() {
            assert_eq!("foo", path.as_str());
        } else {
            panic!("Expecting path call target!");
        }
    } else {
        panic!("Expecting statement node!");
    }
    Ok(())
}

#[test]
fn slice_path_nested() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{foo.bar.qux}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    if let Node::Statement(ref call) = node {
        if let CallTarget::Path(ref path) = call.target() {
            assert_eq!("foo.bar.qux", path.as_str());
        } else {
            panic!("Expecting path call target!");
        }
    } else {
        panic!("Expecting statement node!");
    }
    Ok(())
}

#[test]
fn slice_path_root() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{@root.foo}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    if let Node::Statement(ref call) = node {
        if let CallTarget::Path(ref path) = call.target() {
            assert_eq!("@root.foo", path.as_str());
        } else {
            panic!("Expecting path call target!");
        }
    } else {
        panic!("Expecting statement node!");
    }
    Ok(())
}

#[test]
fn slice_path_parents() -> Result<()> {
    let registry = Registry::new();
    let value = r"{{../../../foo/bar.txt}}";
    let template = registry.parse(NAME, value)?;
    let node = template.node().into_iter().next().unwrap();
    if let Node::Statement(ref call) = node {
        if let CallTarget::Path(ref path) = call.target() {
            assert_eq!("../../../foo/bar.txt", path.as_str());
        } else {
            panic!("Expecting path call target!");
        }
    } else {
        panic!("Expecting statement node!");
    }
    Ok(())
}
