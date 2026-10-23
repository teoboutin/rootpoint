//! The panels around the slide view: the toolbar, the slide list, and the side panel with the
//! breadcrumb, the actions on the selection, and the selected area's content and style.

use eframe::egui::{self, Align2, Color32, FontId, Sense, Stroke, Ui, Vec2};
use rootpoint_model::{
    AssetId, AssetSource, Axis, Content, FontRef, HAlign, ImageFit, Node, Rgba, SlideId, VAlign,
};
use rootpoint_protocol::StyleOption;

use crate::App;
use crate::hit::Edge;
use crate::selection;

/// The sizes of the size menu, in slide units. A menu, not a drag field: each choice is one
/// `text` command and one undo step, where a drag field would send one each frame of its drag.
const SIZES: [f64; 10] = [
    24.0, 32.0, 40.0, 48.0, 64.0, 80.0, 96.0, 120.0, 160.0, 200.0,
];

/// The colours of the colour row: one click is one command and one undo step, where a colour
/// picker sends one each frame of its drag.
const COLOURS: [(u8, u8, u8); 8] = [
    (0, 0, 0),
    (255, 255, 255),
    (96, 96, 96),
    (200, 30, 30),
    (230, 120, 0),
    (30, 140, 50),
    (30, 80, 200),
    (120, 40, 160),
];

