//! The caret of a text edited in place, per `design@rootpoint-app@edit-in-place`: a caret position
//! of the text's tree, a text node and an offset in it, mapped to the byte index of the layout
//! `rootpoint_render::area_text` gives, where Parley's `Cursor` and `Selection` place and draw it.
//! The actions that change the text, by the small commands of
//! `design@rootpoint-protocol@text-edit-commands`, are in `path@rootpoint-app@src/actions.rs`.

use std::ops::Range;

use eframe::egui::{Pos2, Rect};
use parley::{Affinity, Cluster, Layout as TextLayout, Selection};
use rootpoint_model::{
    CaretPos, Content, Node, NodeId, Span, SpanId, TextContent, TextItem, TextNode, TextNodeRef,
};
use rootpoint_render::{AreaText, TextBrush, TextMap, area_text};

use crate::App;
use crate::hit::Layout;

/// The width of the drawn caret, in points.
pub const CARET_WIDTH: f32 = 2.0;

/// The text being edited in place: its area, the caret and the selection's anchor as caret
/// positions of its tree, and the offsets of the IME's composing text in the caret's text node,
/// which stands in the area's text while composing.
#[derive(Clone, Debug)]
pub(crate) struct Editing {
    pub area: NodeId,
    pub caret: CaretPos,
    /// Where the selection began, `None` when it is collapsed. The selection is the range between
    /// it and the caret, both lifted to their nearest common text node, [`selection`].
    pub anchor: Option<CaretPos>,
    /// The side of the caret's byte index it is drawn on, which tells the end of a line from the
    /// start of the next where a line wraps.
    pub affinity: Affinity,
    /// The Parley selection of the last move by lines, kept while the caret stays where it put
    /// it, so that a run of moves by lines keeps the caret's column.
    pub column: Option<Selection>,
    pub preedit: Option<Range<usize>>,
}

/// A move of the caret.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaretMove {
    Left,
    Right,
    WordLeft,
    WordRight,
    Up,
    Down,
    LineStart,
    LineEnd,
    TextStart,
    TextEnd,
}

/// One item of a text node's sequence, as an offset counts it: a character, or a span whole.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Item<'a> {
    Char(char),
    Span(&'a Span),
}

/// The text node `node` of a text.
pub(crate) fn text_node(text: &TextContent, node: TextNodeRef) -> Option<&TextNode> {
    match node {
        TextNodeRef::Root => Some(&text.root),
        TextNodeRef::Span(id) => text
            .root
            .spans()
            .into_iter()
            .find(|s| s.id == id)
            .map(|s| &s.text),
    }
}

/// The items of a text node, one per offset.
pub(crate) fn items(node: &TextNode) -> Vec<Item<'_>> {
    let mut out = Vec::new();
    for item in &node.items {
        match item {
            TextItem::Chars(s) => out.extend(s.chars().map(Item::Char)),
            TextItem::Span(span) => out.push(Item::Span(span)),
        }
    }
    out
}

/// The text node that holds `span`, and the span's offset in it.
pub(crate) fn parent(text: &TextContent, span: SpanId) -> Option<(TextNodeRef, usize)> {
    fn find(node: &TextNode, here: TextNodeRef, span: SpanId) -> Option<(TextNodeRef, usize)> {
        for (i, item) in items(node).into_iter().enumerate() {
            if let Item::Span(s) = item {
                if s.id == span {
                    return Some((here, i));
                }
                if let Some(found) = find(&s.text, TextNodeRef::Span(s.id), span) {
                    return Some(found);
                }
            }
        }
        None
    }
    find(&text.root, TextNodeRef::Root, span)
}

/// The text nodes from the root down to `pos`'s, each with the offset in it of the next one's span,
/// and the last with `pos`'s offset.
fn path(text: &TextContent, pos: CaretPos) -> Vec<(TextNodeRef, usize)> {
    let mut out = vec![(pos.node, pos.offset)];
    let mut node = pos.node;
    while let TextNodeRef::Span(id) = node {
        let Some((up, at)) = parent(text, id) else {
            break;
        };
        out.push((up, at));
        node = up;
    }
    out.reverse();
    out
}

/// Whether a caret position stands at a byte index where a cluster of the layout starts, or at
/// the text's end, so that the caret is drawn between two glyphs and not inside one.
fn on_cluster_boundary(layout: &TextLayout<TextBrush>, index: usize) -> bool {
    Cluster::from_byte_index(layout, index).is_none_or(|c| c.text_range().start == index)
}

