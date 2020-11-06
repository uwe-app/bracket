use crate::{
    parser::{
        ast::Node,
        trim::{TrimHint, TrimState},
        Parser,
    },
    SyntaxResult,
};

/// Event for node iterators that also
/// encapsulates the whitespace trim state.
#[derive(Debug)]
pub struct NodeEvent<'a> {
    pub node: &'a Node<'a>,
    pub trim: TrimState,
}

impl<'a> NodeEvent<'a> {
    pub fn new(node: &'a Node, trim: TrimState) -> Self {
        Self { node, trim }
    }
}

/// Iterate nodes yielding children for documents but does
/// not descend into block nodes.
pub struct NodeIter<'source> {
    node: &'source Node<'source>,
}

impl<'source> NodeIter<'source> {
    pub fn new(node: &'source Node) -> Self {
        Self { node }
    }

    /// Create an iterator that adds trim state information
    /// to each node.
    ///
    /// The hint can be used to determine the start trim information
    /// for the first node.
    pub fn trim(self, hint: Option<TrimHint>) -> TrimIter<'source> {
        TrimIter::new(Box::new(self), hint)
    }
}

impl<'source> Iterator for NodeIter<'source> {
    type Item = &'source Node<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        match *self.node {
            Node::Document(_) => Some(self.node),
            Node::Block(_) => Some(self.node),
            Node::Text(_) => Some(self.node),
            Node::Statement(_) => Some(self.node),
            Node::RawBlock(_)
            | Node::RawStatement(_)
            | Node::RawComment(_)
            | Node::Comment(_) => Some(self.node),
            Node::Condition(_) => None,
        }
    }
}

/// Iterate documents and blocks.
pub struct BlockIter<'source> {
    node: &'source Node<'source>,
    children: Option<std::slice::Iter<'source, Node<'source>>>,
}

impl<'source> BlockIter<'source> {
    pub fn new(node: &'source Node) -> Self {
        Self {
            node,
            children: None,
        }
    }

    /// Create an iterator that adds trim state information
    /// to each node.
    ///
    /// The hint can be used to determine the start trim information
    /// for the first node.
    pub fn trim(self, hint: Option<TrimHint>) -> TrimIter<'source> {
        TrimIter::new(Box::new(self), hint)
    }
}

impl<'source> Iterator for BlockIter<'source> {
    type Item = &'source Node<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = match *self.node {
            Node::Document(ref doc) => {
                let it = self.children.get_or_insert(doc.nodes().iter());
                let child = it.next();
                if child.is_none() {
                    self.children.take();
                }
                child
            }
            Node::Block(ref block) => {
                let it = self.children.get_or_insert(block.nodes().iter());
                let child = it.next();
                if child.is_none() {
                    self.children.take();
                }
                child
            }
            Node::Text(_) => None,
            Node::Statement(_) => None,
            Node::RawBlock(_)
            | Node::RawStatement(_)
            | Node::RawComment(_)
            | Node::Comment(_) => None,
            Node::Condition(_) => None,
        };

        node
    }
}

/// Iterator that yields nodes with trim flags that indicate
/// whether the current node should have leading and trailing
/// whitespace removed.
pub struct TrimIter<'source> {
    iter: std::iter::Peekable<
        Box<dyn Iterator<Item = &'source Node<'source>> + 'source>,
    >,
    prev_trim_after: Option<bool>,
    hint: Option<TrimHint>,
}

impl<'source> TrimIter<'source> {
    pub fn new(
        nodes: Box<dyn Iterator<Item = &'source Node<'source>> + 'source>,
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

impl<'source> Iterator for TrimIter<'source> {
    type Item = NodeEvent<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.iter.next();
        let peek = self.iter.peek();

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

        node.map(|n| NodeEvent::new(n, state))
    }
}