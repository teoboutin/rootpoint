//! The operations that change a document, and their inverses.
//!
//! Every change of a document is an [`Op`] given to [`apply`], which returns the operation that
//! undoes it. The GUI, the protocol and the tests change a document through nothing else.

use crate::document::*;
use crate::validate::Invalid;

/// Which part of a split node the new area takes: the part before the split position, or after.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Before,
    After,
}

/// A change of a document.
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    /// Insert, at index `at`, a slide whose root is one empty area.
    AddSlide {
        at: usize,
    },
    RemoveSlide {
        slide: SlideId,
    },
    /// Insert a removed slide back, whole, at index `at`.
    RestoreSlide {
        at: usize,
        slide: Slide,
    },
    MoveSlide {
        slide: SlideId,
        to: usize,
    },
    /// Insert, after the slide, a copy whose every node has a new id.
    DuplicateSlide {
        slide: SlideId,
    },
    /// Split any node at `at`, a fraction of its extent along `axis`; the new empty area takes the
    /// part on `side`.
    Split {
        node: NodeId,
        axis: Axis,
        at: f64,
        side: Side,
    },
    /// Remove any node but a slide's root; a neighbour takes its extent.
    Remove {
        node: NodeId,
    },
    /// Replace a slide's whole tree. The inverse of the structural edits: a snapshot of the former
    /// tree restores merges, collapses and divider positions exactly, which an inverse per node
    /// would have to reconstruct, and slides are small.
    SetTree {
        slide: SlideId,
        root: Node,
    },
    MoveDivider {
        split: NodeId,
        index: usize,
        pos: f64,
    },
    /// Exchange the contents of two areas, which keep their ids and their places.
    Swap {
        a: NodeId,
        b: NodeId,
    },
    SetContent {
        area: NodeId,
        content: Content,
    },
    AddAsset {
        asset: Asset,
    },
    /// Remove an asset that no area uses.
    RemoveAsset {
        id: AssetId,
    },
    RestoreAsset {
        id: AssetId,
        asset: Asset,
    },
}

/// Why an operation was refused. A refused operation leaves the document unchanged.
#[derive(Clone, Debug, PartialEq)]
pub enum OpError {
    UnknownSlide(SlideId),
    UnknownNode(NodeId),
    UnknownAsset(AssetId),
    /// The id is taken, by the slide or asset an operation would restore.
    IdTaken(u64),
    NotAnArea(NodeId),
    NotASplit(NodeId),
    /// A slide's root cannot be removed; to clear a slide, replace its tree or empty its area.
    RootNotRemovable(NodeId),
    IndexOutOfRange {
        index: usize,
        len: usize,
    },
    /// A split position or a divider position outside (0, 1), or out of order with its
    /// neighbours.
    PositionOutOfRange(f64),
    AssetInUse(AssetId),
    /// The result would break an invariant of the document, which the reason names.
    Invalid(String),
}

/// Apply `op` to `doc`, and return the operation that undoes it. On an error the document is
/// unchanged.
pub fn apply(doc: &mut Document, op: Op) -> Result<Op, OpError> {
    // Work on a copy, so that a refusal found halfway leaves `doc` as it was. Documents are small.
    let mut work = doc.clone();
    let inverse = apply_in(&mut work, op)?;
    // An operation whose result breaks an invariant is refused, per
    // `design@rootpoint-model@validated-documents`: a restored slide or asset whose
    // ids collide, a tree given whole, or a split whose divider positions rounding has made equal
    // or has pushed to 0 or 1.
    work.validate()
        .map_err(|Invalid(reason)| OpError::Invalid(reason))?;
    *doc = work;
    Ok(inverse)
}

