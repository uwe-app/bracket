//! Iterators for the AST nodes.
use crate::{
    parser::ast::Node,
    trim::{TrimHint, TrimState},
};

/// Event that encapsulates a node and whitespace trim state.
#[derive(Debug)]
pub struct NodeEvent<'a> {
    /// The node being emitted.
    pub node: &'a Node<'a>,
    /// The trim state for the node.
    pub trim: TrimState,
    /// Whether this is the first event in the current iteration.
    pub first: bool,
    /// Whether this is the last event in the current iteration.
    pub last: bool,
}

impl<'a> NodeEvent<'a> {
    pub fn new(
        node: &'a Node,
        trim: TrimState,
        first: bool,
        last: bool,
    ) -> Self {
        Self {
            node,
            trim,
            first,
            last,
        }
    }
}

/// Iterator for branch nodes.
///
/// Descends into document and block nodes and yields the child 
/// nodes.
pub struct BranchIter<'source> {
    node: &'source Node<'source>,
    children: Option<std::slice::Iter<'source, Node<'source>>>,
}

impl<'source> BranchIter<'source> {
    /// Create a new branch iterator.
    pub fn new(node: &'source Node) -> Self {
        Self {
            node,
            children: None,
        }
    }

    /// Create an iterator that adds trim state information
    /// to each node.
    ///
    /// The input hint will be used to determine the trim state 
    /// of the first node.
    pub fn event(self, hint: Option<TrimHint>) -> EventIter<'source> {
        EventIter::new(self, hint)
    }
}

impl<'source> Iterator for BranchIter<'source> {
    type Item = &'source Node<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        let iter = match *self.node {
            Node::Document(ref node) => {
                Some(self.children.get_or_insert(node.nodes().iter()))
            }
            Node::Block(ref node) => {
                Some(self.children.get_or_insert(node.nodes().iter()))
            }
            Node::Text(_) => None,
            Node::Statement(_) => None,
            Node::RawStatement(_) | Node::RawComment(_) | Node::Comment(_) => {
                None
            }
        };

        if let Some(it) = iter {
            let child = it.next();
            if child.is_none() {
                self.children.take();
            }
            child
        } else {
            None
        }
    }
}

/// Iterator that yields node events.
///
/// Node events contain the underlying node and a trim state that indicates
/// whether the current node should have leading and trailing
/// whitespace removed.
///
/// They may also be seeded with a [TrimHint](crate::trim::TrimHint) from a 
/// previous iteration.
pub struct EventIter<'source> {
    iter: std::iter::Peekable<BranchIter<'source>>,
    prev_trim_after: Option<bool>,
    hint: Option<TrimHint>,
}

impl<'source> EventIter<'source> {

    /// Create a new event iterator.
    pub(crate) fn new(
        nodes: BranchIter<'source>,
        hint: Option<TrimHint>,
    ) -> Self {
        let iter = nodes.peekable();
        Self {
            iter,
            hint,
            prev_trim_after: None,
        }
    }
}

impl<'source> Iterator for EventIter<'source> {
    type Item = NodeEvent<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.iter.next();
        let peek = self.iter.peek();

        let first = self.prev_trim_after.is_none();

        // Trim the start of the current node.
        let start = if let Some(trim_after) = self.prev_trim_after.take() {
            trim_after
        } else {
            if let Some(hint) = self.hint.take() {
                hint.after
            } else {
                false
            }
        };

        // Trim the end of the current node.
        let mut end = false;
        if let Some(next) = peek {
            if next.trim().before {
                end = true;
            }
        }

        if let Some(ref current) = node {
            self.prev_trim_after = Some(current.trim().after);
        }

        let state = TrimState::from((start, end));

        node.map(|n| NodeEvent::new(n, state, first, peek.is_none()))
    }
}
