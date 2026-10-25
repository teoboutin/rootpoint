//! The map between a text's caret positions and the byte indices of the string the renderer lays
//! out, `design@rootpoint-render@area-text-layout`.

use std::collections::HashMap;
use std::ops::Range;

use rootpoint_model::{CaretPos, SpanId, TextContent, TextItem, TextNode, TextNodeRef};

/// Where each caret position of a text stands in its laid-out string, the text's characters in
/// order, and the byte range each span covers. A caret position at an edge, before a span or at
/// its start, stands at the same byte index as the other position of that edge: the two differ by
/// the text node they type into, not by where they are drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextMap {
    /// For each text node, the byte index of each of its offsets, from 0 to its length.
    offsets: HashMap<TextNodeRef, Vec<usize>>,
    spans: HashMap<SpanId, Range<usize>>,
}

impl TextMap {
    /// The map of a text, its laid-out string being [`TextContent::plain_text`].
    pub fn new(text: &TextContent) -> Self {
        let mut map = TextMap::default();
        let mut at = 0;
        map.walk(TextNodeRef::Root, &text.root, &mut at);
        map
    }

    fn walk(&mut self, node: TextNodeRef, text: &TextNode, at: &mut usize) {
        let mut offsets = vec![*at];
        for item in &text.items {
            match item {
                TextItem::Chars(s) => {
                    for c in s.chars() {
                        *at += c.len_utf8();
                        offsets.push(*at);
                    }
                }
                TextItem::Span(span) => {
                    let start = *at;
                    self.walk(TextNodeRef::Span(span.id), &span.text, at);
                    self.spans.insert(span.id, start..*at);
                    offsets.push(*at);
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

    /// The number of offsets of a text node: its length, a span counting as one, plus one.
    pub fn positions(&self, node: TextNodeRef) -> Option<usize> {
        self.offsets.get(&node).map(Vec::len)
    }

    /// The byte range of the laid-out string a span covers.
    pub fn span_range(&self, span: SpanId) -> Option<Range<usize>> {
        self.spans.get(&span).cloned()
    }
}