fn apply_in(doc: &mut Document, op: Op) -> Result<Op, OpError> {
    match op {
        Op::AddSlide { at } => {
            check_index(at, doc.slides.len() + 1)?;
            let slide = SlideId(doc.alloc());
            let root = Node::Area(Area {
                id: NodeId(doc.alloc()),
                content: Content::Empty,
            });
            doc.slides.insert(at, Slide { id: slide, root });
            Ok(Op::RemoveSlide { slide })
        }
        Op::RemoveSlide { slide } => {
            let at = slide_index(doc, slide)?;
            let slide = doc.slides.remove(at);
            Ok(Op::RestoreSlide { at, slide })
        }
        Op::RestoreSlide { at, slide } => {
            check_index(at, doc.slides.len() + 1)?;
            if doc.slide(slide.id).is_some() {
                return Err(OpError::IdTaken(slide.id.0));
            }
            let id = slide.id;
            doc.slides.insert(at, slide);
            Ok(Op::RemoveSlide { slide: id })
        }
        Op::MoveSlide { slide, to } => {
            let from = slide_index(doc, slide)?;
            check_index(to, doc.slides.len())?;
            let s = doc.slides.remove(from);
            doc.slides.insert(to, s);
            Ok(Op::MoveSlide { slide, to: from })
        }
        Op::DuplicateSlide { slide } => {
            let at = slide_index(doc, slide)?;
            let mut root = doc.slides[at].root.clone();
            renumber(&mut root, doc);
            let copy = SlideId(doc.alloc());
            doc.slides.insert(at + 1, Slide { id: copy, root });
            Ok(Op::RemoveSlide { slide: copy })
        }
        Op::Split {
            node,
            axis,
            at,
            side,
        } => {
            check_position(at)?;
            let s = slide_of(doc, node)?;
            let before = doc.slides[s].root.clone();
            let new_area = NodeId(doc.alloc());
            let new_split = NodeId(doc.alloc());
            let root = &mut doc.slides[s].root;
            split(root, node, axis, at, side, new_area, new_split);
            Ok(Op::SetTree {
                slide: doc.slides[s].id,
                root: before,
            })
        }
        Op::Remove { node } => {
            let s = slide_of(doc, node)?;
            if doc.slides[s].root.id() == node {
                return Err(OpError::RootNotRemovable(node));
            }
            let before = doc.slides[s].root.clone();
            remove(&mut doc.slides[s].root, node);
            Ok(Op::SetTree {
                slide: doc.slides[s].id,
                root: before,
            })
        }
        Op::SetTree { slide, root } => {
            let s = slide_index(doc, slide)?;
            let before = std::mem::replace(&mut doc.slides[s].root, root);
            Ok(Op::SetTree {
                slide,
                root: before,
            })
        }
        Op::MoveDivider { split, index, pos } => {
            let node = find_mut(doc, split)?;
            let Node::Split(sp) = node else {
                return Err(OpError::NotASplit(split));
            };
            check_index(index, sp.dividers.len())?;
            let low = if index == 0 {
                0.0
            } else {
                sp.dividers[index - 1]
            };
            let high = sp.dividers.get(index + 1).copied().unwrap_or(1.0);
            if !(pos > low && pos < high) {
                return Err(OpError::PositionOutOfRange(pos));
            }
            let old = std::mem::replace(&mut sp.dividers[index], pos);
            Ok(Op::MoveDivider {
                split,
                index,
                pos: old,
            })
        }
        Op::Swap { a, b } => {
            let content_a = area_mut(doc, a)?.content.clone();
            let content_b = std::mem::replace(&mut area_mut(doc, b)?.content, content_a);
            area_mut(doc, a)?.content = content_b;
            Ok(Op::Swap { a, b })
        }
        Op::SetContent { area, content } => {
            if let Content::Image(image) = &content
                && !doc.assets.contains_key(&image.asset)
            {
                return Err(OpError::UnknownAsset(image.asset));
            }
            let old = std::mem::replace(&mut area_mut(doc, area)?.content, content);
            Ok(Op::SetContent { area, content: old })
        }
        Op::AddAsset { asset } => {
            let id = AssetId(doc.alloc());
            doc.assets.insert(id, asset);
            Ok(Op::RemoveAsset { id })
        }
        Op::RemoveAsset { id } => {
            if !doc.assets.contains_key(&id) {
                return Err(OpError::UnknownAsset(id));
            }
            if asset_in_use(doc, id) {
                return Err(OpError::AssetInUse(id));
            }
            let asset = doc.assets.remove(&id).expect("checked above");
            Ok(Op::RestoreAsset { id, asset })
        }
        Op::RestoreAsset { id, asset } => {
            if doc.assets.contains_key(&id) {
                return Err(OpError::IdTaken(id.0));
            }
            doc.assets.insert(id, asset);
            Ok(Op::RemoveAsset { id })
        }
    }
}

fn check_index(index: usize, len: usize) -> Result<(), OpError> {
    if index < len {
        Ok(())
    } else {
        Err(OpError::IndexOutOfRange { index, len })
    }
}

fn check_position(pos: f64) -> Result<(), OpError> {
    if pos > 0.0 && pos < 1.0 {
        Ok(())
    } else {
        Err(OpError::PositionOutOfRange(pos))
    }
}

fn slide_index(doc: &Document, slide: SlideId) -> Result<usize, OpError> {
    doc.slides
        .iter()
        .position(|s| s.id == slide)
        .ok_or(OpError::UnknownSlide(slide))
}

/// The index of the slide that holds the node.
fn slide_of(doc: &Document, node: NodeId) -> Result<usize, OpError> {
    doc.slides
        .iter()
        .position(|s| s.root.find(node).is_some())
        .ok_or(OpError::UnknownNode(node))
}

fn find_mut(doc: &mut Document, id: NodeId) -> Result<&mut Node, OpError> {
    doc.slides
        .iter_mut()
        .find_map(|s| find_in(&mut s.root, id))
        .ok_or(OpError::UnknownNode(id))
}

