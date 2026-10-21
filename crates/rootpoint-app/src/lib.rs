//! The editor of rootpoint: a window that shows the slides of a presentation, where every action
//! of the editor is a protocol command run on one session.

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

mod actions;
mod assets;
pub mod hit;
mod panels;
pub mod selection;
mod view;

use eframe::egui::{
    self, CentralPanel, ColorImage, Context, Key, KeyboardShortcut, Modifiers, Panel, Rect,
    TextureHandle, TextureOptions, Ui,
};
use rootpoint_model::{NodeId, SlideId};
use rootpoint_protocol::{Command, Files, Reply, Session};
use rootpoint_render::{Precision, Report, render_slide};

use assets::Assets;
use hit::{Edge, Layout, Press};
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
    /// The file the presentation was last loaded from or saved to, which Save writes.
    path: Option<String>,
    /// The text field's area and its text, while the field has focus; its group is open then.
    editing: Option<(NodeId, String)>,
    /// Esc hides the preview line until Ctrl and Shift are both released.
    preview_hidden: bool,
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
    /// A drag of the grip of an area, inside an undo group, which sends its commands at the drop.
    Grip {
        area: NodeId,
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
    /// Where the last frame placed the text field, if it showed one.
    text_field: Option<Rect>,
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
            path: None,
            editing: None,
            preview_hidden: false,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Where the last frame placed the shown slide in the slide view, in points.
    pub fn layout(&self) -> Option<&Layout> {
        self.view.layout.as_ref()
    }

    /// Where the last frame placed each row of the slide list, in points.
    pub fn slide_rows(&self) -> &[(SlideId, Rect)] {
        &self.view.rows
    }

    /// Where the last frame placed the text field of the side panel, if it showed one.
    pub fn text_field_rect(&self) -> Option<Rect> {
        self.view.text_field
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
        // A release never delivered, as when the window loses focus mid-drag, leaves no group
        // open.
        if !ui.input(|i| i.focused) {
            self.release_drag();
            self.blur_text();
        }
        if !ui.ctx().egui_wants_keyboard_input() {
            self.shortcuts(ui);
        }
        self.view.text_field = None;
        Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        Panel::bottom("status").show(ui, |ui| {
            for line in self.status() {
                ui.label(line);
            }
        });
        Panel::left("slides").show(ui, |ui| self.slide_list(ui));
        Panel::right("selection")
            .default_size(260.0)
            .show(ui, |ui| self.side_panel(ui));
        CentralPanel::no_frame().show(ui, |ui| self.slide_view(ui));
    }

    /// The keys of the editor, while no text field has focus. `consume_shortcut` ignores extra
    /// Shift, so Ctrl+Shift+Z is tested before Ctrl+Z; `consume_key` ignores extra Shift and Alt,
    /// so Alt and the arrows are tested before Shift and the arrows.
    fn shortcuts(&mut self, ui: &mut Ui) {
        let shortcut = |ui: &mut Ui, modifiers, key| {
            ui.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(modifiers, key)))
        };
        if shortcut(ui, Modifiers::COMMAND | Modifiers::SHIFT, Key::Z)
            || shortcut(ui, Modifiers::COMMAND, Key::Y)
        {
            self.redo();
        }
        if shortcut(ui, Modifiers::COMMAND, Key::Z) {
            self.undo();
        }
        if shortcut(ui, Modifiers::COMMAND, Key::O) {
            self.open_dialog();
        }
        if shortcut(ui, Modifiers::COMMAND, Key::S) {
            self.save();
        }
        for (key, m, edge) in [
            (Key::ArrowUp, Move::Parent, Edge::Up),
            (Key::ArrowDown, Move::Child, Edge::Down),
            (Key::ArrowLeft, Move::Previous, Edge::Left),
            (Key::ArrowRight, Move::Next, Edge::Right),
        ] {
            if ui.input_mut(|i| i.consume_key(Modifiers::ALT, key)) {
                self.move_selection(m);
            } else if ui.input_mut(|i| i.consume_key(Modifiers::SHIFT, key)) {
                self.split_middle(edge);
            }
        }
        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Delete)) {
            self.remove_selection();
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
