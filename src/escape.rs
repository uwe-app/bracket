//! Escape function trait and default functions.
//!
//! The default is to escape for HTML content using `escape_html`.

/// Type for escape functions.
pub type EscapeFn = Box<dyn Fn(&str) -> String + Send + Sync>;

/// Escape for HTML output.
pub fn html(s: &str) -> String {
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

/// Do not escape output.
pub fn noop(s: &str) -> String {
    s.to_owned()
}
