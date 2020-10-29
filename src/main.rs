use hbs::{
    parser::{Parser, ParserOptions},
    Registry, Result,
};

fn main() -> Result<'static, ()> {
    let s = r#"\{{expr}}
{{{unescaped}}}

{{var foo="bar"}}

{{> (var)}}

\{{ > }}
\{{ > a.b}}

{{{{  raw }}}}
This is some raw text {{inline-raw}}.
{{{{/raw}}}}

{{# test-block}}
This is some block text with an {{inline}}
{{/test-block}}

{{> partial}}

{{#> partial-block}}
{{@partial-block}}
{{/partial-block}}

{{!-- a comment --}}

{{foo null true false -20 3.14 2E+2 "blah\"baz"}}

\{{[1,2,3]}}
\{{false}}
\{{null}}

\{{foo {"a": "b"}}}
"#;

    //let s = "{{ > }}";

    //let s = "{{{{raw}}}}foo{{{{/raw}}}}{{!-- raw comment --}}";
    //let s = "{{foo ../bar}}";
    //let s = "Some text";

    let s = "{{# block}}foo{{/block}}";

    let options = ParserOptions {
        file_name: String::from("src/main.rs"),
        line_offset: 3,
        byte_offset: 0,
    };

    let mut parser = Parser::new(s, options);

    for node in parser {
        let node = node?;
        println!("Got a node via iterator {:#?}", node);
    }

    //parser.parse().expect("Failed to parse!");

    /*
    match Registry::compile(s, options) {
        Ok(tpl) => {
            //println!("{:#?}", tpl);
            println!("{}", tpl);
        }
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }
    */

    Ok(())
}
