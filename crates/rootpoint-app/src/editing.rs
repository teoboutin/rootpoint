//! Editing an area's text in place, per `design@rootpoint-app@edit-in-place`: a Parley `Selection`
//! on the layout `rootpoint_render::area_text` gives, and one `text` command a change, inside the
//! group a stay holds, per `design@rootpoint-app@gesture-groups`.

use std::ops::Range;

use eframe::egui::{Pos2, Rect};
use parley::{Affinity, Selection};
use rootpoint_model::{Content, Node, NodeId};
use rootpoint_protocol::{Command, TreeItem};
use rootpoint_render::{AreaText, area_text};

use crate::App;
use crate::hit::Layout;

/// The width of the drawn caret, in points.
pub const CARET_WIDTH: f32 = 2.0;

/// The text being edited in place: its area, the caret and the selection on its layout, and the
/// byte range of the IME's composing text, which stands in the area's text while composing.
#[derive(Clone, Debug)]
pub(crate) struct Editing {
    pub area: NodeId,
    pub selection: Selection,
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

    /// Enter editing the text of `area` with the caret at the point `p` of the slide view: a click on
    /// the selected text area, so a first click selects and a second edits. The stay opens an undo
    /// group, after closing any group still open.
    pub fn edit_at(&mut self, area: NodeId, p: Pos2, layout: &Layout) {
        if self.socket_holds() {
            return;
        }
        if !self.is_text_area(area) {
            return;
        }
        let Some(text) = self.area_text(area) else {
            return;
        };
        let (x, y) = layout_point(&text, layout, p);
        let selection = Selection::from_point(&text.layout, x, y);
        if self.open_group() {
            self.edit = Some(Editing {
                area,
                selection,
                preedit: None,
            });
        }
    }

    /// Leave editing, closing the stay's group, so its edits are one undo step; the area stays
    /// selected. A composition still open is dropped from the text first.
    pub fn stop_editing(&mut self) {
        if self.edit.as_ref().is_some_and(|e| e.preedit.is_some()) {
            self.set_preedit("");
        }
        if self.edit.take().is_some() {
            self.run(Command::End);
        }
    }

    /// Place the caret of the text being edited at the point `p` of the slide view.
    pub fn place_caret(&mut self, p: Pos2, layout: &Layout) {
        self.at_point(p, layout, |_, l, x, y| Selection::from_point(l, x, y));
    }

    /// Extend the selection to the point `p` of the slide view, as a drag inside the text does.
    pub fn extend_to_point(&mut self, p: Pos2, layout: &Layout) {
        self.at_point(p, layout, |s, l, x, y| s.extend_to_point(l, x, y));
    }

    /// Extend the selection to the point `p` of the slide view by a Shift+click.
    pub fn shift_click(&mut self, p: Pos2, layout: &Layout) {
        self.at_point(p, layout, |s, l, x, y| s.shift_click_extension(l, x, y));
    }

    /// Move the caret, collapsing the selection, or, with `extend`, move the selection's focus.
    pub fn move_caret(&mut self, m: CaretMove, extend: bool) {
        let len = self.edited_text().map_or(0, |t| t.len());
        self.with_layout(|s, l| match m {
            CaretMove::Left => s.previous_visual(l, extend),
            CaretMove::Right => s.next_visual(l, extend),
            CaretMove::WordLeft => s.previous_visual_word(l, extend),
            CaretMove::WordRight => s.next_visual_word(l, extend),
            CaretMove::Up => s.previous_line(l, extend),
            CaretMove::Down => s.next_line(l, extend),
            CaretMove::LineStart => s.line_start(l, extend),
            CaretMove::LineEnd => s.line_end(l, extend),
            CaretMove::TextStart => {
                let start = Selection::from_byte_index(l, 0, Affinity::Downstream).focus();
                if extend {
                    s.extend(start)
                } else {
                    start.into()
                }
            }
            CaretMove::TextEnd => {
                let end = Selection::from_byte_index(l, len, Affinity::Upstream).focus();
                if extend { s.extend(end) } else { end.into() }
            }
        });
    }

    /// Select the whole text.
    pub fn select_all(&mut self) {
        let len = self.edited_text().map_or(0, |t| t.len());
        self.with_layout(|_, l| {
            let start = Selection::from_byte_index(l, 0, Affinity::Downstream).focus();
            let end = Selection::from_byte_index(l, len, Affinity::Upstream).focus();
            Selection::new(start, end)
        });
    }