fn find_in(node: &mut Node, id: NodeId) -> Option<&mut Node> {
    if node.id() == id {
        return Some(node);
    }
    match node {
        Node::Split(s) => s.children.iter_mut().find_map(|c| find_in(c, id)),
        Node::Area(_) => None,
    }
}

fn area_mut(doc: &mut Document, id: NodeId) -> Result<&mut Area, OpError> {
    match find_mut(doc, id)? {
        Node::Area(a) => Ok(a),
        Node::Split(_) => Err(OpError::NotAnArea(id)),
    }
}

fn asset_in_use(doc: &Document, id: AssetId) -> bool {
    doc.slides
        .iter()
        .flat_map(|s| s.root.areas())
        .any(|a| match &a.content {
            Content::Image(image) => image.asset == id,
            Content::Text(text) => text.style.font == FontRef::Asset(id),
            Content::Empty => false,
        })
}

/// Give every node of the subtree a new id.
fn renumber(node: &mut Node, doc: &mut Document) {
    match node {
        Node::Split(s) => {
            s.id = NodeId(doc.alloc());
            for child in &mut s.children {
                renumber(child, doc);
            }
        }
        Node::Area(a) => a.id = NodeId(doc.alloc()),
    }
}

fn empty_area(id: NodeId) -> Node {
    Node::Area(Area {
        id,
        content: Content::Empty,
    })
}

/// Split the node `target` of the tree rooted at `root`. The caller has checked that the node is in
/// this tree and that `at` is in (0, 1).
fn split(
    root: &mut Node,
    target: NodeId,
    axis: Axis,
    at: f64,
    side: Side,
    new_area: NodeId,
    new_split: NodeId,
) {
    // The node is itself a split of `axis`: the new area joins it, first or last.
    if let Some(Node::Split(s)) = find_in(root, target)
        && s.axis == axis
    {
        match side {
            Side::After => {
                s.dividers.iter_mut().for_each(|d| *d *= at);
                s.dividers.push(at);
                s.children.push(empty_area(new_area));
            }
            Side::Before => {
                s.dividers
                    .iter_mut()
                    .for_each(|d| *d = at + *d * (1.0 - at));
                s.dividers.insert(0, at);
                s.children.insert(0, empty_area(new_area));
            }
        }
        return;
    }
    // The node's parent splits along `axis`: the new area is inserted beside the node, and the
    // parent gains one divider.
    if let Some((parent, i)) = parent_of(root, target)
        && parent.axis == axis
    {
        let (start, end) = parent.child_extent(i);
        let pos = start + at * (end - start);
        let index = match side {
            Side::After => i + 1,
            Side::Before => i,
        };
        parent.children.insert(index, empty_area(new_area));
        parent.dividers.insert(i, pos);
        return;
    }
    // Otherwise the node is wrapped in a new split of `axis`.
    let node = find_in(root, target).expect("the caller checked the node is in this tree");
    let old = std::mem::replace(node, empty_area(new_area));
    let children = match side {
        Side::After => vec![old, empty_area(new_area)],
        Side::Before => vec![empty_area(new_area), old],
    };
    *node = Node::Split(Split {
        id: new_split,
        axis,
        children,
        dividers: vec![at],
    });
}

/// The split holding `target` as a child, and the child's index in it.
fn parent_of(node: &mut Node, target: NodeId) -> Option<(&mut Split, usize)> {
    let Node::Split(s) = node else {
        return None;
    };
    if let Some(i) = s.children.iter().position(|c| c.id() == target) {
        return Some((s, i));
    }
    s.children.iter_mut().find_map(|c| parent_of(c, target))
}

/// Remove the node `target`, which is not the root, from the tree rooted at `root`.
fn remove(root: &mut Node, target: NodeId) {
    let (parent, i) = parent_of(root, target).expect("the caller checked the node is not the root");
    // The sibling before takes the extent, or the sibling after for a first child.
    parent.dividers.remove(i.saturating_sub(1));
    parent.children.remove(i);
    if parent.children.len() > 1 {
        return;
    }
    let parent_id = parent.id;
    let only = parent
        .children
        .pop()
        .expect("a split had two children or more");
    // The split left with one child is replaced by it; a child split of its new parent's axis is
    // merged into that parent, so no split has a child of its own axis.
    match parent_of(root, parent_id) {
        Some((grand, j)) => match only {
            Node::Split(child) if child.axis == grand.axis => {
                let (start, end) = grand.child_extent(j);
                let scaled = child.dividers.iter().map(|d| start + d * (end - start));
                grand.dividers.splice(j..j, scaled);
                grand.children.splice(j..=j, child.children);
            }
            only => grand.children[j] = only,
        },
        None => *root = only,
    }
}
