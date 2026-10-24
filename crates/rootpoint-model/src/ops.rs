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
    /// Insert, at index `at`, a slide whose root is one area holding an empty text.
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
    /// Split any node at `at`, a fraction of its extent along `axis`; the new area, an empty text,
    /// takes the part on `side`.
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
    /// Remove an asset that no area and no style of the table uses.
    RemoveAsset {
        id: AssetId,
    },
    RestoreAsset {
        id: AssetId,
        asset: Asset,
    },
    /// Replace an asset's path, as a move of its file does; the asset keeps its id, so the areas
    /// that name it are unchanged.
    SetAssetPath {
        id: AssetId,
        path: String,
    },
    /// Add a named style; the name is not empty and not in the table.
    AddStyle {
        name: String,
        style: TextStyle,
    },
    /// Replace a named style's values, which every text naming it then draws with.
    SetStyle {
        name: String,
        style: TextStyle,
    },
    /// Rename a style, and every text naming it with it. Refused while a text names `to`, which
    /// the inverse would rename back with the others.
    RenameStyle {
        from: String,
        to: String,
    },
    /// Remove a style, and give the texts naming it the replacement `into`: another style, or a
    /// copy of the removed values each; with none, they keep naming the removed style.
    RemoveStyle {
        name: String,
        into: Option<Replacement>,
    },
    /// Put a removed style back, and make `users` name it again. The inverse of `RemoveStyle`,
    /// which `into` repeats so that the inverse of this one removes it the same way.
    RestoreStyle {
        name: String,
        style: TextStyle,
        users: Vec<NodeId>,
        into: Option<Replacement>,
    },
}

