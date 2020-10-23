/// Generic wrapper for iterating into tokens using Logos modes.
///
/// O: Outer token type which *enters* into the mode.
/// I: Inner token type which *exits* the mode.
use logos::{Lexer, Logos, Span};

#[derive(Clone, Default)]
pub struct Extras;

enum Modes<'source, O, I>
where
    O: Logos<'source, Source = str, Extras = Extras> + Clone + PartialEq,
    I: Logos<'source, Source = str, Extras = Extras> + Clone + PartialEq,
{
    Outer(Lexer<'source, O>),
    Inner(Lexer<'source, I>),
}

impl<'source, O, I> Modes<'source, O, I>
where
    O: Logos<'source, Source = str, Extras = Extras> + Clone + PartialEq,
    I: Logos<'source, Source = str, Extras = Extras> + Clone + PartialEq,
{
    fn new(s: &'source str) -> Self {
        Self::Outer(O::lexer(s))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Tokens<O, I> {
    OuterToken(O),
    InnerToken(I),
}

struct ModeBridge<'source, O, I>
where
    O: Logos<'source, Source = str, Extras = Extras> + Clone + PartialEq,
    I: Logos<'source, Source = str, Extras = Extras> + Clone + PartialEq,
{
    mode: Modes<'source, O, I>,
    start: O,
    end: I,
}

// Clones as we switch between modes
impl<'source, O, I> Iterator for ModeBridge<'source, O, I>
where
    O: Logos<'source, Source = str, Extras = Extras> + Clone + PartialEq,
    I: Logos<'source, Source = str, Extras = Extras> + Clone + PartialEq,
{
    type Item = (Tokens<O, I>, Span);
    fn next(&mut self) -> Option<Self::Item> {
        use Tokens::*;
        match &mut self.mode {
            Modes::Inner(inner) => {
                let result = inner.next();
                let span = inner.span();
                if let Some(token) = result {
                    if self.end == token {
                        self.mode = Modes::Outer(inner.to_owned().morph());
                    }
                    Some((InnerToken(token), span))
                } else {
                    None
                }
            }
            Modes::Outer(outer) => {
                let result = outer.next();
                let span = outer.span();
                if let Some(token) = result {
                    if self.start == token {
                        self.mode = Modes::Inner(outer.to_owned().morph());
                    }
                    Some((OuterToken(token), span))
                } else {
                    None
                }
            }
        }
    }
}

pub fn lex<'source, O, I>(
    s: &'source str,
    start: O,
    end: I,
) -> Vec<(Tokens<O, I>, Span)>
where
    O: Logos<'source, Source = str, Extras = Extras> + Clone + PartialEq,
    I: Logos<'source, Source = str, Extras = Extras> + Clone + PartialEq,
{
    let moded = ModeBridge::<O, I> {
        mode: Modes::new(s),
        start,
        end,
    };
    let results: Vec<(Tokens<O, I>, Span)> = moded.collect();
    results
}
