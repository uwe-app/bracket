/// Generic wrapper for iterating into tokens using Logos modes.
///
/// O: Outer token type which *enters* into the mode.
/// I: Inner token type which *exits* the mode.

//use std::borrow::ToOwned;
//use std::clone::Clone;
use logos::{Lexer, Logos, Span};

enum Modes<'source, O: Logos<'source, Source = str> + PartialEq, I: Logos<'source> + PartialEq> {
    Outer(Lexer<'source, O>),
    Inner(Lexer<'source, I>),
}

impl<'source, O: Logos<'source, Source = str> + PartialEq, I: Logos<'source> + PartialEq> Modes<'source, O, I> {
    fn new(s: &'source str) -> Self {
        Self::Outer(O::lexer(s))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Tokens<O, I> {
    OuterToken(O),
    InnerToken(I),
}

struct ModeBridge<'source, O: Logos<'source, Source = str> + PartialEq, I: Logos<'source> + PartialEq> {
    mode: Modes<'source, O, I>,
    start: O,
    end: I,
}

// Clones as we switch between modes
impl<'source, O: Logos<'source, Source = str> + PartialEq, I: Logos<'source> + PartialEq> Iterator
    for ModeBridge<'source, O, I>
{
    type Item = (Tokens<O, I>, Span);
    fn next(&mut self) -> Option<Self::Item> {
        use Tokens::*;
        match &mut self.mode {
            Modes::Inner(inner) => {
                let result = inner.next();
                let span = inner.span();
                //println!("Inner span {:?}", span);
                if let Some(token) = result {
                    if self.end == token {
                        //self.mode = Modes::Outer(inner.to_owned().morph());
                    }
                    Some((InnerToken(token), span))
                } else {
                    None
                }
            }
            Modes::Outer(outer) => {
                let result = outer.next();
                let span = outer.span();
                //println!("Outer span {:?}", span);
                if let Some(token) = result {
                    if self.start == token {
                        //self.mode = Modes::Inner(outer.to_owned().morph());
                    }
                    Some((OuterToken(token), span))
                } else {
                    None
                }
            }
        }
    }
}

pub fn lex<'source, O: Logos<'source, Source = str> + PartialEq, I: Logos<'source> + PartialEq>(
    s: &'source str,
    start: O,
    end: I,
) -> Vec<(Tokens<O, I>, Span)> {
    let moded = ModeBridge::<O, I> {
        mode: Modes::new(s),
        start,
        end,
    };
    let results: Vec<(Tokens<O, I>, Span)> = moded.collect();
    results
}
