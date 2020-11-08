//! Types that control how whitespace is trimmed.
use crate::parser::ast::Node;

/// State that indicates how whitespace should be trimmed
/// from the node being rendered.
#[derive(Clone, Copy, Default, Debug)]
pub struct TrimState {
    pub start: bool,
    pub end: bool,
}

impl<'a> From<(&'a Node<'a>, &'a Node<'a>)> for TrimState {
    fn from(nodes: (&'a Node<'a>, &'a Node<'a>)) -> Self {
        let (previous, next) = nodes;
        Self {
            start: previous.trim().after,
            end: next.trim().before,
        }
    }
}

impl From<(bool, bool)> for TrimState {
    fn from(values: (bool, bool)) -> Self {
        Self {
            start: values.0,
            end: values.1,
        }
    }
}

/// Hint that indicates how whitespace should be trimmed
/// for nodes before and after the current node.
#[derive(Clone, Copy, Default, Debug)]
pub struct TrimHint {
    pub before: bool,
    pub after: bool,
}
