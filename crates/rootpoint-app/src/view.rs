//! The slide view: the shown slide's texture, the pointer's gestures on it, and the editor's marks
//! drawn over it.

use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, Key, PointerButton, Pos2, Rect, Sense, Shape,
    Stroke, StrokeKind, Ui, Vec2,
};
use rootpoint_model::Axis;

use crate::hit::{Layout, Press, Zone};
use crate::{App, Gesture, texture_size};

/// How much further out each outer frame of the caret's spans is drawn than the one inside it, in
/// points.
const FRAME_GAP: f32 = 2.0;

impl App {
    /// The shown slide, scaled uniformly to the largest size that fits the view, letterboxed:
    /// `render_slide` draws it so, at the view's size in physical pixels. The pointer's presses
    /// in it are taken first, with the layout the frame starts from, then the slide is drawn as
    /// they left it, and the editor's marks over it.
    pub(crate) fn slide_view(&mut self, ui: &mut Ui) {
        let rect = ui.max_rect();
        self.view.layout = None;
        let Some(slide) = self.shown_slide() else {
            ui.centered_and_justified(|ui| ui.label("No slide: open a presentation."));
            return;
        };
        let ctx = ui.ctx().clone();
        let size = texture_size(&ctx, rect);
        if size[0] == 0 || size[1] == 0 {
            return;
        }
        let response = ui.interact(
            rect,
            egui::Id::unique("slide view"),
            Sense::click_and_drag(),
        );
        if let Some(layout) = Layout::new(self.session.document(), slide, rect, size) {
            self.pointer(ui, &response, &layout);
        }
        // A command may have removed the slide, or shown another.
        let Some(slide) = self.shown_slide() else {
            return;
        };
        if self.view.drawn != Some((slide, size)) {
            self.draw(&ctx, slide, size);
        }
        if let Some(texture) = &self.view.texture {
            let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
            ui.painter().image(texture.id(), rect, uv, Color32::WHITE);
        }
        self.view.layout = Layout::new(self.session.document(), slide, rect, size);
        // The final view draws no mark over the slide.
        if !self.final_view()
            && let Some(layout) = self.view.layout.clone()
        {
            self.overlay(ui, &layout);
        }
    }

    /// The axis of the split at the pointer that the held keys ask for, Ctrl a vertical line, which
    /// splits beside, and Shift a horizontal one, which stacks, and whether Alt snaps it; `None`
    /// with neither key held, after Esc until both are released, while a text is edited, while the
    /// socket holds control, or in the final view.
    fn split_keys(&mut self, ui: &Ui) -> Option<(Axis, bool)> {
        if self.socket_holds() || self.final_view() {
            return None;
        }
        let m = ui.input(|i| i.modifiers);
        let axis = if m.command {
            Some(Axis::LeftToRight)
        } else if m.shift {
            Some(Axis::TopToBottom)
        } else {
            None
        };
        if axis.is_none() {
            self.preview_hidden = false;
        }
        // While a text is edited, Shift extends its selection and splits nothing.
        if self.edit.is_some() {
            return None;
        }
        axis.filter(|_| !self.preview_hidden).map(|a| (a, m.alt))
    }

