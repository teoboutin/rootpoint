//! The operations that change a document, and their inverses.
//!
//! Every change of a document is an [`Op`] given to [`apply`], which returns the operation that
//! undoes it. The GUI, the protocol and the tests change a document through nothing else.

use rootpoint_math::Formula;

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
    /// Insert, after the slide, a copy whose every node has a new id; a reference of the copy to an
    /// area of the slide names that area's copy.
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
    /// Exchange the contents of two areas, which keep their ids and their places; every reference
    /// to either area follows its content to the other.
    Swap {
        a: NodeId,
        b: NodeId,
    },
    SetContent {
        area: NodeId,
        content: Content,
    },
    /// Set the content of each area, as the inverse of an operation that changes several texts.
    SetContents {
        contents: Vec<(NodeId, Content)>,
    },
    AddAsset {
        asset: Asset,
    },
    /// Remove an asset that no area, no span and no style of either table uses.
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
    /// Insert characters in a text node of an area's text, at an offset.
    InsertText {
        area: NodeId,
        node: TextNodeRef,
        offset: usize,
        text: String,
    },
    /// Insert a formula, one item, in a text node of an area's text, at an offset.
    InsertMath {
        area: NodeId,
        node: TextNodeRef,
        offset: usize,
        formula: Formula,
    },
    /// Insert a reference to the area `target`, one item, in a text node, at an offset.
    InsertRef {
        area: NodeId,
        node: TextNodeRef,
        offset: usize,
        target: NodeId,
    },
    /// Replace the formula at an offset of a text node.
    SetMath {
        area: NodeId,
        node: TextNodeRef,
        offset: usize,
        formula: Formula,
    },
    /// Make every reference to the area `from` name `to`. Its inverse sets each text it changed
    /// back, so it moves back only the references it moved.
    RetargetReferences {
        from: NodeId,
        to: NodeId,
    },
    /// Remove a range of offsets of a text node; a span in the range goes whole.
    DeleteText {
        area: NodeId,
        node: TextNodeRef,
        range: std::ops::Range<usize>,
    },
    /// Put a range of offsets of a text node into a new span, whose id the counter gives; an empty
    /// range opens an empty span.
    WrapSpan {
        area: NodeId,
        node: TextNodeRef,
        range: std::ops::Range<usize>,
        style: SpanChoice,
    },
    /// Replace a span by its content, in its parent.
    UnwrapSpan {
        area: NodeId,
        span: SpanId,
    },
    SetSpanStyle {
        area: NodeId,
        span: SpanId,
        style: SpanChoice,
    },
    /// Cut a span at an offset inside it into two spans of its style, the second with a new id.
    /// Refused at the span's edges, where one half would be empty.
    SplitSpan {
        area: NodeId,
        span: SpanId,
        offset: usize,
    },
    /// Replace an area's whole text tree, every span given a new id; the area keeps its style.
    SetText {
        area: NodeId,
        root: TextNode,
    },
    /// Add a named span style; the name is not empty and not in the span style table.
    AddSpanStyle {
        name: String,
        style: SpanStyle,
    },
    /// Replace a named span style's values, which every span naming it then draws with.
    SetSpanStyleValues {
        name: String,
        style: SpanStyle,
    },
    /// Rename a span style, and every span naming it with it. Refused while a span names `to`.
    RenameSpanStyle {
        from: String,
        to: String,
    },
    /// Remove a span style, and give the spans naming it the replacement `into`: another span
    /// style, or a copy of the removed values each; with none, they keep naming the removed style.
    RemoveSpanStyle {
        name: String,
        into: Option<Replacement>,
    },
    /// Put a removed span style back, and make `users` name it again: the inverse of
    /// `RemoveSpanStyle`, as `RestoreStyle` is of `RemoveStyle`.
    RestoreSpanStyle {
        name: String,
        style: SpanStyle,
        users: Vec<(NodeId, SpanId)>,
        into: Option<Replacement>,
    },
    /// Replace the whole document, as a reload of its file changed outside the editor does; the
    /// inverse holds the document replaced, so an undo brings the editor's own back.
    ReplaceDocument {
        document: Box<Document>,
    },
}

