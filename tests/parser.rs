use hbs::{lexer::ast::*, lexer::parser::*, Result};

use serde_json::{Number, Value};

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
fn parse_statement_path_root() -> Result<'static, ()> {
    let value = "{{@root.foo}}";
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => match call.target() {
                    CallTarget::Path(ref path) => {
                        assert_eq!(true, path.is_root());
                    }
                    _ => panic!("Expecting path call target"),
                },
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }
    Ok(())
}

#[test]
fn parse_statement_path_parents() -> Result<'static, ()> {
    let value = "{{../../../foo}}";
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => match call.target() {
                    CallTarget::Path(ref path) => {
                        assert_eq!(3, path.parents());
                    }
                    _ => panic!("Expecting path call target"),
                },
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }
    Ok(())
}

#[test]
fn parse_statement_path_explicit_this() -> Result<'static, ()> {
    let value = "{{this.foo}}";
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => match call.target() {
                    CallTarget::Path(ref path) => {
                        assert_eq!(true, path.is_explicit());
                    }
                    _ => panic!("Expecting path call target"),
                },
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }
    Ok(())
}

#[test]
fn parse_statement_path_explicit_dot() -> Result<'static, ()> {
    let value = "{{./foo}}";
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => match call.target() {
                    CallTarget::Path(ref path) => {
                        assert_eq!(true, path.is_explicit());
                    }
                    _ => panic!("Expecting path call target"),
                },
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }
    Ok(())
}

#[test]
fn parse_statement_partial() -> Result<'static, ()> {
    let value = "{{ > foo}}";
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
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_arg_string() -> Result<'static, ()> {
    let value = r#"{{foo "bar\nbaz"}}"#;
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::Json(Value::String(String::from(
                            r"bar\nbaz"
                        ))),
                        args.first().unwrap()
                    );
                }
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_arg_bool_true() -> Result<'static, ()> {
    let value = r#"{{foo true}}"#;
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::Json(Value::Bool(true)),
                        args.first().unwrap()
                    );
                }
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_arg_bool_false() -> Result<'static, ()> {
    let value = r#"{{foo false}}"#;
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::Json(Value::Bool(false)),
                        args.first().unwrap()
                    );
                }
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_arg_null() -> Result<'static, ()> {
    let value = r#"{{foo null}}"#;
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::Json(Value::Null),
                        args.first().unwrap()
                    );
                }
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_arg_num_int() -> Result<'static, ()> {
    let value = r#"{{foo 10}}"#;
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    let expected = Number::from(10);
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::Json(Value::Number(expected)),
                        args.first().unwrap()
                    );
                }
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_arg_num_int_signed() -> Result<'static, ()> {
    let value = r#"{{foo -10}}"#;
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    let expected = Number::from(-10);
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::Json(Value::Number(expected)),
                        args.first().unwrap()
                    );
                }
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_arg_num_int_signed_exponent() -> Result<'static, ()> {
    let value = r#"{{foo -2e+2}}"#;
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    let expected: Number = "-2e+2".parse().unwrap();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::Json(Value::Number(expected)),
                        args.first().unwrap()
                    );
                }
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_arg_num_float() -> Result<'static, ()> {
    let value = r#"{{foo 3.14}}"#;
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    let expected: Number = "3.14".parse().unwrap();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::Json(Value::Number(expected)),
                        args.first().unwrap()
                    );
                }
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_arg_num_float_signed() -> Result<'static, ()> {
    let value = r#"{{foo -0.5}}"#;
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    let expected: Number = "-0.5".parse().unwrap();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::Json(Value::Number(expected)),
                        args.first().unwrap()
                    );
                }
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }
    Ok(())
}

#[test]
fn parse_arg_num_float_signed_exponent() -> Result<'static, ()> {
    let value = r#"{{foo -0.5E-2}}"#;
    let mut parser = Parser::new(Default::default());
    let node = parser.parse(value)?;

    match node {
        Node::Block(b) => {
            assert_eq!(&BlockType::Root, b.kind());
            assert_eq!(1, b.nodes().len());
            let node = b.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    let expected: Number = "-0.5E-2".parse().unwrap();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::Json(Value::Number(expected)),
                        args.first().unwrap()
                    );
                }
                _ => panic!("Expecting statement node."),
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