    /// The press, the drag and the release of the primary button in the slide view, and Esc; and
    /// the secondary click, which opens the node menu.
    fn pointer(&mut self, ui: &Ui, response: &egui::Response, layout: &Layout) {
        // A secondary click opens the menu on the area under the pointer, on a divider's band or
        // on the grip too, since no drag starts on the secondary button; on the letterbox it only
        // ends a stay in editing, as any press outside the edited area does. During a drag it does
        // nothing, so the menu never opens on a node the drag is changing,
        // `design@rootpoint-app@node-menu`.
        if response.secondary_clicked()
            && self.gesture == Gesture::Idle
            && !self.text_drag
            && let Some(at) = response.interact_pointer_pos()
        {
            match layout.area_at(at) {
                Some(area) => self.open_node_menu(area, at),
                None if self.edit.is_some() => self.stop_editing(),
                None => {}
            }
        }
        let (pressed, origin, down, released, pos, double, esc, alt) = ui.input(|i| {
            (
                i.pointer.primary_pressed(),
                i.pointer.press_origin(),
                i.pointer.primary_down(),
                i.pointer.primary_released(),
                i.pointer.interact_pos(),
                i.pointer.button_double_clicked(PointerButton::Primary),
                i.key_pressed(Key::Escape),
                i.modifiers.alt,
            )
        });
        // In the final view a press starts no gesture, `design@rootpoint-app@final-view-presses`,
        // and a click on an area selects it, which ends the final view.
        if self.final_view() {
            if response.clicked()
                && let Some(at) = pos
                && let Some(area) = layout.area_at(at)
            {
                self.select_node(area);
                self.left_final_view = Some(ui.input(|i| i.time));
            }
            return;
        }
        // The second press of a double-click whose first click left the final view starts
        // nothing: the author double-clicked with no mark in view, so neither half resets a
        // divider, enters editing or splits, `design@rootpoint-app@edit-entry`.
        if pressed && let Some(left) = self.left_final_view.take() {
            let delay = ui.ctx().options(|o| o.input_options.max_double_click_delay);
            if ui.input(|i| i.time) - left < delay {
                return;
            }
        }
        let split = self.split_keys(ui);
        if esc && split.is_some() && self.gesture == Gesture::Idle {
            self.preview_hidden = true;
        }
        let selected = self.session.view().selected;
        // egui reports a drag only once the pointer has moved, so the press is read from its
        // origin, not from the frame `drag_started` is set.
        if pressed
            && let Some(origin) = origin
            && response.rect.contains(origin)
            && response.contains_pointer()
        {
            // The edited area has no grip: it would cover the first characters of its text.
            let press = layout.press(
                origin,
                selected.filter(|&a| Some(a) != self.editing()),
                split,
            );
            self.press = Some(press);
            // While a text is edited, a press inside its area moves the caret; any other press
            // leaves editing, then acts as it would.
            if let Some(editing) = self.editing() {
                if press == Press::Area(editing) {
                    if ui.input(|i| i.modifiers.shift) {
                        self.shift_click(origin, layout);
                    } else {
                        self.place_caret(origin, layout);
                    }
                    self.press = None;
                    self.text_drag = true;
                    return;
                }
                self.stop_editing();
            }
            // A click on the selected text area enters editing at the click; a first click on an
            // area only selects it, so a double-click on a text area does both.
            self.edit_on_click =
                matches!(press, Press::Area(a) if Some(a) == selected && self.is_text_area(a))
                    .then_some(origin);
            match press {
                Press::Grip(area) => self.grab_grip(area),
                Press::Divider(i) => {
                    let d = &layout.dividers[i];
                    // The press may be up to `GRAB` points off the line: the drag keeps that
                    // offset, so the divider does not jump to the pointer.
                    self.grab_offset = match d.axis {
                        Axis::LeftToRight => Vec2::new(d.from.x - origin.x, 0.0),
                        Axis::TopToBottom => Vec2::new(0.0, d.from.y - origin.y),
                    };
                    self.grab_divider(d.split, d.index);
                }
                Press::Split(_) | Press::Area(_) | Press::Nothing => {}
            }
        }
        // A drag that began inside the edited text selects it.
        if self.text_drag {
            if down && let Some(pos) = pos {
                self.extend_to_point(pos, layout);
            }
            if released || !down {
                self.text_drag = false;
            }
            return;
        }
        match self.gesture {
            Gesture::Divider { .. } | Gesture::Grip { .. } if esc => self.cancel_drag(),
            Gesture::Divider {
                split, index, left, ..
            } => {
                // Until the pointer leaves the press, nothing is sent, so a click or a
                // double-click on a divider leaves no step of its own. Once it has left, a return
                // to the press's place sends too, and puts the divider back.
                if let Gesture::Divider { left, .. } = &mut self.gesture
                    && pos.is_some()
                    && pos != origin
                {
                    *left = true;
                }
                if down
                    && let Some(pos) = pos
                    && (left || Some(pos) != origin)
                    && let Some(i) = layout
                        .dividers
                        .iter()
                        .position(|d| d.split == split && d.index == index)
                    && let Some(at) = layout.drag_position(i, pos + self.grab_offset, alt)
                {
                    self.drag_divider(at);
                }
            }
            Gesture::Grip { area } if released => {
                let drop = pos
                    .and_then(|p| layout.drop(p, area))
                    .map(|d| (d.target, d.zone));
                self.drop_grip(drop);
            }
            Gesture::Grip { .. } | Gesture::Idle | Gesture::Cancelled => {}
        }
        if released {
            let press = self.press.take();
            self.release_drag();
            match press {
                Some(Press::Area(area)) if response.clicked() && self.edit_on_click.is_some() => {
                    if let Some(at) = self.edit_on_click.take() {
                        self.edit_at(area, at, layout);
                    }
                }
                Some(Press::Area(area)) if response.clicked() => self.select_node(area),
                // Releasing the key before the button removes the line and issues nothing.
                Some(Press::Split(preview))
                    if response.clicked() && split.map(|s| s.0) == Some(preview.axis) =>
                {
                    self.split_at(preview)
                }
                Some(Press::Divider(i)) if double => {
                    let d = &layout.dividers[i];
                    self.reset_divider(d.split, d.index);
                }
                _ => {}
            }
        }
    }

