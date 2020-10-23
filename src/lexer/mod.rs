use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceInfo {
    pub line: Range<usize>,
    pub span: logos::Span,
}

impl SourceInfo {
    pub fn set_range(&mut self, span: logos::Span) {
        self.span = span
    }
}

pub mod ast;
pub mod grammar;
pub mod parser;