impl App {
    /// The toolbar: the file actions, the sample, undo and redo.
    pub(crate) fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Open…").clicked() {
                self.open_dialog();
            }
            // The browser build saves no file, so it shows no button that saves, per
            // `design@rootpoint-app@browser-load-only`.
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Save").clicked() {
                self.save();
            }
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Save as…").clicked() {
                self.save_dialog();
            }
            if ui.button("Sample").clicked() {
                self.open_sample();
            }
            ui.separator();
            if ui
                .add_enabled(self.shown_slide().is_some(), egui::Button::new("Present"))
                .clicked()
            {
                self.present();
            }
            ui.separator();
            if ui.button("Undo").clicked() {
                self.undo();
            }
            if ui.button("Redo").clicked() {
                self.redo();
            }
        });
    }

    /// The slide list: a row per slide, which a click shows and a drag moves, and the buttons
    /// that add a slide after the shown one, and remove or duplicate the shown one. A drag moves a
    /// slide to any place at once, where move-up and move-down buttons cost a click per place.
    pub(crate) fn slide_list(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Add").clicked() {
                self.add_slide();
            }
            let any = self.shown_slide().is_some();
            if ui.add_enabled(any, egui::Button::new("Remove")).clicked() {
                self.remove_slide();
            }
            if ui
                .add_enabled(any, egui::Button::new("Duplicate"))
                .clicked()
            {
                self.duplicate_slide();
            }
        });
        ui.separator();
        let shown = self.shown_slide();
        let rows: Vec<(SlideId, String)> = self
            .session
            .document()
            .slides
            .iter()
            .enumerate()
            .map(|(i, s)| {
                (
                    s.id,
                    format!("{}  {}  {}", i + 1, s.id, first_text(&s.root)),
                )
            })
            .collect();
        let visuals = ui.visuals().clone();
        self.view.rows.clear();
        for (i, (id, label)) in rows.into_iter().enumerate() {
            // One widget senses both the click that shows the slide and the drag that moves it.
            // Inside `Ui::dnd_drag_source`, which senses the drag with a second widget over its
            // contents, a click on a row inside it did not show its slide: the row test of
            // `path@rootpoint-app@tests/gestures.rs` failed at its click.
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), 22.0),
                Sense::click_and_drag(),
            );
            self.view.rows.push((id, rect));
            if Some(id) == shown {
                ui.painter()
                    .rect_filled(rect, 2.0, visuals.selection.bg_fill);
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(rect, 2.0, visuals.widgets.hovered.weak_bg_fill);
            }
            ui.painter().text(
                rect.left_center() + Vec2::new(6.0, 0.0),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(14.0),
                visuals.text_color(),
            );
            if response.dragged() {
                response.dnd_set_drag_payload(id);
            }
            // While the socket holds control a row moves nothing, so it shows no drop line.
            if !self.socket_holds()
                && response
                    .dnd_hover_payload::<SlideId>()
                    .is_some_and(|p| *p != id)
            {
                ui.painter().hline(
                    rect.x_range(),
                    rect.center().y,
                    Stroke::new(2.0, visuals.selection.stroke.color),
                );
            }
            if let Some(slide) = response.dnd_release_payload::<SlideId>() {
                if *slide != id {
                    self.move_slide(*slide, i);
                }
            } else if response.clicked() {
                self.show_slide(id);
            }
        }
    }

    /// The side panel: the breadcrumb, the actions on the selection, and the selected area's
    /// content.
    pub(crate) fn side_panel(&mut self, ui: &mut Ui) {
        self.breadcrumb(ui);
        if self.session.view().selected.is_none() {
            return;
        }
        ui.separator();
        ui.label("Split, the new area:");
        ui.horizontal(|ui| {
            for (edge, text) in [
                (Edge::Left, "left"),
                (Edge::Right, "right"),
                (Edge::Up, "above"),
                (Edge::Down, "below"),
            ] {
                let can = self.can_split_middle(edge.split().0);
                if ui.add_enabled(can, egui::Button::new(text)).clicked() {
                    self.split_middle(edge);
                }
            }
        });
        if ui.button("Remove").clicked() {
            self.remove_selection();
        }
        if let Some(content) = self.selected_content().cloned() {
            ui.separator();
            self.content(ui, &content);
        }
    }

    /// The path from the shown slide's root to the selection, for instance
    /// `s1 › n4 beside › n8 stacked › n7 area`; a click on an entry selects it.
    fn breadcrumb(&mut self, ui: &mut Ui) {
        ui.heading("Selection");
        let Some(slide) = self.shown_slide() else {
            return;
        };
        let mut entries = vec![(None, slide.to_string())];
        if let Some(node) = self.session.view().selected
            && let Some(s) = self.session.document().slide(slide)
            && let Some(path) = selection::path(s, node)
        {
            entries.extend(path.iter().map(|n| {
                let kind = match n {
                    Node::Split(s) if s.axis == Axis::LeftToRight => "beside",
                    Node::Split(_) => "stacked",
                    Node::Area(_) => "area",
                };
                (Some(n.id()), format!("{} {kind}", n.id()))
            }));
        }
        let mut clicked = None;
        ui.horizontal_wrapped(|ui| {
            for (i, (node, text)) in entries.iter().enumerate() {
                if i > 0 {
                    ui.label("›");
                }
                if ui.link(text).clicked() {
                    clicked = Some(*node);
                }
            }
        });
        match clicked {
            Some(Some(node)) => self.select_node(node),
            Some(None) => self.show_slide(slide),
            None => {}
        }
    }

    /// The selected area's content: its kind, then the text's style, or the image and its fit; the
    /// text itself is edited in place, on the slide.
    fn content(&mut self, ui: &mut Ui, content: &Content) {
        ui.horizontal(|ui| {
            ui.label("Content:");
            if ui
                .selectable_label(matches!(content, Content::Text(_)), "text")
                .clicked()
                && !matches!(content, Content::Text(_))
            {
                self.set_text_content();
            }
            let images = self.image_assets();
            let image = ui.add_enabled(
                !images.is_empty(),
                egui::Button::selectable(matches!(content, Content::Image(_)), "image"),
            );
            if image.clicked()
                && !matches!(content, Content::Image(_))
                && let Some(first) = images.first()
            {
                self.set_image(*first, ImageFit::Fit);
            }
        });
        if ui.button("Add an image…").clicked() {
            self.add_asset_dialog();
        }
        match content {
            Content::Text(t) => {
                self.style(ui, &t.style);
            }
            Content::Image(i) => self.image(ui, i.asset, i.fit),
        }
    }

    /// The style of the selected text: each change sends one option.
    fn style(&mut self, ui: &mut Ui, style: &rootpoint_model::TextStyle) {
        let mut chosen = None;
        ui.horizontal(|ui| {
            egui::ComboBox::from_label("size")
                .selected_text(format!("{}", style.size))
                .show_ui(ui, |ui| {
                    for size in SIZES {
                        if ui
                            .selectable_label(style.size == size, format!("{size}"))
                            .clicked()
                        {
                            chosen = Some(StyleOption::Size(size));
                        }
                    }
                });
            let mut bold = style.bold;
            if ui.checkbox(&mut bold, "bold").changed() {
                chosen = Some(StyleOption::Bold(bold));
            }
            let mut italic = style.italic;
            if ui.checkbox(&mut italic, "italic").changed() {
                chosen = Some(StyleOption::Italic(italic));
            }
        });
        ui.horizontal(|ui| {
            for (r, g, b) in COLOURS {
                let colour = Rgba { r, g, b, a: 255 };
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
                ui.painter()
                    .rect_filled(rect, 2.0, Color32::from_rgb(r, g, b));
                if style.colour == colour {
                    ui.painter().rect_stroke(
                        rect,
                        2.0,
                        Stroke::new(2.0, ui.visuals().selection.stroke.color),
                        egui::StrokeKind::Outside,
                    );
                }
                if response.clicked() {
                    chosen = Some(StyleOption::Colour(colour));
                }
            }
        });
        ui.horizontal(|ui| {
            for (align, text) in [
                (HAlign::Left, "left"),
                (HAlign::Center, "centre"),
                (HAlign::Right, "right"),
            ] {
                if ui.selectable_label(style.h_align == align, text).clicked() {
                    chosen = Some(StyleOption::Align(align));
                }
            }
            ui.separator();
            for (align, text) in [
                (VAlign::Top, "top"),
                (VAlign::Middle, "middle"),
                (VAlign::Bottom, "bottom"),
            ] {
                if ui.selectable_label(style.v_align == align, text).clicked() {
                    chosen = Some(StyleOption::VAlign(align));
                }
            }
        });
        let fonts = self.font_assets();
        if !fonts.is_empty() {
            let current = match &style.font {
                FontRef::Asset(id) => id.to_string(),
                FontRef::Builtin(name) => name.clone(),
            };
            egui::ComboBox::from_label("font")
                .selected_text(current)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(matches!(style.font, FontRef::Builtin(_)), "sans")
                        .clicked()
                    {
                        chosen = Some(StyleOption::Font(None));
                    }
                    for (id, name) in fonts {
                        let on = style.font == FontRef::Asset(id);
                        if ui.selectable_label(on, format!("{id} {name}")).clicked() {
                            chosen = Some(StyleOption::Font(Some(id)));
                        }
                    }
                });
        }
        if let Some(option) = chosen {
            self.set_style(option);
        }
    }

    /// The selected image: its asset and its fit.
    fn image(&mut self, ui: &mut Ui, asset: AssetId, fit: ImageFit) {
        let mut chosen = None;
        egui::ComboBox::from_label("image")
            .selected_text(asset.to_string())
            .show_ui(ui, |ui| {
                for id in self.image_assets() {
                    let name = self.asset_name(id);
                    if ui
                        .selectable_label(id == asset, format!("{id} {name}"))
                        .clicked()
                    {
                        chosen = Some((id, fit));
                    }
                }
            });
        ui.horizontal(|ui| {
            for (f, text) in [(ImageFit::Fit, "fit"), (ImageFit::Fill, "fill")] {
                if ui.selectable_label(fit == f, text).clicked() && fit != f {
                    chosen = Some((asset, f));
                }
            }
        });
        if let Some((asset, fit)) = chosen {
            self.set_image(asset, fit);
        }
    }

    /// The assets that are not font files, which the image menu offers.
    fn image_assets(&self) -> Vec<AssetId> {
        let fonts: Vec<AssetId> = self.font_assets().into_iter().map(|f| f.0).collect();
        self.session
            .document()
            .assets
            .keys()
            .filter(|id| !fonts.contains(id))
            .copied()
            .collect()
    }

    /// The assets whose file is a TrueType or OpenType font, by its name or its first bytes. Only
    /// these are offered as fonts: any other asset chosen as a font would draw in the built-in sans.
    fn font_assets(&self) -> Vec<(AssetId, String)> {
        self.session
            .document()
            .assets
            .iter()
            .filter(|(_, a)| match &a.source {
                AssetSource::Path(p) => {
                    let p = p.to_ascii_lowercase();
                    p.ends_with(".ttf") || p.ends_with(".otf")
                }
                AssetSource::Embedded(b) => {
                    b.starts_with(&[0, 1, 0, 0]) || b.starts_with(b"OTTO") || b.starts_with(b"true")
                }
            })
            .map(|(id, _)| (*id, self.asset_name(*id)))
            .collect()
    }

    /// An asset's file name, or "embedded".
    fn asset_name(&self, id: AssetId) -> String {
        match self.session.document().assets.get(&id).map(|a| &a.source) {
            Some(AssetSource::Path(p)) => p.rsplit('/').next().unwrap_or(p).to_owned(),
            Some(AssetSource::Embedded(_)) => "embedded".to_owned(),
            None => String::new(),
        }
    }
}

/// The start of a slide's first text, for its row in the slide list.
fn first_text(root: &Node) -> String {
    let text = root
        .areas()
        .into_iter()
        .find_map(|a| match &a.content {
            Content::Text(t) if !t.text.trim().is_empty() => Some(t.text.as_str()),
            _ => None,
        })
        .unwrap_or("");
    let line = text.lines().next().unwrap_or("");
    let mut out: String = line.chars().take(24).collect();
    if line.chars().count() > 24 {
        out.push('…');
    }
    out
}
