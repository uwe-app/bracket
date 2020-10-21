use hbs::{Error, Result, Template};

fn main() -> Result<()> {
    let s = r"\{{expr}}
{{{unescaped}}}

{{var}}

{{{{  raw }}}}
This is some raw text.
{{{{/raw}}}}

{{# block}}
This is some block text with an {{inline}}
{{/block}}

{{> partial}}

{{#> partial-block}}
{{@partial-block}}
{{/partial-block}}
";

    match Template::compile(s) {
        Ok(tpl) => {
            println!("{:#?}", tpl);
            println!("{}", tpl.to_string());
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
