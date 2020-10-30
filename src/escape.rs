pub type EscapeFn = Box<dyn Fn(&str) -> String + Send + Sync>;

// SEE: https://github.com/sunng87/handlebars-rust/blob/d8eff5d139fa7ff9a84882e0cda86fa0db4eeb8e/src/support.rs#L42-L58
pub fn html_escape(s: &str) -> String {
    let mut output = String::new();
    for c in s.chars() {
        match c {
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '&' => output.push_str("&amp;"),
            '\'' => output.push_str("&#x27;"),
            '`' => output.push_str("&#x60;"),
            '=' => output.push_str("&#x3D;"),
            _ => output.push(c),
        }
    }
    output
}

pub fn no_escape(s: &str) -> String {
    s.to_owned()
}
