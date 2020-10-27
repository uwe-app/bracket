use hbs::{lexer::ast::*, lexer::parser::*, Result};

#[test]
fn parse_statement() -> Result<'static, ()> {
    let value = "{{foo}}";
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            assert_eq!(false, node.trim_before());
            assert_eq!(false, node.trim_after());
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_statement_partial() -> Result<'static, ()> {
    let value = "{{> foo}}";
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    assert_eq!(true, call.is_partial());
                }
                _ => panic!("Expecting statement node.")
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_statement_trim() -> Result<'static, ()> {
    let value = "{{~foo~}}";
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            assert_eq!(true, node.trim_before());
            assert_eq!(true, node.trim_after());
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_block_trim() -> Result<'static, ()> {
    let value = "{{~#foo~}}bar{{~/foo~}}";
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            assert_eq!(true, node.trim_before());
            assert_eq!(true, node.trim_after());

            match node {
                Node::Block(b) => {
                    assert_eq!(true, b.trim_before_close());
                    assert_eq!(true, b.trim_after_close());
                }
                _ => panic!("Expecting block node!"),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}