/// What the texts naming a removed style, or the spans naming a removed span style, name instead.
#[derive(Clone, Debug, PartialEq)]
pub enum Replacement {
    /// Another style of the table the removed style was in.
    Named(String),
    /// A copy of the removed style's values, in each text or span.
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
    /// The name is taken, by a style of the table the operation acts on.
    StyleExists(String),
    /// A span names the span style a rename would give its name to.
    SpanStyleNamed {
        name: String,
        span: SpanId,
    },
    /// A text names the style a rename would give its name to.
    StyleNamed {
        name: String,
        area: NodeId,
    },
    EmptyStyleName,
    /// The area a text operation, or a style's restoration, names is not a text area.
    NotAText(NodeId),
    /// A span the area's text lacks.
    UnknownSpan(SpanId),
    /// An offset, or the end of a range, past the end of its text node, or a range whose start is
    /// past its end.
    OffsetOutOfRange {
        offset: usize,
        len: usize,
    },
    /// A custom span style that sets no field: the span would change nothing, and its caller
    /// unwraps the span instead.
    EmptySpanStyle,
    /// A span cut at its edge, where one half would be empty.
    SpanEdge(SpanId),
    /// No formula stands at this offset of its text node.
    NotAFormula {
        offset: usize,
    },
    /// The result would break an invariant of the document, which the reason names.
    Invalid(String),
}

