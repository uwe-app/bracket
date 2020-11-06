use crate::{
    SyntaxResult,
    parser::{Parser, ast::Node, trim::TrimState},
};

/// Event for node iterators that also 
/// encapsulates the whitespace trim state.
#[derive(Debug)]
pub struct NodeEvent<'a>(pub &'a Node<'a>, pub TrimState);

/// Iterate nodes yielding children for documents but does
/// not descend into block nodes.
pub struct NodeIter<'source> {
    node: &'source Node<'source>,
    complete: bool,
    children: Option<std::slice::Iter<'source, Node<'source>>>,
}

impl<'source> NodeIter<'source> {
    pub fn new(node: &'source Node, trim: TrimState) -> Self {
        Self {
            node,
            complete: false,
            children: None,
        } 
    }

    pub fn trim(self) -> TrimIter<'source> {
        TrimIter::new(self, Default::default()) 
    }
}

impl<'source> Iterator for NodeIter<'source> {
    type Item = &'source Node<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.complete {
            return None
        }

        let (node, completed) = match *self.node {
            Node::Document(ref doc) => {
                let it = self.children.get_or_insert(doc.nodes().iter());
                let child = it.next();
                if child.is_none() {
                    self.children.take();
                }
                (child, child.is_none())
            }
            Node::Block(_) => (Some(self.node), true),
            Node::Text(_) => (Some(self.node), true),
            Node::Statement(_) => (Some(self.node), true),
            Node::RawBlock(_)
            | Node::RawStatement(_)
            | Node::RawComment(_)
            | Node::Comment(_) => (Some(self.node), true),
            Node::Condition(_) => (None, true),
        };

        self.complete = completed;
        node
    }
}

/// Iterator that yields nodes with trim flags that indicate 
/// whether the current node should have leading and trailing 
/// whitespace removed.
pub struct TrimIter<'source> {
    iter: std::iter::Peekable<NodeIter<'source>>,
    prev_trim_after: Option<bool>,
    trim: TrimState,
}

impl<'source> TrimIter<'source> {
    pub fn new(nodes: NodeIter<'source>, trim: TrimState) -> Self {
        let iter = nodes.peekable();
        Self {
            iter,
            trim,
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
        let start = self.prev_trim_after.is_some()
            && self.prev_trim_after.take().unwrap();

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

        node.map(|n| NodeEvent(n, state))
    }
}

