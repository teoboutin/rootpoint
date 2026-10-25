//! The map between a text's caret positions and the byte indices of the string the renderer lays
//! out, `design@rootpoint-render@area-text-layout`.

use std::collections::{BTreeMap, HashMap};
use std::ops::Range;

use rootpoint_math::Formula;
use rootpoint_model::{CaretPos, NodeId, SpanId, TextContent, TextItem, TextNode, TextNodeRef};

/// The character a formula stands on in the laid-out string, after its inline box: the word joiner,
/// which a font draws as nothing, with no advance, so the caret positions before and after the
/// formula are two byte indices, either side of the box. Parley 0.12 places an inline box at a byte
/// index and gives it none of the string, so with no character after it, the position after a
/// formula would be the position after the next character.
const FORMULA_MARK: char = '\u{2060}';

/// Where each caret position of a text stands in its laid-out string, the text's characters in
/// order, each formula as [`FORMULA_MARK`] after its box, and each reference as the number it
/// draws; and the byte range each span covers. A caret position at an edge, before a span or at
/// its start, stands at the same byte index as the other position of that edge: the two differ by
/// the text node they type into, not by where they are drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextMap {
    /// For each text node, the byte index of each of its offsets, from 0 to its length.
    offsets: HashMap<TextNodeRef, Vec<usize>>,
    spans: HashMap<SpanId, Range<usize>>,
    text: String,
    formulas: Vec<MapFormula>,
}

/// A formula of a text: the byte range it covers in the laid-out string, its box then its mark, the
/// spans that hold it, outer first, and the formula.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MapFormula {
    pub range: Range<usize>,
    pub spans: Vec<SpanId>,
    pub formula: Formula,
}

impl TextMap {
    /// The map of a text, a reference drawn as its equation's number in `numbers`, `(n)`, or as
    /// `(??)` when the area it names has no number.
    pub fn new(text: &TextContent, numbers: &BTreeMap<NodeId, u32>) -> Self {
        let mut map = TextMap::default();
        map.walk(TextNodeRef::Root, &text.root, numbers, &mut Vec::new());
        map
    }

    fn walk(
        &mut self,
        node: TextNodeRef,
        text: &TextNode,
        numbers: &BTreeMap<NodeId, u32>,
        spans: &mut Vec<SpanId>,
    ) {
        let mut offsets = vec![self.text.len()];
        for item in &text.items {
            match item {
                TextItem::Chars(s) => {
                    for c in s.chars() {
                        self.text.push(c);
                        offsets.push(self.text.len());
                    }
                }
                TextItem::Span(span) => {
                    let start = self.text.len();
                    spans.push(span.id);
                    self.walk(TextNodeRef::Span(span.id), &span.text, numbers, spans);
                    spans.pop();
                    self.spans.insert(span.id, start..self.text.len());
                    offsets.push(self.text.len());
                }
                TextItem::Math(formula) => {
                    let start = self.text.len();
                    self.text.push(FORMULA_MARK);
                    self.formulas.push(MapFormula {
                        range: start..self.text.len(),
                        spans: spans.clone(),
                        formula: formula.clone(),
                    });
                    offsets.push(self.text.len());
                }
                TextItem::Ref(area) => {
                    match numbers.get(area) {
                        Some(n) => self.text.push_str(&format!("({n})")),
                        None => self.text.push_str("(??)"),
                    }
                    offsets.push(self.text.len());
                }
            }
        }
        self.offsets.insert(node, offsets);
    }

    /// The byte index a caret position stands at, or `None` for a text node the text lacks or an
    /// offset past its end.
    pub fn byte_index(&self, pos: CaretPos) -> Option<usize> {
        self.offsets.get(&pos.node)?.get(pos.offset).copied()
    }

    /// The number of offsets of a text node: its length, a span, a formula and a reference each
    /// counting as one, plus one.
    pub fn positions(&self, node: TextNodeRef) -> Option<usize> {
        self.offsets.get(&node).map(Vec::len)
    }

    /// The byte range of the laid-out string a span covers.
    pub fn span_range(&self, span: SpanId) -> Option<Range<usize>> {
        self.spans.get(&span).cloned()
    }

    /// The string the renderer lays out.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The text's formulas, in the order of the text.
    pub(crate) fn formulas(&self) -> &[MapFormula] {
        &self.formulas
    }
}