    /// Put the caret at a byte offset of the text, as a test does to visit every offset.
    pub fn set_caret(&mut self, index: usize) {
        self.with_layout(|_, l| Selection::from_byte_index(l, index, Affinity::Downstream));
    }

    /// The byte offset of the caret in the text being edited.
    pub fn caret_index(&self) -> Option<usize> {
        self.edit.as_ref().map(|e| e.selection.focus().index())
    }

    /// The selected text, empty when the selection is collapsed.
    pub fn selected_text(&self) -> Option<String> {
        let edit = self.edit.as_ref()?;
        let text = self.edited_text()?;
        Some(text[edit.selection.text_range()].to_owned())
    }

    /// Type a string: it replaces the selection, and the caret follows it. Enter types a new line.
    pub fn type_str(&mut self, s: &str) {
        let Some(edit) = &self.edit else {
            return;
        };
        let range = edit.selection.text_range();
        self.replace(range, s, None);
    }

    /// Backspace: delete the selection, or the character before the caret.
    pub fn delete_backward(&mut self) {
        self.delete(false);
    }

    /// Delete: delete the selection, or the character after the caret.
    pub fn delete_forward(&mut self) {
        self.delete(true);
    }

    /// Cut: the selected text, deleted, for the clipboard.
    pub fn cut(&mut self) -> Option<String> {
        let text = self.selected_text().filter(|t| !t.is_empty())?;
        let range = self.edit.as_ref()?.selection.text_range();
        self.replace(range, "", None);
        Some(text)
    }

    /// The IME's composing text: it replaces the previous one at the caret, or the selection when
    /// composing starts, so the render draws it where it will stand; an empty one removes it.
    pub fn set_preedit(&mut self, s: &str) {
        let Some(edit) = &self.edit else {
            return;
        };
        // An empty preedit with no composition open changes nothing, per
        // `design@rootpoint-app@empty-preedit`.
        if s.is_empty() && edit.preedit.is_none() {
            return;
        }
        let range = edit
            .preedit
            .clone()
            .unwrap_or_else(|| edit.selection.text_range());
        let preedit = (!s.is_empty()).then(|| range.start..range.start + s.len());
        self.replace(range, s, Some(preedit));
    }

    /// The IME's final text: it replaces the composing text, which ends.
    pub fn commit_ime(&mut self, s: &str) {
        let Some(edit) = &self.edit else {
            return;
        };
        let range = edit
            .preedit
            .clone()
            .unwrap_or_else(|| edit.selection.text_range());
        self.replace(range, s, Some(None));
    }

    /// The byte range of the composing text, if the IME is composing.
    pub fn preedit_range(&self) -> Option<Range<usize>> {
        self.edit.as_ref()?.preedit.clone()
    }

    /// The caret's rectangle in the slide view, in points, `CARET_WIDTH` wide, placed through the
    /// area's layout and its origin as `render_slide` draws them.
    pub fn caret_rect(&mut self, layout: &Layout) -> Option<Rect> {
        let area = self.editing()?;
        let text = self.area_text(area)?;
        let edit = self.edit.as_ref()?;
        let focus = edit.selection.refresh(&text.layout).focus();
        let width = CARET_WIDTH / layout.points_per_unit();
        Some(to_view(&text, layout, focus.geometry(&text.layout, width)))
    }

    /// The selection's rectangles in the slide view, in points, one per line it covers.
    pub fn selection_rects(&mut self, layout: &Layout) -> Vec<Rect> {
        self.range_rects(layout, |e| Some(e.selection.text_range()))
    }

    /// The composing text's rectangles in the slide view, which the overlay underlines.
    pub fn preedit_rects(&mut self, layout: &Layout) -> Vec<Rect> {
        self.range_rects(layout, |e| e.preedit.clone())
    }

    /// Replace `range` of the edited text by `s`, by one `text` command, and put the caret after
    /// `s`. `preedit` sets the composing range when given, and keeps it otherwise, shifted by the
    /// change when it lies after the edit.
    fn replace(&mut self, range: Range<usize>, s: &str, preedit: Option<Option<Range<usize>>>) {
        let Some(edit) = &self.edit else {
            return;
        };
        let area = edit.area;
        let Some(mut text) = self.edited_text() else {
            return;
        };
        let range = clamp(range, &text);
        text.replace_range(range.clone(), s);
        let caret = range.start + s.len();
        let kept = edit.preedit.clone();
        self.run(Command::Text {
            area,
            root: Some(vec![TreeItem::Chars(text)]),
            style: Vec::new(),
        });
        let preedit = match preedit {
            Some(p) => p,
            None => kept.and_then(|p| {
                if p.end <= range.start {
                    Some(p)
                } else if p.start >= range.end {
                    // After the edit: moved by the change of length.
                    let shift = |i: usize| i + s.len() - range.len();
                    Some(shift(p.start)..shift(p.end))
                } else {
                    None
                }
            }),
        };
        if let Some(edit) = &mut self.edit {
            edit.preedit = preedit;
        }
        self.set_caret(caret);
    }

