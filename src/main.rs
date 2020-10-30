use hbs::{
    parser::{Parser, ParserOptions},
    Registry, Result,
};

use serde_json::json;

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

    let name = "test";
    //let s = "some {{# bar}}foo{{/bar}} text";
    let s = "{{title}}";
    let data = json!({"title": "foo"});

    let mut registry = Registry::new();
    registry.register_template_string(name, s, Default::default());

    let result = registry.render(name, &data).unwrap();

    println!("Render {:?}", result);

    /*
    let options = ParserOptions {
        file_name: String::from("src/main.rs"),
        line_offset: 0,
        byte_offset: 0,
    };

    let mut parser = Parser::new(s, options);
    let doc = parser.parse()?;
    for node in doc.iter() {
        println!("Got node {:?}", node);
    }
    */

    //for node in parser {
    //let node = node?;
    //println!("{:#?}", node);
    //}

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