    /// The editor's marks, drawn with egui over the slide's texture and never into its image: a
    /// line on each divider, a highlight on the divider under the pointer or being dragged, a thin
    /// outline on the node of the outline's row under the pointer, an outline and a grip on the
    /// selection, the preview line of a split, and the hint of a drop.
    fn overlay(&mut self, ui: &Ui, layout: &Layout) {
        let painter = ui.painter();
        let accent = ui.visuals().selection.stroke.color;
        let hover = ui.input(|i| i.pointer.hover_pos());
        let selected = self.session.view().selected;
        let split = self.split_keys(ui);
        let active = match self.gesture {
            Gesture::Divider { split, index, .. } => layout
                .dividers
                .iter()
                .position(|d| d.split == split && d.index == index),
            // While the socket holds control, no divider offers a drag.
            Gesture::Idle if split.is_none() && !self.socket_holds() => {
                hover.and_then(|p| layout.divider_at(p))
            }
            _ => None,
        };
        for (i, d) in layout.dividers.iter().enumerate() {
            let stroke = if Some(i) == active {
                Stroke::new(3.0, accent)
            } else {
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(128, 128, 128, 160))
            };
            painter.line_segment([d.from, d.to], stroke);
        }
        if let Some(i) = active {
            ui.ctx().set_cursor_icon(match layout.dividers[i].axis {
                Axis::LeftToRight => CursorIcon::ResizeHorizontal,
                Axis::TopToBottom => CursorIcon::ResizeVertical,
            });
        }
        // The node of the outline's row under the pointer, so the author sees which part of the
        // slide a row names.
        if let Some(node) = self.view.hovered_row
            && let Some(r) = layout.rect(node)
        {
            painter.rect_stroke(r, 0.0, Stroke::new(1.0, accent), StrokeKind::Inside);
        }
        if let Some(node) = selected
            && let Some(r) = layout.rect(node)
        {
            painter.rect_stroke(r, 0.0, Stroke::new(2.0, accent), StrokeKind::Inside);
            if self.editing() != Some(node)
                && !self.socket_holds()
                && let Some(grip) = layout.grip(node)
            {
                painter.rect_filled(grip, 2.0, accent);
                if hover.is_some_and(|p| grip.contains(p)) && self.gesture == Gesture::Idle {
                    ui.ctx().set_cursor_icon(CursorIcon::Grab);
                }
            }
        }
        if self.gesture == Gesture::Idle
            && let (Some(node), Some((axis, snap)), Some(p)) = (selected, split, hover)
            && layout.rect(node).is_some_and(|r| r.contains(p))
            && let Some(preview) = layout.preview(node, axis, p, snap)
        {
            painter.line_segment([preview.from, preview.to], Stroke::new(2.0, accent));
        }
        if let Gesture::Grip { area } = self.gesture {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            if let Some(p) = hover
                && let Some(drop) = layout.drop(p, area)
            {
                let tint = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 70);
                painter.add(Shape::convex_polygon(
                    drop.shape,
                    tint,
                    Stroke::new(1.0, accent),
                ));
                let label = match drop.zone {
                    Zone::Swap => "swap",
                    Zone::Move(edge) => edge.label(),
                };
                let at = p + Vec2::new(14.0, -14.0);
                let galley = painter.layout_no_wrap(
                    label.to_owned(),
                    FontId::proportional(14.0),
                    Color32::WHITE,
                );
                let back = Align2::LEFT_BOTTOM
                    .anchor_size(at, galley.size())
                    .expand(4.0);
                painter.rect_filled(back, 3.0, Color32::from_black_alpha(200));
                painter.galley(back.min + Vec2::splat(4.0), galley, Color32::WHITE);
            }
        }
        // The selection, the composing text's underline and the caret of a text edited in place,
        // and the caret's place for the IME, per `design@rootpoint-app@edit-in-place`; and a thin
        // frame around each span that holds the caret, which takes no space in the slide: each
        // outer frame lighter, and set one point further out, so that frames meeting at an edge
        // stay apart.
        if self.edit.is_some() {
            for frame in self.span_frames(layout) {
                let alpha = 230u8.saturating_sub(60 * frame.depth.min(3) as u8);
                let colour =
                    Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), alpha);
                let out = FRAME_GAP * frame.depth as f32;
                for r in frame.rects {
                    ui.painter().rect_stroke(
                        r.expand(out),
                        0.0,
                        Stroke::new(1.0, colour),
                        StrokeKind::Outside,
                    );
                }
            }
            let tint = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 70);
            for r in self.selection_rects(layout) {
                ui.painter().rect_filled(r, 0.0, tint);
            }
            for r in self.preedit_rects(layout) {
                ui.painter()
                    .hline(r.x_range(), r.max.y, Stroke::new(1.5, accent));
            }
            if let Some(caret) = self.caret_rect(layout) {
                // In the colour the caret types with, so that the two positions at the edge of a
                // span of another colour are told apart.
                let colour = self.caret_values().map_or(accent, |v| {
                    Color32::from_rgba_unmultiplied(v.colour.r, v.colour.g, v.colour.b, 255)
                });
                ui.painter().rect_filled(caret, 0.0, colour);
                let area = self.editing().and_then(|a| layout.rect(a)).unwrap_or(caret);
                ui.ctx().output_mut(|o| {
                    o.ime = Some(egui::output::IMEOutput {
                        purpose: egui::IMEPurpose::Normal,
                        rect: area,
                        cursor_rect: caret,
                        should_interrupt_composition: false,
                    });
                });
            }
        }
    }
}