/// The caret position Right gives, by the milestone's table of caret moves: past one cluster of
/// characters, into a span at its start, or out of a span at its end; nothing at the text's end.
/// With `over`, as Shift with Right, a span is passed whole rather than entered.
pub(crate) fn right(
    text: &TextContent,
    map: &TextMap,
    layout: &TextLayout<TextBrush>,
    pos: CaretPos,
    over: bool,
) -> CaretPos {
    let Some(node) = text_node(text, pos.node) else {
        return pos;
    };
    let items = items(node);
    match items.get(pos.offset) {
        Some(Item::Span(s)) if !over => CaretPos {
            node: TextNodeRef::Span(s.id),
            offset: 0,
        },
        Some(Item::Span(_)) => CaretPos {
            offset: pos.offset + 1,
            ..pos
        },
        Some(Item::Char(_)) => {
            let mut j = pos.offset + 1;
            while j < items.len()
                && matches!(items[j], Item::Char(_))
                && !map
                    .byte_index(CaretPos { offset: j, ..pos })
                    .is_none_or(|b| on_cluster_boundary(layout, b))
            {
                j += 1;
            }
            CaretPos { offset: j, ..pos }
        }
        None => match pos.node {
            TextNodeRef::Root => pos,
            TextNodeRef::Span(id) => parent(text, id).map_or(pos, |(node, at)| CaretPos {
                node,
                offset: at + 1,
            }),
        },
    }
}

/// The caret position Left gives, the mirror of [`right`].
pub(crate) fn left(
    text: &TextContent,
    map: &TextMap,
    layout: &TextLayout<TextBrush>,
    pos: CaretPos,
    over: bool,
) -> CaretPos {
    let Some(node) = text_node(text, pos.node) else {
        return pos;
    };
    let items = items(node);
    if pos.offset == 0 {
        return match pos.node {
            TextNodeRef::Root => pos,
            TextNodeRef::Span(id) => {
                parent(text, id).map_or(pos, |(node, at)| CaretPos { node, offset: at })
            }
        };
    }
    match items.get(pos.offset - 1) {
        Some(Item::Span(s)) if !over => CaretPos {
            node: TextNodeRef::Span(s.id),
            offset: items_len(&s.text),
        },
        Some(Item::Span(_)) => CaretPos {
            offset: pos.offset - 1,
            ..pos
        },
        Some(Item::Char(_)) => {
            let mut j = pos.offset - 1;
            while j > 0
                && matches!(items[j - 1], Item::Char(_))
                && !map
                    .byte_index(CaretPos { offset: j, ..pos })
                    .is_none_or(|b| on_cluster_boundary(layout, b))
            {
                j -= 1;
            }
            CaretPos { offset: j, ..pos }
        }
        None => pos,
    }
}

/// The number of items of a text node, its last offset.
pub(crate) fn items_len(node: &TextNode) -> usize {
    node.items
        .iter()
        .map(|item| match item {
            TextItem::Chars(s) => s.chars().count(),
            TextItem::Span(_) => 1,
        })
        .sum()
}

