extern crate log;

use bracket::registry::Registry;

use serde_json::json;

fn main() {
    let mut registry = Registry::new();
    let name = "main.rs";
    let value = "{{foo.bar.qux}}";
    let data = json!({});
    let template = registry.compile(value, Default::default()).unwrap();

    //match registry.once(name, &template, &data) {
        //Ok(result) => {
            //println!("{}", result);
        //}
        //Err(e) => log::error!("{:?}", e),
    //}
}
