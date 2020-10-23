/// Generic wrapper for iterating into tokens using Logos modes.
///
/// O: Outer token type which *enters* into the mode.
/// I: Inner token type which *exits* the mode.
use logos::{Lexer, Logos, Span};

#[derive(Clone, Default)]
pub struct Extras {
    pub lines: usize,
}

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

                //println!("Adding token with line {:?}", inner.extras.lines);

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

/// Parse a mode block.
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
    moded.collect()
}

/// Normalize tokens into a single token by updating the span end
/// for a single token.
pub fn normalize<'source, O, I>(
    tokens: Vec<(Tokens<O, I>, Span)>,
    item: I,
    test: &dyn Fn(&I) -> bool,
) -> Vec<(Tokens<O, I>, Span)>
where
    O: Logos<'source, Source = str, Extras = Extras> + Clone + PartialEq,
    I: Logos<'source, Source = str, Extras = Extras> + Clone + PartialEq,
{
    let mut normalized: Vec<(Tokens<O, I>, Span)> = Vec::new();
    let mut span: Option<Span> = None;

    for (t, s) in tokens.into_iter() {
        match t {
            Tokens::OuterToken(outer) => {
                normalized.push((Tokens::OuterToken(outer), s)) 
            }    
            Tokens::InnerToken(inner) => {
                if test(&inner) {
                    if let Some(ref mut span) = span {
                        span.end = s.end; 
                    } else {
                        span = Some(s);
                    }
                } else {
                    if let Some(span) = span.take() {
                        normalized.push(
                            (Tokens::InnerToken(item.clone()), span));
                    }
                    normalized.push((Tokens::InnerToken(inner), s)) 
                }
            }    
        } 
    }

    normalized
}
