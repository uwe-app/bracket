use hbs::{Result, Template, lexer::parser::ParserOptions};

fn main() -> Result<()> {
    let s = r#"\{{expr}}
{{{unescaped}}}

{{var}}

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

{{[1,2,3]}}
{{true}}
{{false}}
{{null}}

{{foo {"a": "b"}}}
"#;

    let options = ParserOptions {
        file_name: String::from("src/main.rs"),
        line_offset: 3
    };

    match Template::compile(s, options) {
        Ok(tpl) => {
            //println!("{:#?}", tpl);
            println!("{}", tpl.to_string());
        }
        Err(e) => {
            eprintln!("{}", e);
            //e.print(s).unwrap();
            std::process::exit(1);
        }
    }

    Ok(())
}
