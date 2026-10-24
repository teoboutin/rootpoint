//! The panels around the slide view: the toolbar, the slide list, and the side panel, whose
//! Selection tab holds the outline of the shown slide's tree, the actions on the selection and the
//! selected area's content and style, and whose Styles tab edits the table of styles, with the
//! dialog of a style's removal.

use eframe::egui::{self, Align2, Color32, FontId, Sense, Stroke, Ui, Vec2};
use rootpoint_model::{
    AssetId, Axis, Breach, Content, FontRef, HAlign, ImageFit, Node, NodeId, Rgba, SlideId,
    StyleChoice, VAlign,
};
use rootpoint_protocol::StyleOption;

use crate::hit::Edge;
use crate::{App, OutlineRow, SideTab};

/// The sizes of the size menu, in slide units. A menu, not a drag field: each choice is one
/// `text` command and one undo step, where a drag field would send one each frame of its drag.
const SIZES: [f64; 11] = [
    24.0, 28.0, 32.0, 40.0, 48.0, 64.0, 80.0, 96.0, 120.0, 160.0, 200.0,
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
    /// The native toolbar's buttons that open a pack or write the presentation, each named by
    /// where it writes, so the author sees what each one does: in memory, a pack or a workspace; in
    /// a workspace, a pack. "Open pack…" is a button of its own beside Open, since the native file
    /// dialog picks a file or a directory, never either.
    #[cfg(not(target_arch = "wasm32"))]
    fn file_buttons(&mut self, ui: &mut Ui) {
        if ui.button("Open pack…").clicked() {
            self.open_pack_dialog();
        }
        if ui.button("Save").clicked() {
            self.save();
        }
        if self.session.root().is_some() {
            if ui.button("Pack to…").clicked() {
                self.pack_dialog();
            }
            return;
        }
        if ui.button("Save as pack…").clicked() {
            self.pack_dialog();
        }
        let unpack = if self.session.pack().is_some() {
            "Unpack to…"
        } else {
            "Save as workspace…"
        };
        if ui.button(unpack).clicked() {
            self.unpack_dialog();
        }
    }

    /// The toolbar: the file actions, the sample, the final view, presenting, undo and redo, and the
    /// state label at its right end.
    pub(crate) fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Open…").clicked() {
                self.open_dialog();
            }
            // The browser build saves no file, so it shows none of the buttons of a pack or a
            // save, per `design@rootpoint-app@browser-load-only`.
            #[cfg(not(target_arch = "wasm32"))]
            self.file_buttons(ui);
            if ui.button("Sample").clicked() {
                self.open_sample();
            }
            ui.separator();
            // The final view changes only the view, so it works while the socket holds control.
            let preview = ui.selectable_label(self.final_view(), "Preview");
            self.view.buttons.push(("Preview".to_owned(), preview.rect));
            if preview.clicked() {
                self.toggle_final_view();
            }
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
            // The state label, at the right end: where the presentation is, and whether it is
            // saved.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = ui.label(self.state_label());
                self.view.buttons.push(("state".to_owned(), label.rect));
            });
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

    /// A button whose place the frame records by its label, so a test can click it; it does nothing
    /// while `enabled` is false.
    fn button(&mut self, ui: &mut Ui, label: &str, enabled: bool) -> bool {
        let response = ui.add_enabled(enabled, egui::Button::new(label));
        self.view.buttons.push((label.to_owned(), response.rect));
        response.clicked()
    }

    /// A tab of the side panel, whose place the frame records by its label.
    fn tab(&mut self, ui: &mut Ui, tab: SideTab, label: &str) {
        let response = ui.selectable_label(self.tab == tab, label);
        self.view.buttons.push((label.to_owned(), response.rect));
        if response.clicked() {
            self.tab = tab;
        }
    }

    /// The side panel: its two tabs, the selection's and the styles'. While the socket holds the
    /// authority to edit, every control that sends an edit is disabled, and the outline and the
    /// list of styles, which change only what the window shows, still work, per
    /// `design@rootpoint-app@one-controller-at-a-time`.
    pub(crate) fn side_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.tab(ui, SideTab::Selection, "Selection");
            self.tab(ui, SideTab::Styles, "Styles");
        });
        ui.separator();
        match self.tab {
            SideTab::Selection => self.selection_tab(ui),
            SideTab::Styles => self.styles_tab(ui),
        }
    }

    /// The Selection tab: the outline, the actions on the selection, and the selected area's
    /// content.
    fn selection_tab(&mut self, ui: &mut Ui) {
        self.outline(ui);
        if self.session.view().selected.is_none() {
            return;
        }
        let editable = !self.socket_holds();
        ui.add_enabled_ui(editable, |ui| self.selection_actions(ui));
    }

    /// The actions on the selection and its content.
    fn selection_actions(&mut self, ui: &mut Ui) {
        ui.separator();
        ui.label("Split, the new area:");
        ui.horizontal_wrapped(|ui| {
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

    /// The shown slide's whole tree, one row per node in depth-first order, after a row for the slide
    /// itself, per `design@rootpoint-app@tree-outline`. A click on a row selects its node, the
    /// slide's row selecting none, and a secondary click also opens the node menu on it; the
    /// selected node's row is highlighted, and the overlay outlines the node of the row under the
    /// pointer. A row's text is cut at the panel's edge, never wrapped.
    /// The outline stays expanded, since a slide's tree is small and collapsing would add state.
    fn outline(&mut self, ui: &mut Ui) {
        ui.heading("Selection");
        self.view.outline.clear();
        self.view.hovered_row = None;
        let Some(slide) = self.shown_slide() else {
            return;
        };
        let doc = self.session.document();
        let Some(the_slide) = doc.slide(slide) else {
            return;
        };
        let missing: Vec<(NodeId, String)> = doc
            .breaches()
            .into_iter()
            .filter_map(|b| match b {
                Breach::MissingStyle { area, name } => Some((area, name)),
                _ => None,
            })
            .collect();
        let mut rows = vec![(None, 0, format!("Slide {slide}"))];
        fn walk(
            node: &Node,
            depth: usize,
            doc: &rootpoint_model::Document,
            missing: &[(NodeId, String)],
            rows: &mut Vec<(Option<NodeId>, usize, String)>,
        ) {
            let label = match node {
                Node::Split(s) => {
                    let (mark, axis) = match s.axis {
                        Axis::TopToBottom => ("[=]", "stacked"),
                        Axis::LeftToRight => ("[|]", "beside"),
                    };
                    format!("{mark} {}  {axis}", s.id)
                }
                Node::Area(a) => match &a.content {
                    Content::Text(t) => {
                        let style = match &t.style {
                            StyleChoice::Custom(_) => "custom".to_owned(),
                            StyleChoice::Named(name) if missing.iter().any(|m| m.0 == a.id) => {
                                format!("\u{26a0} {name}")
                            }
                            StyleChoice::Named(name) => name.clone(),
                        };
                        let start: String = t
                            .text
                            .lines()
                            .next()
                            .unwrap_or("")
                            .chars()
                            .take(40)
                            .collect();
                        format!("T  {}  {style}  \"{start}\"", a.id)
                    }
                    Content::Image(i) => {
                        let name = match doc.assets.get(&i.asset) {
                            Some(a) => a.path.rsplit('/').next().unwrap_or(&a.path).to_owned(),
                            None => format!("\u{26a0} {}", i.asset),
                        };
                        format!("I  {}  {name}", a.id)
                    }
                },
            };
            rows.push((Some(node.id()), depth, label));
            if let Node::Split(s) = node {
                for child in &s.children {
                    walk(child, depth + 1, doc, missing, rows);
                }
            }
        }
        walk(&the_slide.root, 1, doc, &missing, &mut rows);
        let selected = self.session.view().selected;
        let visuals = ui.visuals().clone();
        let mut clicked = None;
        let mut secondary = None;
        for (node, depth, label) in rows {
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::click());
            let here = node == selected;
            if here {
                ui.painter()
                    .rect_filled(rect, 2.0, visuals.selection.bg_fill);
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(rect, 2.0, visuals.widgets.hovered.weak_bg_fill);
            }
            if response.hovered() {
                self.view.hovered_row = node;
            }
            ui.painter().with_clip_rect(rect).text(
                rect.left_center() + Vec2::new(4.0 + 14.0 * depth as f32, 0.0),
                Align2::LEFT_CENTER,
                label.clone(),
                FontId::proportional(13.0),
                visuals.text_color(),
            );
            self.view.outline.push(OutlineRow {
                node,
                depth,
                rect,
                selected: here,
                label,
            });
            if response.clicked() {
                clicked = Some(node);
            }
            if response.secondary_clicked() {
                let at = response.interact_pointer_pos().unwrap_or(rect.center());
                secondary = Some((node, at));
            }
        }
        match clicked {
            Some(Some(node)) => self.select_node(node),
            Some(None) => self.show_slide(slide),
            None => {}
        }
        // A secondary click on a node's row opens the node menu on it; the slide's row names no
        // node, and shows its slide as its click does, `design@rootpoint-app@node-menu`.
        match secondary {
            Some((Some(node), at)) => self.open_node_menu(node, at),
            Some((None, _)) => self.show_slide(slide),
            None => {}
        }
    }

    /// The selected area's content: its kind, then the text's style, or the image and its fit; the
    /// text itself is edited in place, on the slide.
    fn content(&mut self, ui: &mut Ui, content: &Content) {
        ui.horizontal_wrapped(|ui| {
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
            // The style's name, or custom, then the values drawn, a named style's or the text's
            // own; a change of a value sends one option, which makes a named style custom, so the
            // Selection tab changes no shared style,
            // `design@rootpoint-app@styles-tab-edits-styles`.
            Content::Text(t) => {
                self.style_choice(ui, &t.style);
                let values = self.session.document().resolve_style(&t.style);
                if let Some(option) = self.style(ui, &values) {
                    self.set_style(option);
                }
            }
            Content::Image(i) => self.image(ui, i.asset, i.fit),
        }
    }

    /// The selected text's style: a style of the table, by name, or its own values, "custom".
    fn style_choice(&mut self, ui: &mut Ui, choice: &StyleChoice) {
        let names: Vec<String> = self.session.document().styles.keys().cloned().collect();
        let shown = match choice {
            StyleChoice::Named(name) if names.contains(name) => name.clone(),
            StyleChoice::Named(name) => format!("\u{26a0} {name}"),
            StyleChoice::Custom(_) => "custom".to_owned(),
        };
        let mut chosen = None;
        egui::ComboBox::from_label("style")
            .selected_text(shown)
            .show_ui(ui, |ui| {
                for name in names {
                    let on = *choice == StyleChoice::Named(name.clone());
                    if ui.selectable_label(on, &name).clicked() && !on {
                        chosen = Some(Some(name));
                    }
                }
                let custom = matches!(choice, StyleChoice::Custom(_));
                if ui.selectable_label(custom, "custom").clicked() && !custom {
                    chosen = Some(None);
                }
            });
        match chosen {
            Some(Some(name)) => self.name_style(name),
            Some(None) => self.make_custom(),
            None => {}
        }
    }

    /// The Styles tab: the table's styles, each with the number of texts that name it; a click
    /// selects one in the list, the window's own state, and the selected style's name and values
    /// below send `style rename` and `style set`; with New, Duplicate and Remove, they are the only
    /// controls of the window that change the table of styles,
    /// `design@rootpoint-app@styles-tab-edits-styles`.
    fn styles_tab(&mut self, ui: &mut Ui) {
        let styles: Vec<(String, usize)> = self
            .session
            .document()
            .styles
            .keys()
            .map(|name| (name.clone(), self.style_users(name)))
            .collect();
        if let Some(selected) = &self.styles_tab.selected
            && !styles.iter().any(|(name, _)| name == selected)
        {
            self.styles_tab.selected = None;
        }
        for (name, users) in &styles {
            let label = match users {
                1 => format!("{name}   1 area"),
                n => format!("{name}   {n} areas"),
            };
            let on = self.styles_tab.selected.as_ref() == Some(name);
            let response = ui.selectable_label(on, label);
            self.view
                .buttons
                .push((format!("style {name}"), response.rect));
            if response.clicked() {
                self.select_style(name);
            }
        }
        let editable = !self.socket_holds();
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            if self.button(ui, "New", editable) {
                self.add_style();
            }
            let any = self.styles_tab.selected.is_some();
            if self.button(ui, "Duplicate", editable && any) {
                self.duplicate_style();
            }
            if self.button(ui, "Remove", editable && any) {
                self.remove_style();
            }
        });
        let Some(name) = self.styles_tab.selected.clone() else {
            return;
        };
        let Some(values) = self.session.document().styles.get(&name).cloned() else {
            return;
        };
        ui.add_enabled_ui(editable, |ui| {
            ui.horizontal(|ui| {
                ui.label("name");
                let mut text = self
                    .styles_tab
                    .name_edit
                    .clone()
                    .unwrap_or_else(|| name.clone());
                let response = ui.add(egui::TextEdit::singleline(&mut text).desired_width(140.0));
                if response.changed() {
                    self.styles_tab.name_edit = Some(text.clone());
                }
                // The rename is sent once, when the field loses the keyboard or Enter is pressed,
                // so that typing a name is one command and one undo step.
                if response.lost_focus() {
                    self.styles_tab.name_edit = None;
                    self.rename_style(text);
                }
            });
            if let Some(option) = self.style(ui, &values) {
                self.set_selected_style(option);
            }
        });
    }

    /// The dialog of the removal of a style that texts name: what they take instead, a custom
    /// copy first, then each other style a text may name, since the command refuses any other,
    /// `design@rootpoint-protocol@no-new-breach`. It is a choice the removal needs, not a
    /// confirmation, `design@rootpoint-model@style-removal`.
    pub(crate) fn removal_dialog(&mut self, ctx: &egui::Context) {
        let Some(removal) = self.styles_tab.removal.clone() else {
            return;
        };
        let others = self.replacements(&removal.name);
        let users = self.style_users(&removal.name);
        let mut choice = None;
        let mut confirm = false;
        let mut cancel = false;
        egui::Window::new("Remove a style")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(format!(
                    "Remove \"{}\", used by {users} {}?",
                    removal.name,
                    if users == 1 { "area" } else { "areas" }
                ));
                if ui.radio(removal.into.is_none(), "a custom copy").clicked() {
                    choice = Some(None);
                }
                for name in &others {
                    if ui
                        .radio(removal.into.as_ref() == Some(name), name.as_str())
                        .clicked()
                    {
                        choice = Some(Some(name.clone()));
                    }
                }
                ui.horizontal(|ui| {
                    let remove = ui.button("Remove style");
                    self.view
                        .buttons
                        .push(("Remove style".to_owned(), remove.rect));
                    confirm = remove.clicked();
                    let back = ui.button("Cancel");
                    self.view.buttons.push(("Cancel".to_owned(), back.rect));
                    cancel = back.clicked();
                });
            });
        if let Some(into) = choice {
            self.choose_replacement(into);
        }
        if confirm {
            self.confirm_removal();
        } else if cancel {
            self.cancel_removal();
        }
    }

    /// The values of a style, a text's or a named style's: each control returns the one option it
    /// changes, which its caller sends.
    fn style(&mut self, ui: &mut Ui, style: &rootpoint_model::TextStyle) -> Option<StyleOption> {
        let mut chosen = None;
        ui.horizontal_wrapped(|ui| {
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
            let response = ui.checkbox(&mut bold, "bold");
            self.view.buttons.push(("bold".to_owned(), response.rect));
            if response.changed() {
                chosen = Some(StyleOption::Bold(bold));
            }
            let mut italic = style.italic;
            if ui.checkbox(&mut italic, "italic").changed() {
                chosen = Some(StyleOption::Italic(italic));
            }
        });
        ui.horizontal_wrapped(|ui| {
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
        ui.horizontal_wrapped(|ui| {
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
        chosen
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
        ui.horizontal_wrapped(|ui| {
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
    pub(crate) fn image_assets(&self) -> Vec<AssetId> {
        let fonts: Vec<AssetId> = self.font_assets().into_iter().map(|f| f.0).collect();
        self.session
            .document()
            .assets
            .keys()
            .filter(|id| !fonts.contains(id))
            .copied()
            .collect()
    }

    /// The assets whose file is a TrueType or OpenType font, by its name. Only
    /// these are offered as fonts: any other asset chosen as a font would draw in the built-in sans.
    fn font_assets(&self) -> Vec<(AssetId, String)> {
        self.session
            .document()
            .assets
            .iter()
            .filter(|(_, a)| {
                let p = a.path.to_ascii_lowercase();
                p.ends_with(".ttf") || p.ends_with(".otf")
            })
            .map(|(id, _)| (*id, self.asset_name(*id)))
            .collect()
    }

    /// An asset's file name, the last component of its path.
    pub(crate) fn asset_name(&self, id: AssetId) -> String {
        match self.session.document().assets.get(&id) {
            Some(a) => a.path.rsplit('/').next().unwrap_or(&a.path).to_owned(),
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
