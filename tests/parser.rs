use hbs::{Result, lexer::parser::*, lexer::ast::*};

#[test]
fn parse_statement() -> Result<'static, ()> {
    let value = "{{foo}}";
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            assert_eq!(false, b.nodes().first().unwrap().trim_before());
            assert_eq!(false, b.nodes().first().unwrap().trim_after());
        }
        _ => {
            panic!("Bad node type for parser().")
        }
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
            assert_eq!(true, b.nodes().first().unwrap().trim_before());
            assert_eq!(true, b.nodes().first().unwrap().trim_after());
        }
        _ => {
            panic!("Bad node type for parser().")
        }
    }

    Ok(())
}
