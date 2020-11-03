extern crate log;

use bracket::registry::Registry;

use serde_json::json;

fn main() {
    let mut registry = Registry::new();
    let name = "main.rs";
    let value = r#"{{foo.bar.qux true false null 3.14  "blah" key=value}}"#;
    let data = json!({});
    let template = registry.compile(value, Default::default()).unwrap();

    println!("{:?}", template);

    //match registry.once(name, &template, &data) {
        //Ok(result) => {
            //println!("{}", result);
        //}
        //Err(e) => log::error!("{:?}", e),
    //}
}
