//! The selection's place in the slide's tree: the path from the root to it, which the breadcrumb
//! shows, and the node each move of Alt and an arrow selects.

use std::collections::HashMap;

use rootpoint_model::{Node, NodeId, Slide};

/// A move of the selection through the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Parent,
    Child,
    Previous,
    Next,
}

/// The nodes from the slide's root to `node`, both included, or `None` when `node` is not in the
/// slide.
pub fn path(slide: &Slide, node: NodeId) -> Option<Vec<&Node>> {
    fn walk<'a>(at: &'a Node, node: NodeId, out: &mut Vec<&'a Node>) -> bool {
        out.push(at);
        if at.id() == node {
            return true;
        }
        if let Node::Split(s) = at
            && s.children.iter().any(|c| walk(c, node, out))
        {
            return true;
        }
        out.pop();
        false
    }
    let mut out = Vec::new();
    walk(&slide.root, node, &mut out).then_some(out)
}

/// For each split, the child last selected in it, so that a move to a child returns to it. It
/// only chooses the node a `select` names.
#[derive(Default)]
pub struct Memory {
    last: HashMap<NodeId, NodeId>,
}

impl Memory {
    /// Note that `node` is selected: it is the last selected child of each split above it.
    pub fn note(&mut self, slide: &Slide, node: NodeId) {
        if let Some(path) = path(slide, node) {
            for pair in path.windows(2) {
                self.last.insert(pair[0].id(), pair[1].id());
            }
        }
    }

    /// The node a move from `node` selects: nothing at the root for `Parent`, on an area for
    /// `Child`, at the first child for `Previous` or at the last for `Next`.
    pub fn target(&self, slide: &Slide, node: NodeId, m: Move) -> Option<NodeId> {
        let path = path(slide, node)?;
        let here = *path.last()?;
        let parent = path.len().checked_sub(2).map(|i| path[i]);
        match m {
            Move::Parent => parent.map(Node::id),
            Move::Child => {
                let Node::Split(s) = here else {
                    return None;
                };
                let last = self.last.get(&s.id);
                let child = s
                    .children
                    .iter()
                    .find(|c| Some(&c.id()) == last)
                    .or(s.children.first())?;
                Some(child.id())
            }
            Move::Previous | Move::Next => {
                let Some(Node::Split(s)) = parent else {
                    return None;
                };
                let i = s.children.iter().position(|c| c.id() == node)?;
                let j = match m {
                    Move::Previous => i.checked_sub(1)?,
                    _ => i + 1,
                };
                s.children.get(j).map(Node::id)
            }
        }
    }
}
