//! Editing an area's text in place: the caret and the selection are Parley's `Selection`, placed on
//! the very layout `render_slide` draws for the area, which `rootpoint_render::area_text` gives, so
//! the caret sits on the drawn glyphs at every size of the view.

use eframe::egui::{Pos2, Rect};
use parley::{Affinity, Selection};
use rootpoint_model::{Content, Node, NodeId};
use rootpoint_render::{AreaText, area_text};

use crate::App;
use crate::hit::Layout;

/// The width of the drawn caret, in points.
pub const CARET_WIDTH: f32 = 2.0;

/// The text being edited in place: its area, and the caret and the selection on its layout.
#[derive(Clone, Debug)]
pub(crate) struct Editing {
    pub area: NodeId,
    pub selection: Selection,
}

/// A move of the caret with no change of the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaretMove {
    Left,
    Right,
    Up,
    Down,
    LineStart,
    LineEnd,
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
    /// the selected text area, so a first click selects and a second edits.
    pub fn edit_at(&mut self, area: NodeId, p: Pos2, layout: &Layout) {
        if !self.is_text_area(area) {
            return;
        }
        let Some(text) = self.area_text(area) else {
            return;
        };
        let (x, y) = layout_point(&text, layout, p);
        self.edit = Some(Editing {
            area,
            selection: Selection::from_point(&text.layout, x, y),
        });
    }

    /// Place the caret of the text being edited at the point `p` of the slide view.
    pub fn place_caret(&mut self, p: Pos2, layout: &Layout) {
        let Some(area) = self.editing() else {
            return;
        };
        let Some(text) = self.area_text(area) else {
            return;
        };
        let (x, y) = layout_point(&text, layout, p);
        if let Some(edit) = &mut self.edit {
            edit.selection = Selection::from_point(&text.layout, x, y);
        }
    }

    /// Leave editing; the area stays selected.
    pub fn stop_editing(&mut self) {
        self.edit = None;
    }

    /// Move the caret, collapsing any selection.
    pub fn move_caret(&mut self, m: CaretMove) {
        let Some(area) = self.editing() else {
            return;
        };
        let Some(text) = self.area_text(area) else {
            return;
        };
        let Some(edit) = &mut self.edit else {
            return;
        };
        let s = edit.selection.refresh(&text.layout);
        let l = &text.layout;
        edit.selection = match m {
            CaretMove::Left => s.previous_visual(l, false),
            CaretMove::Right => s.next_visual(l, false),
            CaretMove::Up => s.previous_line(l, false),
            CaretMove::Down => s.next_line(l, false),
            CaretMove::LineStart => s.line_start(l, false),
            CaretMove::LineEnd => s.line_end(l, false),
        };
    }

    /// Put the caret at a byte offset of the text, as a test does to visit every offset.
    pub fn set_caret(&mut self, index: usize) {
        let Some(area) = self.editing() else {
            return;
        };
        let Some(text) = self.area_text(area) else {
            return;
        };
        if let Some(edit) = &mut self.edit {
            edit.selection = Selection::from_byte_index(&text.layout, index, Affinity::Downstream);
        }
    }

    /// The byte offset of the caret in the text being edited.
    pub fn caret_index(&self) -> Option<usize> {
        self.edit.as_ref().map(|e| e.selection.focus().index())
    }

    /// The caret's rectangle in the slide view, in points, `CARET_WIDTH` wide, placed through the
    /// area's layout and its origin as `render_slide` draws them.
    pub fn caret_rect(&mut self, layout: &Layout) -> Option<Rect> {
        let area = self.editing()?;
        let text = self.area_text(area)?;
        let edit = self.edit.as_ref()?;
        let focus = edit.selection.refresh(&text.layout).focus();
        let width = CARET_WIDTH / layout.points_per_unit();
        let b = focus.geometry(&text.layout, width);
        let min = layout.to_points(text.origin.0 + b.x0, text.origin.1 + b.y0);
        let max = layout.to_points(text.origin.0 + b.x1, text.origin.1 + b.y1);
        Some(Rect::from_min_max(min, max))
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