    /// Delete the selection, or, with none, the character after the caret or before it. The
    /// character is found by the caret's logical clusters, as Parley's `PlainEditor` does: a
    /// visual move at a soft line break only changes the caret's affinity, and would delete
    /// nothing. Backspace deletes one character of a cluster, or a whole line break or emoji.
    fn delete(&mut self, forward: bool) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let range = if edit.selection.is_collapsed() {
            let (Some(text), Some(buffer)) = (self.area_text(edit.area), self.edited_text()) else {
                return;
            };
            let focus = edit.selection.refresh(&text.layout).focus();
            let [before, after] = focus.logical_clusters(&text.layout);
            if forward {
                let Some(c) = after else {
                    return;
                };
                c.text_range()
            } else {
                let Some(c) = before else {
                    return;
                };
                let r = c.text_range();
                if c.is_hard_line_break() || c.is_emoji() {
                    r
                } else {
                    let Some((start, _)) = buffer
                        .get(..r.end)
                        .and_then(|t| t.char_indices().next_back())
                    else {
                        return;
                    };
                    start..r.end
                }
            }
        } else {
            edit.selection.text_range()
        };
        if !range.is_empty() {
            self.replace(range, "", None);
        }
    }

    /// The text of the area being edited, as the document holds it.
    fn edited_text(&self) -> Option<String> {
        match self.node(self.editing()?)? {
            Node::Area(a) => match &a.content {
                Content::Text(t) => Some(t.plain_text()),
                _ => None,
            },
            Node::Split(_) => None,
        }
    }

    /// Set the selection from the area's current layout.
    fn with_layout(
        &mut self,
        f: impl FnOnce(Selection, &parley::Layout<rootpoint_render::TextBrush>) -> Selection,
    ) {
        let Some(area) = self.editing() else {
            return;
        };
        let Some(text) = self.area_text(area) else {
            return;
        };
        if let Some(edit) = &mut self.edit {
            let s = edit.selection.refresh(&text.layout);
            edit.selection = f(s, &text.layout);
        }
    }

    fn at_point(
        &mut self,
        p: Pos2,
        layout: &Layout,
        f: impl FnOnce(Selection, &parley::Layout<rootpoint_render::TextBrush>, f32, f32) -> Selection,
    ) {
        let Some(area) = self.editing() else {
            return;
        };
        let Some(text) = self.area_text(area) else {
            return;
        };
        let (x, y) = layout_point(&text, layout, p);
        if let Some(edit) = &mut self.edit {
            let s = edit.selection.refresh(&text.layout);
            edit.selection = f(s, &text.layout, x, y);
        }
    }

    fn range_rects(
        &mut self,
        layout: &Layout,
        range: impl FnOnce(&Editing) -> Option<Range<usize>>,
    ) -> Vec<Rect> {
        let Some(area) = self.editing() else {
            return Vec::new();
        };
        let Some(text) = self.area_text(area) else {
            return Vec::new();
        };
        let Some(r) = self.edit.as_ref().and_then(range).filter(|r| !r.is_empty()) else {
            return Vec::new();
        };
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
    fn area_text(&mut self, area: NodeId) -> Option<AreaText> {
        let slide = self.shown_slide()?;
        self.assets.refresh(&self.session, &mut *self.files);
        let (fonts, _) = self.assets.sets();
        area_text(self.session.document(), slide, area, fonts)
    }
}

/// A point of the slide view, in the coordinates of a text's layout: slide units from the
/// layout's origin.
fn layout_point(text: &AreaText, layout: &Layout, p: Pos2) -> (f32, f32) {
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

/// A byte range kept within a text and on its character boundaries.
fn clamp(range: Range<usize>, text: &str) -> Range<usize> {
    let fix = |mut i: usize| {
        i = i.min(text.len());
        while !text.is_char_boundary(i) {
            i -= 1;
        }
        i
    };
    let (start, end) = (fix(range.start), fix(range.end));
    start.min(end)..end.max(start)
}