/// Apply `op` to `doc`, and return the operation that undoes it. On an error the document is
/// unchanged.
pub fn apply(doc: &mut Document, op: Op) -> Result<Op, OpError> {
    // Work on a copy, so that a refusal found halfway leaves `doc` as it was. Documents are small.
    let mut work = doc.clone();
    let inverse = apply_in(&mut work, op)?;
    // A text given whole, as by `Op::SetContent`, is brought to its canonical form, so that an
    // operation and its inverse give a document equal to the original.
    normalise_texts(&mut work);
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
            let mut copies = std::collections::BTreeMap::new();
            renumber(&mut root, doc, &mut copies);
            redirect(&mut root, &copies);
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
        // The references follow the contents, so a moved equation keeps them; the swap is its own
        // inverse, references included.
        Op::Swap { a, b } => {
            let content_a = area_mut(doc, a)?.content.clone();
            let content_b = std::mem::replace(&mut area_mut(doc, b)?.content, content_a);
            area_mut(doc, a)?.content = content_b;
            retarget_texts(doc, |id| {
                if id == a {
                    Some(b)
                } else if id == b {
                    Some(a)
                } else {
                    None
                }
            });
            Ok(Op::Swap { a, b })
        }
        Op::SetContents { contents } => {
            let mut before = Vec::with_capacity(contents.len());
            for (area, content) in contents {
                let old = std::mem::replace(&mut area_mut(doc, area)?.content, content);
                before.push((area, old));
            }
            before.reverse();
            Ok(Op::SetContents { contents: before })
        }
        Op::RetargetReferences { from, to } => {
            let before = retarget_texts(doc, |id| (id == from).then_some(to));
            Ok(Op::SetContents { contents: before })
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
        // The counter keeps the larger of the two, so that no id the editor handed out, as to a
        // script, is given again: the ids only grow, `design@rootpoint-model@stable-ids`.
        Op::ReplaceDocument { document } => {
            let next_id = doc.next_id.max(document.next_id);
            let old = std::mem::replace(doc, *document);
            doc.next_id = next_id;
            Ok(Op::ReplaceDocument {
                document: Box::new(old),
            })
        }
        // Every edit of a text tree is undone by the area's former content, as `Op::SetTree` undoes
        // a slide's structural edits: a snapshot brings back every span with its id, and a text is
        // small.
        Op::InsertText {
            area,
            node,
            offset,
            text,
        } => edit_text(doc, area, |_, root| {
            let mut seq = Seq::of(text_node(root, node)?);
            seq.check(offset)?;
            let chars = text.chars().map(Elem::Char);
            seq.0.splice(offset..offset, chars);
            seq.store(text_node(root, node)?);
            Ok(())
        }),
        Op::InsertMath {
            area,
            node,
            offset,
            formula,
        } => insert_item(doc, area, node, offset, Elem::Math(formula)),
        Op::InsertRef {
            area,
            node,
            offset,
            target,
        } => insert_item(doc, area, node, offset, Elem::Ref(target)),
        Op::SetMath {
            area,
            node,
            offset,
            formula,
        } => edit_text(doc, area, |_, root| {
            let mut seq = Seq::of(text_node(root, node)?);
            match seq.0.get_mut(offset) {
                Some(Elem::Math(f)) => *f = formula,
                _ => return Err(OpError::NotAFormula { offset }),
            }
            seq.store(text_node(root, node)?);
            Ok(())
        }),
        Op::DeleteText { area, node, range } => edit_text(doc, area, |_, root| {
            let mut seq = Seq::of(text_node(root, node)?);
            seq.check_range(&range)?;
            seq.0.drain(range);
            seq.store(text_node(root, node)?);
            Ok(())
        }),
        Op::WrapSpan {
            area,
            node,
            range,
            style,
        } => edit_text(doc, area, |doc, root| {
            check_span_style(&style)?;
            let mut seq = Seq::of(text_node(root, node)?);
            seq.check_range(&range)?;
            let inner: Vec<Elem> = seq.0.drain(range.clone()).collect();
            let mut text = TextNode::default();
            Seq(inner).store(&mut text);
            let span = Span {
                id: SpanId(doc.alloc()),
                style,
                text,
            };
            seq.0.insert(range.start, Elem::Span(span));
            seq.store(text_node(root, node)?);
            Ok(())
        }),
        Op::UnwrapSpan { area, span } => edit_text(doc, area, |_, root| {
            let (parent, at) = span_parent(root, span)?;
            let mut seq = Seq::of(parent);
            let Elem::Span(removed) = seq.0.remove(at) else {
                unreachable!("span_parent gives the index of the span")
            };
            let inner = Seq::of(&removed.text).0;
            seq.0.splice(at..at, inner);
            seq.store(parent);
            Ok(())
        }),
        Op::SetSpanStyle { area, span, style } => edit_text(doc, area, |_, root| {
            check_span_style(&style)?;
            let (parent, at) = span_parent(root, span)?;
            let mut seq = Seq::of(parent);
            if let Elem::Span(s) = &mut seq.0[at] {
                s.style = style;
            }
            seq.store(parent);
            Ok(())
        }),
        Op::SplitSpan { area, span, offset } => edit_text(doc, area, |doc, root| {
            let (parent, at) = span_parent(root, span)?;
            let mut seq = Seq::of(parent);
            let Elem::Span(first) = &mut seq.0[at] else {
                unreachable!("span_parent gives the index of the span")
            };
            // The second half takes the span's style, so a custom style that sets nothing, as a
            // file can hold, would make a span that sets nothing.
            check_span_style(&first.style)?;
            let mut inner = Seq::of(&first.text);
            inner.check(offset)?;
            if offset == 0 || offset == inner.0.len() {
                return Err(OpError::SpanEdge(span));
            }
            let tail = Seq(inner.0.split_off(offset));
            inner.store(&mut first.text);
            let mut text = TextNode::default();
            tail.store(&mut text);
            let second = Span {
                id: SpanId(doc.alloc()),
                style: first.style.clone(),
                text,
            };
            seq.0.insert(at + 1, Elem::Span(second));
            seq.store(parent);
            Ok(())
        }),
        Op::SetText { area, root: new } => edit_text(doc, area, |doc, root| {
            for span in new.spans() {
                check_span_style(&span.style)?;
            }
            *root = new;
            crate::ops::renumber_spans(root, doc);
            Ok(())
        }),
        Op::AddSpanStyle { name, style } => {
            free_span_style_name(doc, &name)?;
            doc.span_styles.insert(name.clone(), style);
            Ok(Op::RemoveSpanStyle { name, into: None })
        }
        Op::SetSpanStyleValues { name, style } => {
            let slot = doc
                .span_styles
                .get_mut(&name)
                .ok_or_else(|| OpError::UnknownStyle(name.clone()))?;
            let old = std::mem::replace(slot, style);
            Ok(Op::SetSpanStyleValues { name, style: old })
        }
        // As `RenameStyle`: a span naming `to` is refused, since the inverse would rename it back
        // with the others.
        Op::RenameSpanStyle { from, to } => {
            free_span_style_name(doc, &to)?;
            if let Some(&(_, span)) = span_style_users(doc, &to).first() {
                return Err(OpError::SpanStyleNamed { name: to, span });
            }
            let style = doc
                .span_styles
                .remove(&from)
                .ok_or_else(|| OpError::UnknownStyle(from.clone()))?;
            doc.span_styles.insert(to.clone(), style);
            for (area, span) in span_style_users(doc, &from) {
                set_span_choice(doc, area, span, SpanChoice::Named(to.clone()))?;
            }
            Ok(Op::RenameSpanStyle { from: to, to: from })
        }
        Op::RemoveSpanStyle { name, into } => {
            let style = doc
                .span_styles
                .remove(&name)
                .ok_or_else(|| OpError::UnknownStyle(name.clone()))?;
            let users = span_style_users(doc, &name);
            let replacement = match &into {
                None => None,
                Some(Replacement::Named(other)) => Some(SpanChoice::Named(other.clone())),
                Some(Replacement::Custom) => Some(SpanChoice::Custom(style.clone())),
            };
            // A copy of a span style that sets nothing would make its users custom spans that set
            // nothing, which no operation makes.
            if let Some(choice) = &replacement
                && !users.is_empty()
            {
                check_span_style(choice)?;
            }
            if let Some(choice) = replacement {
                for &(area, span) in &users {
                    set_span_choice(doc, area, span, choice.clone())?;
                }
            }
            Ok(Op::RestoreSpanStyle {
                name,
                style,
                users,
                into,
            })
        }
        Op::RestoreSpanStyle {
            name,
            style,
            users,
            into,
        } => {
            free_span_style_name(doc, &name)?;
            doc.span_styles.insert(name.clone(), style);
            for (area, span) in users {
                set_span_choice(doc, area, span, SpanChoice::Named(name.clone()))?;
            }
            Ok(Op::RemoveSpanStyle { name, into })
        }
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

/// Change the text of the area `area` by `edit`, which is given the document, for the counter, and
/// the text's root, and return the inverse: the area's former content.
fn edit_text(
    doc: &mut Document,
    area: NodeId,
    edit: impl FnOnce(&mut Document, &mut TextNode) -> Result<(), OpError>,
) -> Result<Op, OpError> {
    let before = area_mut(doc, area)?.content.clone();
    let Content::Text(text) = &before else {
        return Err(OpError::NotAText(area));
    };
    let mut root = text.root.clone();
    edit(doc, &mut root)?;
    if let Content::Text(text) = &mut area_mut(doc, area)?.content {
        text.root = root;
    }
    Ok(Op::SetContent {
        area,
        content: before,
    })
}

/// Insert one item at an offset of a text node.
fn insert_item(
    doc: &mut Document,
    area: NodeId,
    node: TextNodeRef,
    offset: usize,
    item: Elem,
) -> Result<Op, OpError> {
    edit_text(doc, area, |_, root| {
        let mut seq = Seq::of(text_node(root, node)?);
        seq.check(offset)?;
        seq.0.insert(offset, item);
        seq.store(text_node(root, node)?);
        Ok(())
    })
}

/// Make each reference of every text that `retarget` maps name what it gives, and return each
/// changed area with its former content, in the order of the slides and of their trees.
fn retarget_texts(
    doc: &mut Document,
    retarget: impl Fn(NodeId) -> Option<NodeId>,
) -> Vec<(NodeId, Content)> {
    let mut before = Vec::new();
    for slide in &mut doc.slides {
        retarget_node(&mut slide.root, &retarget, &mut before);
    }
    before
}

fn retarget_node(
    node: &mut Node,
    retarget: &impl Fn(NodeId) -> Option<NodeId>,
    before: &mut Vec<(NodeId, Content)>,
) {
    match node {
        Node::Split(s) => {
            for child in &mut s.children {
                retarget_node(child, retarget, before);
            }
        }
        Node::Area(a) => {
            if let Content::Text(text) = &mut a.content {
                let old = text.clone();
                if text.root.retarget(retarget) {
                    before.push((a.id, Content::Text(old)));
                }
            }
        }
    }
}

/// Refuse a custom span style that sets no field: such a span changes nothing, so its caller
/// unwraps it instead, `design@rootpoint-model@text-tree-normal-form`. A named span is never refused, whatever its values, so that a span keeps its
/// name when its style's values come to equal its parent's.
fn check_span_style(style: &SpanChoice) -> Result<(), OpError> {
    match style {
        SpanChoice::Custom(s) if *s == SpanStyle::default() => Err(OpError::EmptySpanStyle),
        _ => Ok(()),
    }
}

/// The text node `node` of a text's root.
fn text_node(root: &mut TextNode, node: TextNodeRef) -> Result<&mut TextNode, OpError> {
    match node {
        TextNodeRef::Root => Ok(root),
        TextNodeRef::Span(id) => find_span(root, id)
            .map(|s| &mut s.text)
            .ok_or(OpError::UnknownSpan(id)),
    }
}

fn find_span(node: &mut TextNode, id: SpanId) -> Option<&mut Span> {
    // The item is found first by a shared search, since returning a mutable borrow from one branch
    // of a loop that goes on borrowing is refused by the borrow checker.
    let at = node.items.iter().position(|item| match item {
        TextItem::Span(s) => s.id == id || s.text.spans().iter().any(|t| t.id == id),
        TextItem::Chars(_) | TextItem::Math(_) | TextItem::Ref(_) => false,
    })?;
    let TextItem::Span(s) = &mut node.items[at] else {
        unreachable!("the position is a span's")
    };
    if s.id == id {
        Some(s)
    } else {
        find_span(&mut s.text, id)
    }
}

/// The text node that holds the span `id` as an item, and the span's offset in it.
fn span_parent(root: &mut TextNode, id: SpanId) -> Result<(&mut TextNode, usize), OpError> {
    fn holds(node: &TextNode, id: SpanId) -> bool {
        node.items
            .iter()
            .any(|i| matches!(i, TextItem::Span(s) if s.id == id))
    }
    fn walk(node: &mut TextNode, id: SpanId) -> Option<&mut TextNode> {
        if holds(node, id) {
            return Some(node);
        }
        node.items.iter_mut().find_map(|item| match item {
            TextItem::Span(s) => walk(&mut s.text, id),
            TextItem::Chars(_) | TextItem::Math(_) | TextItem::Ref(_) => None,
        })
    }
    let parent = walk(root, id).ok_or(OpError::UnknownSpan(id))?;
    let at = Seq::of(parent)
        .0
        .iter()
        .position(|e| matches!(e, Elem::Span(s) if s.id == id))
        .expect("walk found the span among the node's items");
    Ok((parent, at))
}

/// One offset of a text node: a character, a span, a formula or a reference.
enum Elem {
    Char(char),
    Span(Span),
    Math(Formula),
    Ref(NodeId),
}

/// A text node as its sequence of offsets, which the operations edit by offset, then store back in
/// the canonical form.
struct Seq(Vec<Elem>);

impl Seq {
    fn of(node: &TextNode) -> Seq {
        let mut out = Vec::new();
        for item in &node.items {
            match item {
                TextItem::Chars(s) => out.extend(s.chars().map(Elem::Char)),
                TextItem::Span(span) => out.push(Elem::Span(span.clone())),
                TextItem::Math(formula) => out.push(Elem::Math(formula.clone())),
                TextItem::Ref(area) => out.push(Elem::Ref(*area)),
            }
        }
        Seq(out)
    }

    fn store(self, node: &mut TextNode) {
        let mut items = Vec::new();
        let mut run = String::new();
        for elem in self.0 {
            let item = match elem {
                Elem::Char(c) => {
                    run.push(c);
                    continue;
                }
                Elem::Span(span) => TextItem::Span(span),
                Elem::Math(formula) => TextItem::Math(formula),
                Elem::Ref(area) => TextItem::Ref(area),
            };
            if !run.is_empty() {
                items.push(TextItem::Chars(std::mem::take(&mut run)));
            }
            items.push(item);
        }
        if !run.is_empty() {
            items.push(TextItem::Chars(run));
        }
        node.items = items;
    }

    fn check(&self, offset: usize) -> Result<(), OpError> {
        if offset <= self.0.len() {
            Ok(())
        } else {
            Err(OpError::OffsetOutOfRange {
                offset,
                len: self.0.len(),
            })
        }
    }

    fn check_range(&self, range: &std::ops::Range<usize>) -> Result<(), OpError> {
        self.check(range.end)?;
        if range.start > range.end {
            return Err(OpError::OffsetOutOfRange {
                offset: range.start,
                len: self.0.len(),
            });
        }
        Ok(())
    }
}

/// Refuse a span style name that is empty or in the span style table.
fn free_span_style_name(doc: &Document, name: &str) -> Result<(), OpError> {
    if name.is_empty() {
        Err(OpError::EmptyStyleName)
    } else if doc.span_styles.contains_key(name) {
        Err(OpError::StyleExists(name.to_owned()))
    } else {
        Ok(())
    }
}

/// The spans that name the span style `name`, each with its area, in the order of the slides, of
/// their trees and of each text.
pub fn span_style_users(doc: &Document, name: &str) -> Vec<(NodeId, SpanId)> {
    let mut out = Vec::new();
    for area in doc.slides.iter().flat_map(|s| s.root.areas()) {
        if let Content::Text(text) = &area.content {
            for span in text.root.spans() {
                if matches!(&span.style, SpanChoice::Named(n) if n == name) {
                    out.push((area.id, span.id));
                }
            }
        }
    }
    out
}

/// Give the span `span` of the text of the area `area` the style `choice`.
fn set_span_choice(
    doc: &mut Document,
    area: NodeId,
    span: SpanId,
    choice: SpanChoice,
) -> Result<(), OpError> {
    match &mut area_mut(doc, area)?.content {
        Content::Text(text) => {
            find_span(&mut text.root, span)
                .ok_or(OpError::UnknownSpan(span))?
                .style = choice;
            Ok(())
        }
        Content::Image(_) | Content::Equation(_) => Err(OpError::NotAText(area)),
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
        Content::Image(_) | Content::Equation(_) => Err(OpError::NotAText(id)),
    }
}

/// An empty text in the style `style`: the content of a new area.
fn new_text(style: StyleChoice) -> Content {
    Content::Text(TextContent::plain("", style))
}

/// Bring every text of the document to its canonical form, [`TextNode::normalise`].
pub(crate) fn normalise_texts(doc: &mut Document) {
    for slide in &mut doc.slides {
        normalise_node(&mut slide.root);
    }
}

fn normalise_node(node: &mut Node) {
    match node {
        Node::Split(s) => s.children.iter_mut().for_each(normalise_node),
        Node::Area(Area {
            content: Content::Text(text),
            ..
        }) => text.root.normalise(),
        Node::Area(_) => {}
    }
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

/// Whether an area or a style of a table uses the asset: an image, a text's or a span's own font,
/// or a named style's or span style's font.
fn asset_in_use(doc: &Document, id: AssetId) -> bool {
    let font = FontRef::Asset(id);
    let span_font = |s: &SpanStyle| s.font.as_ref() == Some(&font);
    doc.styles.values().any(|s| s.font == font)
        || doc.span_styles.values().any(span_font)
        || doc
            .slides
            .iter()
            .flat_map(|s| s.root.areas())
            .any(|a| match &a.content {
                Content::Image(image) => image.asset == id,
                Content::Equation(_) => false,
                Content::Text(text) => {
                    matches!(&text.style, StyleChoice::Custom(style) if style.font == font)
                        || text.root.spans().into_iter().any(
                            |span| matches!(&span.style, SpanChoice::Custom(s) if span_font(s)),
                        )
                }
            })
}

/// Give every node of the subtree, and every span of its texts, a new id, and record each area's new
/// id in `copies` by its former one.
fn renumber(
    node: &mut Node,
    doc: &mut Document,
    copies: &mut std::collections::BTreeMap<NodeId, NodeId>,
) {
    match node {
        Node::Split(s) => {
            s.id = NodeId(doc.alloc());
            for child in &mut s.children {
                renumber(child, doc, copies);
            }
        }
        Node::Area(a) => {
            let id = NodeId(doc.alloc());
            copies.insert(a.id, id);
            a.id = id;
            if let Content::Text(text) = &mut a.content {
                renumber_spans(&mut text.root, doc);
            }
        }
    }
}

/// Make each reference of the subtree's texts to an area `copies` holds name that area's copy.
fn redirect(node: &mut Node, copies: &std::collections::BTreeMap<NodeId, NodeId>) {
    match node {
        Node::Split(s) => s.children.iter_mut().for_each(|c| redirect(c, copies)),
        Node::Area(Area {
            content: Content::Text(text),
            ..
        }) => {
            text.root.retarget(&|id| copies.get(&id).copied());
        }
        Node::Area(_) => {}
    }
}

/// Give every span of the text node, and of the spans below, a new id.
pub(crate) fn renumber_spans(node: &mut TextNode, doc: &mut Document) {
    for item in &mut node.items {
        if let TextItem::Span(span) = item {
            span.id = SpanId(doc.alloc());
            renumber_spans(&mut span.text, doc);
        }
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
