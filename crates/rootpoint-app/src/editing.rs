//! The caret of a text edited in place, per `design@rootpoint-app@edit-in-place`: a caret position
//! of the text's tree, a text node and an offset in it, mapped to the byte index of the layout
//! `rootpoint_render::area_text` gives, where Parley's `Cursor` and `Selection` place and draw it.
//! The actions that change the text, by the small commands of
//! `design@rootpoint-protocol@text-edit-commands`, are in `path@rootpoint-app@src/actions.rs`.

use std::ops::Range;

use eframe::egui::{Pos2, Rect};
use parley::{Affinity, Cluster, Layout as TextLayout, Selection};
use rootpoint_model::{
    CaretPos, Content, Document, Node, NodeId, Span, SpanId, TextContent, TextItem, TextNode,
    TextNodeRef, TextStyle,
};
use rootpoint_render::{AreaText, TextBrush, TextMap, area_text};

use crate::App;
use crate::hit::Layout;

/// The width of the drawn caret, in points.
pub const CARET_WIDTH: f32 = 2.0;

/// The half width of the frame of an empty span, which covers no glyph, around the caret, in
/// points.
pub const EMPTY_FRAME_HALF_WIDTH: f32 = 3.0;

/// The frame of a span that holds the caret, as the overlay draws it: one rectangle per line the
/// span covers, in points, and its depth among the frames, 0 for the innermost.
#[derive(Clone, Debug, PartialEq)]
pub struct SpanFrame {
    pub span: SpanId,
    pub depth: usize,
    pub rects: Vec<Rect>,
}

/// The text being edited in place: its area, the caret and the selection's anchor as caret
/// positions of its tree, and the IME's composing text, which stands in the area's text while
/// composing: the text node it was typed into, and its offsets there.
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
    pub preedit: Option<(TextNodeRef, Range<usize>)>,
    /// The caret in a formula: in the math item after `caret` of a text, or in an equation area's
    /// formula, where `caret` names nothing; `None` while the caret is in the text.
    pub math: Option<MathCaret>,
}

/// The caret in a formula: its place and the atom a first press of Backspace or Delete selected,
/// the editor's state alone, `design@rootpoint-app@caret-editor-state`; and the name of the command
/// being typed after `\`, which the overlay shows at the caret and the formula holds once it closes,
/// held by the editor for the reason `App::rename_command` gives.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MathCaret {
    pub path: rootpoint_math::CellPath,
    pub selected: Option<rootpoint_math::CellPath>,
    pub command: Option<String>,
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

