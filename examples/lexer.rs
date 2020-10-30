use hbs::lexer::lex;

/// Demonstrates low-level access to the token stream.
fn main() {
    let content = include_str!("document.md");
    for token in lex(content) {
        println!("{:#?}", token);
    }
}