/// The caret position a byte index of the layout gives, as Parley's `Cursor` places it: in the
/// deepest text node that owns the glyph on the cursor's side, `Downstream` the glyph after the
/// index and `Upstream` the glyph before. Where several edges meet at the index, the caret goes
/// into the node of that glyph, on the side of the glyph a click took. With no glyph on that side,
/// at a text's start or end, the other side is taken; an empty text gives the root's offset 0.
pub(crate) fn from_byte(text: &TextContent, index: usize, affinity: Affinity) -> CaretPos {
    /// The node and offset of the character starting (`after`) or ending at `index`.
    fn walk(
        node: &TextNode,
        here: TextNodeRef,
        at: &mut usize,
        index: usize,
        after: bool,
    ) -> Option<CaretPos> {
        for (offset, item) in items(node).into_iter().enumerate() {
            match item {
                Item::Char(c) => {
                    let (start, end) = (*at, *at + c.len_utf8());
                    *at = end;
                    if after && start == index {
                        return Some(CaretPos { node: here, offset });
                    }
                    if !after && end == index {
                        return Some(CaretPos {
                            node: here,
                            offset: offset + 1,
                        });
                    }
                }
                Item::Span(s) => {
                    if let Some(pos) = walk(&s.text, TextNodeRef::Span(s.id), at, index, after) {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }
    let find = |after: bool| walk(&text.root, TextNodeRef::Root, &mut 0, index, after);
    let downstream = affinity == Affinity::Downstream;
    find(downstream)
        .or_else(|| find(!downstream))
        .unwrap_or(CaretPos {
            node: TextNodeRef::Root,
            offset: 0,
        })
}

/// The selection between `anchor` and `caret`, lifted to their nearest common text node, as
/// #selection-one-level of `milestone@plans@rich-text` says: an end inside a span of that node
/// takes the span whole. The text node, the selected range of its offsets, and the caret's end.
pub(crate) fn selection(
    text: &TextContent,
    anchor: CaretPos,
    caret: CaretPos,
) -> (TextNodeRef, Range<usize>, usize) {
    let (a, f) = (path(text, anchor), path(text, caret));
    // The deepest node both paths pass through.
    let depth = a
        .iter()
        .zip(&f)
        .take_while(|(x, y)| x.0 == y.0)
        .count()
        .max(1)
        - 1;
    let node = a[depth].0;
    let (a_off, a_deeper) = (a[depth].1, a.len() > depth + 1);
    let (f_off, f_deeper) = (f[depth].1, f.len() > depth + 1);
    // At one offset, the end that is no deeper stands before the span the other is inside.
    let anchor_first = a_off < f_off || (a_off == f_off && !a_deeper);
    let (anchor_at, caret_at) = if anchor_first {
        (a_off, f_off + usize::from(f_deeper))
    } else {
        (a_off + usize::from(a_deeper), f_off)
    };
    (
        node,
        anchor_at.min(caret_at)..anchor_at.max(caret_at),
        caret_at,
    )
}

/// The ids of the spans of a text that hold no item, the inner ones before those that hold them.
pub(crate) fn empty_spans(text: &TextContent) -> Vec<SpanId> {
    let mut spans = text.root.spans();
    spans.reverse();
    spans
        .into_iter()
        .filter(|s| s.text.items.is_empty())
        .map(|s| s.id)
        .collect()
}

/// The characters of the items `range` of a text node, a span's included.
pub(crate) fn range_text(node: &TextNode, range: Range<usize>) -> String {
    fn push_all(node: &TextNode, out: &mut String) {
        for item in &node.items {
            match item {
                TextItem::Chars(s) => out.push_str(s),
                TextItem::Span(span) => push_all(&span.text, out),
            }
        }
    }
    let mut out = String::new();
    for item in items(node).into_iter().skip(range.start).take(range.len()) {
        match item {
            Item::Char(c) => out.push(c),
            Item::Span(s) => push_all(&s.text, &mut out),
        }
    }
    out
}

impl Editing {
    /// The selection, lifted to one text node, or `None` when it is collapsed or empty.
    pub(crate) fn selected(&self, text: &TextContent) -> Option<(TextNodeRef, Range<usize>)> {
        let anchor = self.anchor?;
        let (node, range, _) = selection(text, anchor, self.caret);
        (!range.is_empty()).then_some((node, range))
    }

    /// The caret's end of the selection, lifted to the selection's text node; the caret itself
    /// when the selection is collapsed.
    pub(crate) fn focus(&self, text: &TextContent) -> CaretPos {
        match self.anchor {
            Some(anchor) => {
                let (node, _, offset) = selection(text, anchor, self.caret);
                CaretPos { node, offset }
            }
            None => self.caret,
        }
    }
}

impl App {
    /// The area whose text is being edited, if any.
    pub fn editing(&self) -> Option<NodeId> {
        self.edit.as_ref().map(|e| e.area)
    }

    /// Whether `node` is an area of the shown slide that holds a text, which can be edited.
    pub fn is_text_area(&self, node: NodeId) -> bool {
        matches!(
            self.node(node),
            Some(Node::Area(a)) if matches!(a.content, Content::Text(_))
        )
    }

    /// The caret of the text being edited, at the selection's end when it is lifted.
    pub fn caret(&self) -> Option<CaretPos> {
        let edit = self.edit.as_ref()?;
        Some(edit.focus(&self.edited_text()?))
    }

    /// The byte offset of the caret in the text being edited, in the string the layout holds.
    pub fn caret_index(&mut self) -> Option<usize> {
        let caret = self.caret()?;
        self.area_text(self.editing()?)?.map.byte_index(caret)
    }

    /// The selected text, empty when the selection is collapsed.
    pub fn selected_text(&self) -> Option<String> {
        let edit = self.edit.as_ref()?;
        let text = self.edited_text()?;
        Some(match edit.selected(&text) {
            Some((node, range)) => range_text(text_node(&text, node)?, range),
            None => String::new(),
        })
    }

    /// The byte range of the composing text, if the IME is composing.
    pub fn preedit_range(&mut self) -> Option<Range<usize>> {
        let edit = self.edit.clone()?;
        let r = edit.preedit?;
        let map = self.area_text(edit.area)?.map;
        let at = |offset| {
            map.byte_index(CaretPos {
                node: edit.caret.node,
                offset,
            })
        };
        Some(at(r.start)?..at(r.end)?)
    }

    /// The caret's rectangle in the slide view, in points, `CARET_WIDTH` wide, placed through the
    /// area's layout and its origin as `render_slide` draws them.
    pub fn caret_rect(&mut self, layout: &Layout) -> Option<Rect> {
        let area = self.editing()?;
        let text = self.area_text(area)?;
        let edit = self.edit.as_ref()?;
        let content = self.edited_text()?;
        let index = text.map.byte_index(edit.focus(&content))?;
        let cursor = Selection::from_byte_index(&text.layout, index, edit.affinity).focus();
        let width = CARET_WIDTH / layout.points_per_unit();
        Some(to_view(&text, layout, cursor.geometry(&text.layout, width)))
    }

    /// The selection's rectangles in the slide view, in points, one per line it covers.
    pub fn selection_rects(&mut self, layout: &Layout) -> Vec<Rect> {
        let Some(range) = (|| {
            let edit = self.edit.as_ref()?;
            let (node, r) = edit.selected(&self.edited_text()?)?;
            let map = self.area_text(edit.area)?.map;
            let at = |offset| map.byte_index(CaretPos { node, offset });
            Some(at(r.start)?..at(r.end)?)
        })() else {
            return Vec::new();
        };
        self.range_rects(layout, range)
    }

    /// The composing text's rectangles in the slide view, which the overlay underlines.
    pub fn preedit_rects(&mut self, layout: &Layout) -> Vec<Rect> {
        match self.preedit_range() {
            Some(range) => self.range_rects(layout, range),
            None => Vec::new(),
        }
    }

    /// The text of the area being edited, as the document holds it.
    pub(crate) fn edited_text(&self) -> Option<TextContent> {
        self.text_content(self.editing()?)
    }

    /// The text of `area`, if it holds one.
    pub(crate) fn text_content(&self, area: NodeId) -> Option<TextContent> {
        match self.node(area)? {
            Node::Area(a) => match &a.content {
                Content::Text(t) => Some(t.clone()),
                _ => None,
            },
            Node::Split(_) => None,
        }
    }

    fn range_rects(&mut self, layout: &Layout, r: Range<usize>) -> Vec<Rect> {
        let Some(area) = self.editing() else {
            return Vec::new();
        };
        let Some(text) = self.area_text(area) else {
            return Vec::new();
        };
        if r.is_empty() {
            return Vec::new();
        }
        let l = &text.layout;
        let s = Selection::new(
            Selection::from_byte_index(l, r.start, Affinity::Downstream).focus(),
            Selection::from_byte_index(l, r.end, Affinity::Upstream).focus(),
        );
        s.geometry(l)
            .into_iter()
            .map(|(b, _)| to_view(&text, layout, b))
            .collect()
    }

    /// The text of `area` as the render crate lays it out, from the session's current fonts.
    pub(crate) fn area_text(&mut self, area: NodeId) -> Option<AreaText> {
        let slide = self.shown_slide()?;
        self.assets.refresh(&self.session, &mut *self.files);
        let (fonts, _) = self.assets.sets();
        area_text(self.session.document(), slide, area, fonts)
    }
}

/// A point of the slide view, in the coordinates of a text's layout: slide units from the
/// layout's origin.
pub(crate) fn layout_point(text: &AreaText, layout: &Layout, p: Pos2) -> (f32, f32) {
    let (u, v) = layout.to_units(p);
    ((u - text.origin.0) as f32, (v - text.origin.1) as f32)
}

/// A box of a text's layout, carried to the slide view.
fn to_view(text: &AreaText, layout: &Layout, b: parley::BoundingBox) -> Rect {
    Rect::from_min_max(
        layout.to_points(text.origin.0 + b.x0, text.origin.1 + b.y0),
        layout.to_points(text.origin.0 + b.x1, text.origin.1 + b.y1),
    )
}
