//! The slide view: the shown slide's texture, the pointer's gestures on it, and the editor's marks
//! drawn over it.

use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, Key, PointerButton, Pos2, Rect, Sense, Shape,
    Stroke, StrokeKind, Ui, Vec2,
};
use rootpoint_model::Axis;

use crate::hit::{Layout, Press, Zone};
use crate::{App, Gesture, texture_size};

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
        if let Some(layout) = self.view.layout.clone() {
            self.overlay(ui, &layout);
        }
    }

    /// The axis of the split at the pointer that the held keys ask for, Ctrl a line across the
    /// width and Shift one across the height, and whether Alt snaps it; `None` with neither key
    /// held, or after Esc until both are released.
    fn split_keys(&mut self, ui: &Ui) -> Option<(Axis, bool)> {
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
        axis.filter(|_| !self.preview_hidden).map(|a| (a, m.alt))
    }

    /// The press, the drag and the release of the primary button in the slide view, and Esc.
    fn pointer(&mut self, ui: &Ui, response: &egui::Response, layout: &Layout) {
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
            let press = layout.press(origin, selected, split);
            self.press = Some(press);
            match press {
                Press::Grip(area) => self.grab_grip(area),
                Press::Divider(i) => {
                    let d = &layout.dividers[i];
                    self.grab_divider(d.split, d.index);
                }
                Press::Split(_) | Press::Area(_) | Press::Nothing => {}
            }
        }
        match self.gesture {
            Gesture::Divider { .. } | Gesture::Grip { .. } if esc => self.cancel_drag(),
            Gesture::Divider { split, index, .. } => {
                if down
                    && let Some(pos) = pos
                    && let Some(i) = layout
                        .dividers
                        .iter()
                        .position(|d| d.split == split && d.index == index)
                    && let Some(at) = layout.drag_position(i, pos, alt)
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
                Some(Press::Area(area)) if response.clicked() => self.select_node(area),
                Some(Press::Split(preview)) if response.clicked() => self.split_at(preview),
                Some(Press::Divider(i)) if double => {
                    let d = &layout.dividers[i];
                    self.reset_divider(d.split, d.index);
                }
                _ => {}
            }
        }
    }

    /// The editor's marks, drawn with egui over the slide's texture and never into its image: a
    /// line on each divider, a highlight on the divider under the pointer or being dragged, an
    /// outline and a grip on the selection, the preview line of a split, and the hint of a drop.
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
            Gesture::Idle if split.is_none() => hover.and_then(|p| layout.divider_at(p)),
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
        if let Some(node) = selected
            && let Some(r) = layout.rect(node)
        {
            painter.rect_stroke(r, 0.0, Stroke::new(2.0, accent), StrokeKind::Inside);
            if let Some(grip) = layout.grip(node) {
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
    }
}
