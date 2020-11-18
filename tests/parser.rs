use bracket::{
    parser::{ast::*, *},
    Result,
};

use serde_json::{Number, Value};

#[test]
fn parse_statement() -> Result<()> {
    let value = "{{foo}}";
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            assert_eq!(false, node.trim().before);
            assert_eq!(false, node.trim().after);
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_statement_sub_expr() -> Result<()> {
    let value = "{{log (json this)}}";
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();

            match node {
                Node::Statement(ref call) => {
                    assert_eq!("log", call.target().as_str());
                    assert_eq!(1, call.arguments().len());
                    let param = call.arguments().first().unwrap();
                    match param {
                        ParameterValue::SubExpr(ref call) => {
                            assert_eq!("json", call.target().as_str());
                            assert_eq!(1, call.arguments().len());
                        }
                        _ => panic!("Expecting sub expression call"),
                    }
                }
                _ => panic!("Expecting call statement"),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_statement_path_root() -> Result<()> {
    let value = "{{@root.foo}}";
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
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
fn parse_statement_path_parents() -> Result<()> {
    let value = "{{../../../foo}}";
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
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
fn parse_statement_path_explicit_this() -> Result<()> {
    let value = "{{this.foo}}";
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
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
fn parse_statement_path_explicit_dot() -> Result<()> {
    let value = "{{./foo}}";
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
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
fn parse_statement_partial() -> Result<()> {
    let value = "foo {{ > bar }} baz";
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;
    match node {
        Node::Document(doc) => {
            assert_eq!(3, doc.nodes().len());
            let node = doc.nodes().get(1).unwrap();
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
fn parse_arg_path() -> Result<()> {
    let value = r#"{{foo ../../bar}}"#;
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    assert_eq!(1, args.len());
                    match args.first().unwrap() {
                        ParameterValue::Path(ref path) => {
                            assert_eq!(2, path.parents());
                            assert_eq!(false, path.is_explicit());
                            assert_eq!(false, path.is_root());
                            assert_eq!(1, path.components().len());

                            let component = path.components().first().unwrap();
                            assert_eq!(true, component.is_identifier());
                            assert_eq!(false, component.is_local());
                        }
                        _ => panic!("Expected path argument value"),
                    }
                }
                _ => panic!("Expecting statement node."),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_arg_string() -> Result<()> {
    let value = r#"{{foo "bar\nbaz"}}"#;
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::from((Value::String(String::from(
                            "bar\nbaz"
                        )), 6..16, 0..1)),
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
fn parse_hash_string() -> Result<()> {
    let value = r#"{{foo bar="baz"}}"#;
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let hash = call.hash();
                    assert_eq!(1, hash.len());
                    assert_eq!(
                        &ParameterValue::from((Value::String(String::from(
                            r"baz"
                        )), 10..15, 0..1)),
                        hash.get("bar").unwrap()
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
fn parse_arg_bool_true() -> Result<()> {
    let value = r#"{{foo true}}"#;
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::from((Value::Bool(true), 6..10, 0..1)),
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
fn parse_arg_bool_false() -> Result<()> {
    let value = r#"{{foo false}}"#;
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::from((Value::Bool(false), 6..11, 0..1)),
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
fn parse_arg_null() -> Result<()> {
    let value = r#"{{foo null}}"#;
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::from((Value::Null, 6..10, 0..1)),
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
fn parse_arg_num_int() -> Result<()> {
    let value = r#"{{foo 10}}"#;
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    let expected = Number::from(10);
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::from((Value::Number(expected), 6..8, 0..1)),
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
fn parse_arg_num_int_signed() -> Result<()> {
    let value = r#"{{foo -10}}"#;
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    let expected = Number::from(-10);
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::from((Value::Number(expected), 6..9, 0..1)),
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
fn parse_arg_num_int_signed_exponent() -> Result<()> {
    let value = r#"{{foo -2e+2}}"#;
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    let expected: Number = "-2e+2".parse().unwrap();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::from((Value::Number(expected), 6..11, 0..1)),
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
fn parse_arg_num_float() -> Result<()> {
    let value = r#"{{foo 3.14}}"#;
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    let expected: Number = "3.14".parse().unwrap();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::from((Value::Number(expected), 6..10, 0..1)),
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
fn parse_arg_num_float_signed() -> Result<()> {
    let value = r#"{{foo -0.5}}"#;
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    let expected: Number = "-0.5".parse().unwrap();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::from((Value::Number(expected), 6..10, 0..1)),
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
fn parse_arg_num_float_signed_exponent() -> Result<()> {
    let value = r#"{{foo -0.5E-2}}"#;
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            match node {
                Node::Statement(ref call) => {
                    let args = call.arguments();
                    let expected: Number = "-0.5E-2".parse().unwrap();
                    assert_eq!(1, args.len());
                    assert_eq!(
                        &ParameterValue::from((Value::Number(expected), 6..13, 0..1)),
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
fn parse_statement_trim() -> Result<()> {
    let value = "{{~foo~}}";
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            assert_eq!(true, node.trim().before);
            assert_eq!(true, node.trim().after);
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_block_trim() -> Result<()> {
    let value = "{{~#foo~}}bar{{~/foo~}}";
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();
            assert_eq!(true, node.trim().before);
            assert_eq!(true, node.trim().after);

            match node {
                Node::Block(b) => {
                    assert_eq!(true, b.trim_close().before);
                    assert_eq!(true, b.trim_close().after);
                }
                _ => panic!("Expecting block node!"),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}

#[test]
fn parse_raw_block() -> Result<()> {
    let value = "{{{{~raw~}}}}foo{{{{~/raw~}}}}";
    let mut parser = Parser::new(value, Default::default());
    let node = parser.parse()?;

    match node {
        Node::Document(doc) => {
            assert_eq!(1, doc.nodes().len());
            let node = doc.nodes().first().unwrap();

            assert_eq!(true, node.trim().before);
            assert_eq!(true, node.trim().after);

            match node {
                Node::Block(block) => {
                    assert_eq!(true, block.is_raw());
                    assert_eq!(true, block.trim_close().before);
                    assert_eq!(true, block.trim_close().after);
                }
                _ => panic!("Expecting block node!"),
            }
        }
        _ => panic!("Bad root node type for parser()."),
    }

    Ok(())
}