/// What the texts naming a removed style name instead.
#[derive(Clone, Debug, PartialEq)]
pub enum Replacement {
    /// Another style of the table.
    Named(String),
    /// A copy of the removed style's values, in each text.
    Custom,
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
    /// A slide's root cannot be removed; to clear a slide, replace its tree or clear its area.
    RootNotRemovable(NodeId),
    IndexOutOfRange {
        index: usize,
        len: usize,
    },
    /// A split position or a divider position outside (0, 1), or out of order with its
    /// neighbours.
    PositionOutOfRange(f64),
    AssetInUse(AssetId),
    UnknownStyle(String),
    /// The name is taken, by a style of the table.
    StyleExists(String),
    /// A text names the style a rename would give its name to.
    StyleNamed {
        name: String,
        area: NodeId,
    },
    EmptyStyleName,
    /// A text a style's restoration names is not a text area.
    NotAText(NodeId),
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
            let content = new_text(doc.new_area_style());
            let root = Node::Area(Area {
                id: NodeId(doc.alloc()),
                content,
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
            let content = new_text(split_style(doc, node));
            let new_area = Area {
                id: NodeId(doc.alloc()),
                content,
            };
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
        // An image whose asset the table lacks is not refused: it is a cosmetic breach, and the
        // inverse of an edit of an area loaded with one brings it back,
        // `design@rootpoint-model@cosmetic-fallbacks`. The commands refuse to name a missing asset
        // instead, `design@rootpoint-protocol@no-new-breach`.
        Op::SetContent { area, content } => {
            let old = std::mem::replace(&mut area_mut(doc, area)?.content, content);
            Ok(Op::SetContent { area, content: old })
        }
        // An asset path that fails `asset_path_is_confined` is not refused either: it is a
        // cosmetic breach no store reads or writes, and an undo may bring back what a file was
        // loaded with, `design@rootpoint-model@cosmetic-fallbacks`. The commands refuse to name one,
        // `design@rootpoint-protocol@no-new-breach`.
        Op::AddAsset { asset } => {
            let id = AssetId(doc.alloc());
            doc.assets.insert(id, asset);
            Ok(Op::RemoveAsset { id })
        }
        Op::RemoveAsset { id } => {
            if !doc.assets.contains_key(&id) {
                return Err(OpError::UnknownAsset(id));
            }
            // A removal is refused, not a reference, and no undo meets it,
            // `design@rootpoint-model@cosmetic-fallbacks`.
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
        Op::SetAssetPath { id, path } => {
            let asset = doc.assets.get_mut(&id).ok_or(OpError::UnknownAsset(id))?;
            let old = std::mem::replace(&mut asset.path, path);
            Ok(Op::SetAssetPath { id, path: old })
        }
        Op::AddStyle { name, style } => {
            free_name(doc, &name)?;
            doc.styles.insert(name.clone(), style);
            Ok(Op::RemoveStyle { name, into: None })
        }
        Op::SetStyle { name, style } => {
            let slot = doc
                .styles
                .get_mut(&name)
                .ok_or_else(|| OpError::UnknownStyle(name.clone()))?;
            let old = std::mem::replace(slot, style);
            Ok(Op::SetStyle { name, style: old })
        }
        // A text naming `to`, a style the table lacks, is refused: the inverse renames every text
        // naming `to` back, so it would catch that text too. No undo meets the refusal, since
        // after a rename from X no text names X.
        Op::RenameStyle { from, to } => {
            free_name(doc, &to)?;
            if let Some(&area) = users(doc, &to).first() {
                return Err(OpError::StyleNamed { name: to, area });
            }
            let style = doc
                .styles
                .remove(&from)
                .ok_or_else(|| OpError::UnknownStyle(from.clone()))?;
            doc.styles.insert(to.clone(), style);
            for id in users(doc, &from) {
                set_choice(doc, id, StyleChoice::Named(to.clone()))?;
            }
            Ok(Op::RenameStyle { from: to, to: from })
        }
        // A replacement naming a style the table lacks is accepted, as every reference to a missing
        // style is but the one a rename refuses above: the commands refuse it,
        // `design@rootpoint-protocol@no-new-breach`.
        Op::RemoveStyle { name, into } => {
            let style = doc
                .styles
                .remove(&name)
                .ok_or_else(|| OpError::UnknownStyle(name.clone()))?;
            let users = users(doc, &name);
            let replacement = match &into {
                None => None,
                Some(Replacement::Named(other)) => Some(StyleChoice::Named(other.clone())),
                Some(Replacement::Custom) => Some(StyleChoice::Custom(style.clone())),
            };
            if let Some(choice) = replacement {
                for &id in &users {
                    set_choice(doc, id, choice.clone())?;
                }
            }
            Ok(Op::RestoreStyle {
                name,
                style,
                users,
                into,
            })
        }
        Op::RestoreStyle {
            name,
            style,
            users,
            into,
        } => {
            free_name(doc, &name)?;
            doc.styles.insert(name.clone(), style);
            for id in users {
                set_choice(doc, id, StyleChoice::Named(name.clone()))?;
            }
            Ok(Op::RemoveStyle { name, into })
        }
    }
}

/// Refuse a style name that is empty or in the table.
fn free_name(doc: &Document, name: &str) -> Result<(), OpError> {
    if name.is_empty() {
        Err(OpError::EmptyStyleName)
    } else if doc.styles.contains_key(name) {
        Err(OpError::StyleExists(name.to_owned()))
    } else {
        Ok(())
    }
}

/// The text areas that name the style `name`, in the order of the slides and of their trees.
fn users(doc: &Document, name: &str) -> Vec<NodeId> {
    doc.slides
        .iter()
        .flat_map(|s| s.root.areas())
        .filter(|a| {
            matches!(&a.content, Content::Text(TextContent { style: StyleChoice::Named(n), .. }) if n == name)
        })
        .map(|a| a.id)
        .collect()
}

/// Give the text of the area `id` the style `choice`.
fn set_choice(doc: &mut Document, id: NodeId, choice: StyleChoice) -> Result<(), OpError> {
    match &mut area_mut(doc, id)?.content {
        Content::Text(text) => {
            text.style = choice;
            Ok(())
        }
        Content::Image(_) => Err(OpError::NotAText(id)),
    }
}

/// An empty text in the style `style`: the content of a new area.
fn new_text(style: StyleChoice) -> Content {
    Content::Text(TextContent {
        text: String::new(),
        style,
    })
}

/// The style of the new area of a split of `node`: a text area's own style, so that splitting a body
/// text gives a body text, `design@rootpoint-model@new-area-style`; otherwise the style
/// `Document::new_area_style` gives. A name the table lacks is not copied, so that no split spreads
/// a breach.
fn split_style(doc: &Document, node: NodeId) -> StyleChoice {
    match doc.node(node) {
        Some(Node::Area(Area {
            content: Content::Text(text),
            ..
        })) => match &text.style {
            StyleChoice::Named(name) if !doc.styles.contains_key(name) => doc.new_area_style(),
            choice => choice.clone(),
        },
        _ => doc.new_area_style(),
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

/// Whether an area or a style of the table uses the asset: an image, a text's own font, or a named
/// style's font.
fn asset_in_use(doc: &Document, id: AssetId) -> bool {
    let font = FontRef::Asset(id);
    doc.styles.values().any(|s| s.font == font)
        || doc
            .slides
            .iter()
            .flat_map(|s| s.root.areas())
            .any(|a| match &a.content {
                Content::Image(image) => image.asset == id,
                Content::Text(TextContent {
                    style: StyleChoice::Custom(style),
                    ..
                }) => style.font == font,
                Content::Text(_) => false,
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

fn empty_area(area: &Area) -> Node {
    Node::Area(area.clone())
}

/// Split the node `target` of the tree rooted at `root`. The caller has checked that the node is in
/// this tree and that `at` is in (0, 1).
fn split(
    root: &mut Node,
    target: NodeId,
    axis: Axis,
    at: f64,
    side: Side,
    new_area: Area,
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
                s.children.push(empty_area(&new_area));
            }
            Side::Before => {
                s.dividers
                    .iter_mut()
                    .for_each(|d| *d = at + *d * (1.0 - at));
                s.dividers.insert(0, at);
                s.children.insert(0, empty_area(&new_area));
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
        parent.children.insert(index, empty_area(&new_area));
        parent.dividers.insert(i, pos);
        return;
    }
    // Otherwise the node is wrapped in a new split of `axis`.
    let node = find_in(root, target).expect("the caller checked the node is in this tree");
    let old = std::mem::replace(node, empty_area(&new_area));
    let children = match side {
        Side::After => vec![old, empty_area(&new_area)],
        Side::Before => vec![empty_area(&new_area), old],
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
