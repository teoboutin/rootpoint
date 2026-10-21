//! The editor of rootpoint: a window that shows the slides of a presentation, where every action
//! of the editor is a protocol command run on one session.

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

mod assets;
pub mod hit;
pub mod selection;

use eframe::egui::{
    self, Align2, CentralPanel, Color32, ColorImage, Context, CursorIcon, FontId, Key, Modifiers,
    Panel, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions, Ui,
    Vec2,
};
use rootpoint_model::{Axis, Content, Node, NodeId, SlideId};
use rootpoint_protocol::{Command, Files, Reply, Session};
use rootpoint_render::{Precision, Report, render_slide};

use assets::Assets;
use hit::{Layout, Press};
use selection::{Memory, Move};

/// The largest side of the slide's texture, in pixels: egui-wgpu 0.37.0 asks the GPU for textures
/// of at most 8192 pixels per side, below the render crate's `MAX_SIDE`.
const MAX_TEXTURE_SIDE: usize = 8192;

/// The editor's state: the session, the files its commands read and write, and what the window
/// shows of it.
pub struct App {
    session: Session,
    files: Box<dyn Files>,
    assets: Assets,
    view: SlideView,
    /// The reason of the last command that failed, shown until the next command.
    error: Option<String>,
    memory: Memory,
    gesture: Gesture,
    /// What the press of the primary button in the slide view took, until its release.
    press: Option<Press>,
}

/// A drag in progress in the slide view.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Gesture {
    Idle,
    /// A drag of a split's divider, inside an undo group, and the position it last sent.
    Divider {
        split: NodeId,
        index: usize,
        pos: f64,
    },
    /// A drag cancelled by Esc: nothing more is sent until the button is released.
    Cancelled,
}

/// The slide view: a texture of `render_slide`'s image, and what it was drawn from.
#[derive(Default)]
struct SlideView {
    texture: Option<TextureHandle>,
    /// The slide the texture holds, and its size in pixels; `None` once a command has run, since
    /// it may have changed the document or the slide shown.
    drawn: Option<(SlideId, [usize; 2])>,
    report: Report,
    /// Where the last frame placed the shown slide.
    layout: Option<Layout>,
    /// Where the last frame placed each row of the slide list, in points.
    rows: Vec<(SlideId, Rect)>,
}