/// One item of a text node's sequence, as an offset counts it: a character, a span whole, or a
/// formula or a reference, which the caret steps over as one item.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Item<'a> {
    Char(char),
    Span(&'a Span),
    Object(&'a TextItem),
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
            TextItem::Math(_) | TextItem::Ref(_) => out.push(Item::Object(item)),
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

/// The caret position Right gives, `design@rootpoint-app@boundary-caret-positions`: past one
/// cluster of characters, into a span at its start, or out of a span at its end; nothing at the
/// text's end.
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
        Some(Item::Span(_) | Item::Object(_)) => CaretPos {
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
        Some(Item::Span(_) | Item::Object(_)) => CaretPos {
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
            TextItem::Span(_) | TextItem::Math(_) | TextItem::Ref(_) => 1,
        })
        .sum()
}

/// The caret position a byte index of the layout gives, as Parley's `Cursor` places it, by the
/// backward rule of `design@rootpoint-app@boundary-caret-positions`: in the deepest text node that
/// owns the glyph on the cursor's side, `Downstream` the glyph after the index and `Upstream` the
/// glyph before. Where several edges meet at the index, the caret goes into the node of that glyph,
/// on the side of the glyph a click took. With no glyph on that side, at a text's start or end, the
/// other side is taken; an empty text gives the root's offset 0.
pub(crate) fn from_byte(
    text: &TextContent,
    map: &TextMap,
    index: usize,
    affinity: Affinity,
) -> CaretPos {
    /// The node and offset of the character, the formula or the reference starting (`after`) or
    /// ending at `index`, the bytes each covers read from the map.
    fn walk(
        node: &TextNode,
        here: TextNodeRef,
        map: &TextMap,
        index: usize,
        after: bool,
    ) -> Option<CaretPos> {
        let at = |offset| map.byte_index(CaretPos { node: here, offset });
        for (offset, item) in items(node).into_iter().enumerate() {
            match item {
                // A reference is drawn as several characters, `(12)`: an index inside it stands
                // at its nearer edge, as the caret never enters it.
                Item::Object(_)
                    if let (Some(start), Some(end)) = (at(offset), at(offset + 1))
                        && start < index
                        && index < end =>
                {
                    let past = index - start > end - index;
                    return Some(CaretPos {
                        node: here,
                        offset: offset + usize::from(past),
                    });
                }
                Item::Char(_) | Item::Object(_) => {
                    if after && at(offset) == Some(index) {
                        return Some(CaretPos { node: here, offset });
                    }
                    if !after && at(offset + 1) == Some(index) {
                        return Some(CaretPos {
                            node: here,
                            offset: offset + 1,
                        });
                    }
                }
                Item::Span(s) => {
                    if let Some(pos) = walk(&s.text, TextNodeRef::Span(s.id), map, index, after) {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }
    let find = |after: bool| walk(&text.root, TextNodeRef::Root, map, index, after);
    let downstream = affinity == Affinity::Downstream;
    find(downstream)
        .or_else(|| find(!downstream))
        .unwrap_or(CaretPos {
            node: TextNodeRef::Root,
            offset: 0,
        })
}

/// The spans that hold a caret position, from its own text node up.
pub(crate) fn spans_holding(text: &TextContent, pos: CaretPos) -> Vec<SpanId> {
    let mut out = Vec::new();
    let mut node = pos.node;
    while let TextNodeRef::Span(id) = node {
        out.push(id);
        match parent(text, id) {
            Some((up, _)) => node = up,
            None => break,
        }
    }
    out
}

/// The values a text node draws with: the area's style, then each span from the root down to the
/// node, each value it sets over its parent's, a span naming a style the table lacks setting none.
pub(crate) fn node_values(doc: &Document, text: &TextContent, node: TextNodeRef) -> TextStyle {
    let mut values = doc.resolve_style(&text.style);
    let mut spans = spans_holding(text, CaretPos { node, offset: 0 });
    spans.reverse();
    let all = text.root.spans();
    for id in spans {
        let Some(set) = all
            .iter()
            .find(|s| s.id == id)
            .and_then(|s| doc.resolve_span(&s.style))
        else {
            continue;
        };
        if let Some(font) = set.font {
            values.font = font;
        }
        if let Some(size) = set.size {
            values.size = size;
        }
        if let Some(bold) = set.bold {
            values.bold = bold;
        }
        if let Some(italic) = set.italic {
            values.italic = italic;
        }
        if let Some(colour) = set.colour {
            values.colour = colour;
        }
    }
    values
}

/// The selection between `anchor` and `caret`, lifted to their nearest common text node, as
/// `design@rootpoint-app@selection-one-level` says: an end inside a span of that node
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

/// A formula or a reference as a copy writes it, as the protocol writes it in a tree: `$<LaTeX>$`
/// and `{ref n<id>}`, so that a formula copied to another program keeps its LaTeX.
fn object_text(item: &TextItem) -> String {
    match item {
        TextItem::Math(formula) => format!("${}$", formula.to_latex()),
        TextItem::Ref(area) => format!("{{ref {area}}}"),
        TextItem::Chars(s) => s.clone(),
        TextItem::Span(_) => String::new(),
    }
}

/// The characters of the items `range` of a text node, a span's included.
pub(crate) fn range_text(node: &TextNode, range: Range<usize>) -> String {
    fn push_all(node: &TextNode, out: &mut String) {
        for item in &node.items {
            match item {
                TextItem::Chars(s) => out.push_str(s),
                TextItem::Span(span) => push_all(&span.text, out),
                object => out.push_str(&object_text(object)),
            }
        }
    }
    let mut out = String::new();
    for item in items(node).into_iter().skip(range.start).take(range.len()) {
        match item {
            Item::Char(c) => out.push(c),
            Item::Span(s) => push_all(&s.text, &mut out),
            Item::Object(object) => out.push_str(&object_text(object)),
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
        let (node, r) = edit.preedit?;
        let map = self.area_text(edit.area)?.map;
        let at = |offset| map.byte_index(CaretPos { node, offset });
        Some(at(r.start)?..at(r.end)?)
    }

    /// The caret's rectangle in the slide view, in points, `CARET_WIDTH` wide, placed through the
    /// area's layout and its origin as `render_slide` draws them. It stands on the line's baseline,
    /// as tall as the size the caret types with, from the ascent to the descent of the line's
    /// first run, scaled to that size; so the two caret positions at the edge of a span of
    /// another size are drawn at two heights.
    /// In a formula, the caret is the formula's, `App::math_caret_rect`.
    pub fn caret_rect(&mut self, layout: &Layout) -> Option<Rect> {
        if self.math_caret().is_some() {
            return self.math_caret_rect(layout);
        }
        let area = self.editing()?;
        let text = self.area_text(area)?;
        let edit = self.edit.as_ref()?;
        let content = self.edited_text()?;
        let focus = edit.focus(&content);
        let index = text.map.byte_index(focus)?;
        let cursor = Selection::from_byte_index(&text.layout, index, edit.affinity).focus();
        let width = CARET_WIDTH / layout.points_per_unit();
        let mut b = cursor.geometry(&text.layout, width);
        let size = node_values(self.session.document(), &content, focus.node).size;
        let line = text.layout.lines().find(|l| {
            let m = l.metrics();
            f64::from(m.block_min_coord.min(m.content_block_min_coord)) <= b.y0 + 1e-3
                && b.y1 <= f64::from(m.block_max_coord.max(m.content_block_max_coord)) + 1e-3
        });
        if let Some(line) = line
            && let Some(run) = line.runs().next()
            && run.font_size() > 0.0
        {
            let m = run.font_metrics();
            let scale = size / f64::from(run.font_size());
            let baseline = f64::from(line.metrics().baseline);
            b.y0 = baseline - f64::from(m.ascent) * scale;
            b.y1 = baseline + f64::from(m.descent) * scale;
        }
        // Parley places a cursor by clusters, and a formula's inline box is none: at the byte of
        // the mark after the box, it would draw the caret before the formula after the box, where
        // the caret after it stands.
        if let Some(f) = text.formulas.iter().find(|f| f.range.start == index) {
            b.x0 = f.origin.0;
            b.x1 = f.origin.0 + f64::from(width);
        }
        Some(to_view(&text, layout, b))
    }

    /// The values the caret types with, those of its text node, at which the overlay draws it.
    pub fn typing_values(&self) -> Option<TextStyle> {
        let edit = self.edit.as_ref()?;
        let content = self.edited_text()?;
        let node = edit.focus(&content).node;
        Some(node_values(self.session.document(), &content, node))
    }

    /// The values the Selection tab's controls show while editing: those the caret types with, or,
    /// when the selection is one whole span, that span's, which a control then changes.
    pub fn caret_values(&self) -> Option<TextStyle> {
        let edit = self.edit.as_ref()?;
        let content = self.edited_text()?;
        let node = match (self.selected_span(), edit.selected(&content)) {
            (Some(span), _) => TextNodeRef::Span(span),
            (None, Some((node, _))) => node,
            (None, None) => edit.caret.node,
        };
        Some(node_values(self.session.document(), &content, node))
    }

    /// The spans that hold the caret, outer first, as the ancestor row lists them after the text;
    /// while one span is selected whole, that span and those that hold it, so the span a click on
    /// the row selected stays in the row, shown as the one the controls change.
    pub fn caret_spans(&self) -> Vec<SpanId> {
        let (Some(edit), Some(content)) = (self.edit.as_ref(), self.edited_text()) else {
            return Vec::new();
        };
        let at = match self.selected_span() {
            Some(span) => CaretPos {
                node: TextNodeRef::Span(span),
                offset: 0,
            },
            None => edit.focus(&content),
        };
        let mut spans = spans_holding(&content, at);
        spans.reverse();
        spans
    }

    /// The span the selection is the whole of, if it is one span.
    pub fn selected_span(&self) -> Option<SpanId> {
        let edit = self.edit.as_ref()?;
        let content = self.edited_text()?;
        let (node, r) = edit.selected(&content)?;
        if r.len() != 1 {
            return None;
        }
        match items(text_node(&content, node)?).get(r.start)? {
            Item::Span(s) => Some(s.id),
            Item::Char(_) | Item::Object(_) => None,
        }
    }

    /// The frames of the spans that hold the caret, inner first, each its span's lines in the
    /// slide view. An empty span covers no glyph, so its frame stands around the caret, as tall as
    /// it and `EMPTY_FRAME_HALF_WIDTH` on each side.
    pub fn span_frames(&mut self, layout: &Layout) -> Vec<SpanFrame> {
        let Some(content) = self.edited_text() else {
            return Vec::new();
        };
        let Some(edit) = self.edit.clone() else {
            return Vec::new();
        };
        let Some(text) = self.area_text(edit.area) else {
            return Vec::new();
        };
        let spans = spans_holding(&content, edit.focus(&content));
        let mut out = Vec::new();
        for (depth, span) in spans.into_iter().enumerate() {
            let Some(range) = text.map.span_range(span) else {
                continue;
            };
            let rects = if range.is_empty() {
                let Some(caret) = self.caret_rect(layout) else {
                    continue;
                };
                vec![Rect::from_min_max(
                    caret.min - eframe::egui::vec2(EMPTY_FRAME_HALF_WIDTH, 0.0),
                    caret.max + eframe::egui::vec2(EMPTY_FRAME_HALF_WIDTH, 0.0),
                )]
            } else {
                self.range_rects(layout, range)
            };
            out.push(SpanFrame { span, depth, rects });
        }
        out
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