impl App {
    /// An editor on an empty session, whose commands read and write `files`.
    pub fn new(files: Box<dyn Files>) -> Self {
        App {
            session: Session::new(),
            files,
            assets: Assets::default(),
            view: SlideView::default(),
            error: None,
            memory: Memory::default(),
            gesture: Gesture::Idle,
            press: None,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Run a command on the session. Every action of the editor goes through here, so the
    /// document and the view after an action are those its commands give.
    pub fn run(&mut self, command: Command) -> Reply {
        let reply = self.session.run(command, &mut *self.files);
        self.error = match &reply {
            Reply::Ok(_) => None,
            Reply::Err(reason) => Some(reason.clone()),
        };
        self.view.drawn = None;
        if let (Some(node), Some(slide)) = (self.session.view().selected, self.shown_slide())
            && let Some(slide) = self.session.document().slide(slide)
        {
            self.memory.note(slide, node);
        }
        reply
    }

    /// Select a node: a click on an area, or on a breadcrumb entry.
    pub fn select_node(&mut self, node: NodeId) {
        self.run(Command::Select { node });
    }

    /// Show a slide and select no node: a click on a slide in the list, or on the breadcrumb's
    /// first entry.
    pub fn show_slide(&mut self, slide: SlideId) {
        self.run(Command::SelectSlide { slide });
    }

    /// Alt and an arrow: select the parent, a child, or a sibling of the selection. A move with
    /// nowhere to go does nothing.
    pub fn move_selection(&mut self, m: Move) {
        let (Some(node), Some(slide)) = (self.session.view().selected, self.shown_slide()) else {
            return;
        };
        let target = self
            .session
            .document()
            .slide(slide)
            .and_then(|s| self.memory.target(s, node, m));
        if let Some(node) = target {
            self.select_node(node);
        }
    }

    /// Add a slide after the shown one, or at the end when none is shown.
    pub fn add_slide(&mut self) {
        let doc = self.session.document();
        let at = self
            .shown_slide()
            .and_then(|s| doc.slides.iter().position(|x| x.id == s))
            .map_or(doc.slides.len(), |i| i + 1);
        self.run(Command::AddSlide { at: Some(at) });
    }

    pub fn remove_slide(&mut self) {
        if let Some(slide) = self.shown_slide() {
            self.run(Command::RemoveSlide { slide });
        }
    }

    pub fn duplicate_slide(&mut self) {
        if let Some(slide) = self.shown_slide() {
            self.run(Command::DuplicateSlide { slide });
        }
    }

    /// A drop of a slide on the row at `to` in the slide list.
    pub fn move_slide(&mut self, slide: SlideId, to: usize) {
        self.run(Command::MoveSlide { slide, to });
    }

    /// The press that grabs a divider opens an undo group, so the whole drag is one undo step.
    pub fn grab_divider(&mut self, split: NodeId, index: usize) {
        let Some(pos) = self.divider(split, index) else {
            return;
        };
        if let Reply::Ok(_) = self.run(Command::Begin) {
            self.gesture = Gesture::Divider { split, index, pos };
        }
    }

    /// A frame of a divider drag that moves it to `pos`.
    pub fn drag_divider(&mut self, pos: f64) {
        if let Gesture::Divider {
            split,
            index,
            pos: last,
        } = self.gesture
            && pos != last
        {
            self.run(Command::Divider { split, index, pos });
            self.gesture = Gesture::Divider { split, index, pos };
        }
    }

    /// The release that ends a divider drag closes its group.
    pub fn release_divider(&mut self) {
        match self.gesture {
            Gesture::Divider { .. } => {
                self.run(Command::End);
            }
            Gesture::Idle | Gesture::Cancelled => {}
        }
        self.gesture = Gesture::Idle;
    }

    /// Esc during a drag: its edits are reverted, and the group closes with no step.
    pub fn cancel_drag(&mut self) {
        if let Gesture::Divider { .. } = self.gesture {
            self.run(Command::Cancel);
            self.gesture = Gesture::Cancelled;
        }
    }

    /// A double-click on a divider sets it halfway between its neighbours, the neighbouring
    /// dividers or the split's edges.
    pub fn reset_divider(&mut self, split: NodeId, index: usize) {
        let Some(Node::Split(s)) = self.node(split) else {
            return;
        };
        let pos = (s.child_extent(index).0 + s.child_extent(index + 1).1) / 2.0;
        if s.dividers.get(index) != Some(&pos) {
            self.run(Command::Divider { split, index, pos });
        }
    }

    /// A node of the shown slide.
    fn node(&self, node: NodeId) -> Option<&Node> {
        let slide = self.session.document().slide(self.shown_slide()?)?;
        slide.root.find(node)
    }

    /// The position of a divider of the shown slide.
    fn divider(&self, split: NodeId, index: usize) -> Option<f64> {
        match self.node(split)? {
            Node::Split(s) => s.dividers.get(index).copied(),
            Node::Area(_) => None,
        }
    }

    /// Where the last frame placed the shown slide in the slide view, in points.
    pub fn layout(&self) -> Option<&Layout> {
        self.view.layout.as_ref()
    }

    /// Where the last frame placed each row of the slide list, in points.
    pub fn slide_rows(&self) -> &[(SlideId, Rect)] {
        &self.view.rows
    }

    /// The slide the editor shows: the view's shown slide, or the first slide when none is.
    pub fn shown_slide(&self) -> Option<SlideId> {
        let doc = self.session.document();
        self.session
            .view()
            .shown
            .or(doc.slides.first().map(|s| s.id))
    }

    /// The lines under the slide view: the last command's error, and each list of the last
    /// render's report that names something not drawn as the document asks.
    pub fn status(&self) -> Vec<String> {
        let mut lines: Vec<String> = self.error.iter().cloned().collect();
        let report = &self.view.report;
        let mut list = |what: &str, mut ids: Vec<String>| {
            // An asset used by two areas is named once.
            let mut seen = std::collections::BTreeSet::new();
            ids.retain(|id| seen.insert(id.clone()));
            if !ids.is_empty() {
                lines.push(format!("{what}: {}", ids.join(", ")));
            }
        };
        list(
            "missing images",
            report
                .missing_images
                .iter()
                .map(|i| i.to_string())
                .collect(),
        );
        list(
            "font not found, drawn in the built-in sans",
            report
                .font_fallbacks
                .iter()
                .map(|n| n.to_string())
                .collect(),
        );
        list(
            "characters no font has",
            report
                .missing_glyphs
                .iter()
                .map(|n| n.to_string())
                .collect(),
        );
        list(
            "text not drawn",
            report.undrawn_text.iter().map(|n| n.to_string()).collect(),
        );
        lines
    }

    /// Show the editor for one frame. It takes a `Ui`, so a test drives it with
    /// `Context::run_ui`, with no window.
    pub fn frame(&mut self, ui: &mut Ui) {
        for (key, m) in [
            (Key::ArrowUp, Move::Parent),
            (Key::ArrowDown, Move::Child),
            (Key::ArrowLeft, Move::Previous),
            (Key::ArrowRight, Move::Next),
        ] {
            if ui.input_mut(|i| i.consume_key(Modifiers::ALT, key)) {
                self.move_selection(m);
            }
        }
        Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open…").clicked() {
                    self.open();
                }
            });
        });
        Panel::bottom("status").show(ui, |ui| {
            for line in self.status() {
                ui.label(line);
            }
        });
        Panel::left("slides").show(ui, |ui| self.slide_list(ui));
        Panel::right("selection").show(ui, |ui| self.breadcrumb(ui));
        CentralPanel::no_frame().show(ui, |ui| self.slide_view(ui));
    }

    /// The slide list: a row per slide, which a click shows and a drag moves, and the buttons
    /// that add a slide after the shown one, and remove or duplicate the shown one.
    fn slide_list(&mut self, ui: &mut Ui) {
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
            if response
                .dnd_hover_payload::<SlideId>()
                .is_some_and(|p| *p != id)
            {
                let y = rect.center().y;
                ui.painter().hline(
                    rect.x_range(),
                    y,
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

    /// Ask for a presentation file, and load it.
    fn open(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("rootpoint presentation", &["ron"])
            .pick_file()
        else {
            return;
        };
        match path.to_str() {
            Some(path) => {
                self.run(Command::Load {
                    path: path.to_owned(),
                });
            }
            None => self.error = Some(format!("{} is not UTF-8", path.display())),
        }
    }

    /// The shown slide, scaled uniformly to the largest size that fits the view, letterboxed:
    /// `render_slide` draws it so, at the view's size in physical pixels. The pointer's presses
    /// in it are taken first, with the layout the frame starts from, then the slide is drawn as
    /// they left it, and the editor's marks over it.
    fn slide_view(&mut self, ui: &mut Ui) {
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
        if let Some(layout) = &self.view.layout {
            self.overlay(ui, layout);
        }
    }

    /// The press, the drag and the release of the primary button in the slide view, and Esc.
    fn pointer(&mut self, ui: &Ui, response: &egui::Response, layout: &Layout) {
        let (pressed, origin, down, released, pos, double, esc) = ui.input(|i| {
            (
                i.pointer.primary_pressed(),
                i.pointer.press_origin(),
                i.pointer.primary_down(),
                i.pointer.primary_released(),
                i.pointer.interact_pos(),
                i.pointer.button_double_clicked(PointerButton::Primary),
                i.key_pressed(Key::Escape),
            )
        });
        // egui reports a drag only once the pointer has moved, so the press is read from its
        // origin, not from the frame `drag_started` is set.
        if pressed
            && let Some(origin) = origin
            && response.rect.contains(origin)
            && response.contains_pointer()
        {
            let press = layout.press(origin);
            self.press = Some(press);
            if let Press::Divider(i) = press {
                let d = &layout.dividers[i];
                self.grab_divider(d.split, d.index);
            }
        }
        if let Gesture::Divider { split, index, .. } = self.gesture {
            if esc {
                self.cancel_drag();
            } else if down
                && let Some(pos) = pos
                && let Some(i) = layout
                    .dividers
                    .iter()
                    .position(|d| d.split == split && d.index == index)
                && let Some(at) = layout.drag_position(i, pos)
            {
                self.drag_divider(at);
            }
        }
        if released {
            let press = self.press.take();
            self.release_divider();
            match press {
                Some(Press::Area(area)) if response.clicked() => self.select_node(area),
                Some(Press::Divider(i)) if double => {
                    let d = &layout.dividers[i];
                    self.reset_divider(d.split, d.index);
                }
                _ => {}
            }
        }
    }

    /// The editor's marks, drawn with egui over the slide's texture and never into its image: a
    /// line on each divider, a highlight on the divider under the pointer or being dragged, and
    /// an outline on the selected node.
    fn overlay(&self, ui: &Ui, layout: &Layout) {
        let painter = ui.painter();
        let accent = ui.visuals().selection.stroke.color;
        let active = match self.gesture {
            Gesture::Divider { split, index, .. } => layout
                .dividers
                .iter()
                .position(|d| d.split == split && d.index == index),
            Gesture::Idle => ui
                .input(|i| i.pointer.hover_pos())
                .and_then(|p| layout.divider_at(p)),
            Gesture::Cancelled => None,
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
        if let Some(node) = self.session.view().selected
            && let Some(r) = layout.rect(node)
        {
            painter.rect_stroke(r, 0.0, Stroke::new(2.0, accent), StrokeKind::Inside);
        }
    }

    /// Render the slide at `size` and upload it to the view's texture, with no conversion: the
    /// render is premultiplied RGBA8 in sRGB, as `ColorImage::from_rgba_premultiplied` takes it.
    fn draw(&mut self, ctx: &Context, slide: SlideId, size: [usize; 2]) {
        self.assets.refresh(&self.session, &mut *self.files);
        // `texture_size` keeps each side within 8192, so it fits in a u16.
        let (w, h) = (size[0] as u16, size[1] as u16);
        let (fonts, images) = self.assets.sets();
        match render_slide(
            self.session.document(),
            slide,
            w,
            h,
            fonts,
            images,
            Precision::Native,
        ) {
            Ok(out) => {
                let image =
                    ColorImage::from_rgba_premultiplied(size, out.pixmap.data_as_u8_slice());
                match &mut self.view.texture {
                    Some(texture) => texture.set(image, TextureOptions::LINEAR),
                    None => {
                        self.view.texture =
                            Some(ctx.load_texture("slide", image, TextureOptions::LINEAR));
                    }
                }
                self.view.report = out.report;
            }
            Err(e) => self.error = Some(format!("the slide could not be drawn: {e:?}")),
        }
        self.view.drawn = Some((slide, size));
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frame(ui);
    }
}

/// The size in physical pixels of a texture that covers `rect`: its size in points times the
/// pixels per point, both sides scaled down by one factor where a side would exceed the largest
/// texture, so the slide keeps its proportions.
fn texture_size(ctx: &Context, rect: Rect) -> [usize; 2] {
    let ppp = ctx.pixels_per_point();
    let limit = MAX_TEXTURE_SIDE.min(ctx.input(|i| i.max_texture_side)) as f32;
    let (w, h) = (rect.width() * ppp, rect.height() * ppp);
    let shrink = (limit / w).min(limit / h).min(1.0);
    [
        (w * shrink).round().max(0.0) as usize,
        (h * shrink).round().max(0.0) as usize,
    ]
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
