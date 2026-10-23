//! The editor of rootpoint: a window that shows the slides of a presentation, where every action
//! of the editor is a protocol command run on one session.

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

mod actions;
mod assets;
#[cfg(unix)]
pub mod control;
pub mod editing;
pub mod hit;
mod memory;
mod menu;
mod panels;
pub mod selection;
mod view;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
use eframe::egui::ViewportCommand;
use eframe::egui::{
    self, CentralPanel, Color32, ColorImage, Context, Frame, Key, KeyboardShortcut, Modifiers,
    Panel, Pos2, Rect, TextureHandle, TextureOptions, Ui,
};
use rootpoint_model::{NodeId, SlideId};
use rootpoint_protocol::{Command, Files, Reply, Session};
use rootpoint_render::{Precision, Report, render_slide};

use assets::Assets;
use hit::{Edge, Layout, Press};
use selection::{Memory, Move};

pub use memory::MemoryFiles;
pub use menu::{MenuEntry, MenuItem, MenuSection};
#[cfg(target_arch = "wasm32")]
pub use web::keep_f5;

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
    /// From the pointer to the grabbed divider's line, in points, during a divider drag.
    grab_offset: egui::Vec2,
    /// The file the presentation was last loaded from or saved to, which Save writes.
    path: Option<String>,
    /// Esc hides the preview line until Ctrl and Shift are both released.
    preview_hidden: bool,
    /// Whether the window was last asked to be full screen, as presenter mode is.
    fullscreen: bool,
    /// The text being edited in place, if any.
    edit: Option<editing::Editing>,
    /// Where the current press was, when its release, if a click, enters editing: it was pressed on
    /// the selected text area. egui clears the press's origin by the release's frame.
    edit_on_click: Option<Pos2>,
    /// Whether the primary button, pressed inside the edited text, drags a selection.
    text_drag: bool,
    /// The control socket, when the window listens on one.
    #[cfg(unix)]
    listener: Option<control::Listener>,
    /// Who holds the authority to edit: the window, or the control socket.
    controller: Controller,
    /// The files, when the app holds them in memory, as the browser build does: the same files
    /// as `files`, which the actions that open a picked directory or add a picked image fill.
    held: Option<MemoryFiles>,
    /// The pickers and full screen of the page.
    #[cfg(target_arch = "wasm32")]
    web: web::Web,
    /// The tab the side panel shows.
    tab: SideTab,
    /// The Styles tab's own state, the window's and not the session's.
    styles_tab: StylesTab,
    /// The node menu, while it is open: the window's own state, not the session's.
    menu: Option<menu::NodeMenu>,
    /// How many times the node menu was opened.
    menu_serial: u64,
}

/// The tabs of the side panel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SideTab {
    /// The outline, the actions on the selection and its content.
    #[default]
    Selection,
    /// The table of named styles.
    Styles,
}

/// The state of the Styles tab: the style selected in its list, the name field's text while it is
/// edited, and the dialog of a removal, if open. It is the window's own state, a widget's, which no
/// command sets.
#[derive(Clone, Debug, Default, PartialEq)]
struct StylesTab {
    selected: Option<String>,
    name_edit: Option<String>,
    removal: Option<Removal>,
}

/// The dialog of the removal of a style that texts name: the style, and the replacement chosen,
/// another style, or `None` for a custom copy.
#[derive(Clone, Debug, PartialEq)]
pub struct Removal {
    pub name: String,
    pub into: Option<String>,
}

/// Who holds the authority to edit, per `design@rootpoint-app@one-controller-at-a-time`: the window
/// or its control socket, every connection of the socket together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Controller {
    /// The window, from the start or after the socket's `release`.
    Window,
    /// The socket, after its `control`.
    Socket,
    /// The window, which the author took back from the socket with the button "Take control":
    /// the socket's refused edits tell its agent to stop and wait for the author's instructions,
    /// until its next `control`.
    TakenFromSocket,
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
        /// Whether the pointer has left the press: from then on every frame sends, the press's
        /// own place included.
        left: bool,
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
    /// Where the last frame placed the button "Take control", while the socket held control.
    take_control: Option<Rect>,
    /// Where the last frame placed the side panel, in points.
    side_panel: Option<Rect>,
    /// The width of the side panel's content, and of the scroll area that shows it, in points.
    side_content: Option<(f32, f32)>,
    /// Where the last frame placed each row of the outline.
    outline: Vec<OutlineRow>,
    /// The node of the outline's row under the pointer in this frame, which the overlay outlines.
    hovered_row: Option<NodeId>,
    /// Where the last frame placed each button and tab it records, by label: the side panel's tabs,
    /// the Styles tab's controls and the removal dialog's buttons.
    buttons: Vec<(String, Rect)>,
    /// Where the last frame placed each entry of the node menu, by label.
    menu_entries: Vec<(String, Rect)>,
}

/// A row of the side panel's outline, as the last frame placed it: the node it names, `None` for
/// the slide's own row, its depth in the tree, its place in points, and whether it is the
/// selection's, and its text.
#[derive(Clone, Debug, PartialEq)]
pub struct OutlineRow {
    pub node: Option<NodeId>,
    pub depth: usize,
    pub rect: Rect,
    pub selected: bool,
    pub label: String,
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
            grab_offset: egui::Vec2::ZERO,
            path: None,
            preview_hidden: false,
            fullscreen: false,
            edit: None,
            edit_on_click: None,
            text_drag: false,
            #[cfg(unix)]
            listener: None,
            controller: Controller::Window,
            held: None,
            #[cfg(target_arch = "wasm32")]
            web: web::Web::default(),
            tab: SideTab::default(),
            styles_tab: StylesTab::default(),
            menu: None,
            menu_serial: 0,
        }
    }

    /// An editor on an empty session whose files are held in memory, as the browser build's are:
    /// `App::open_directory` and `App::add_picked_image` put files in, and a save writes none.
    pub fn in_memory() -> Self {
        let files = MemoryFiles::default();
        let mut app = App::new(Box::new(files.clone()));
        app.held = Some(files);
        app
    }

    /// Whether the editor shows the final result, the session's preview: the slide with no mark,
    /// and no gesture on it. "Preview" names the line of a split at the pointer in the window, so
    /// the window names this the final view.
    pub fn final_view(&self) -> bool {
        self.session.view().preview
    }

    /// Who holds the authority to edit.
    pub fn controller(&self) -> Controller {
        self.controller
    }

    /// Whether the socket holds the authority to edit. While it does, every action of the window
    /// that changes the document returns at once, and the view actions run, per
    /// `design@rootpoint-app@one-controller-at-a-time`, so every input that calls an action is
    /// guarded there.
    pub(crate) fn socket_holds(&self) -> bool {
        self.controller == Controller::Socket
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

    /// The tab the side panel shows.
    pub fn side_tab(&self) -> SideTab {
        self.tab
    }

    /// Show a tab of the side panel, the window's own state.
    pub fn show_tab(&mut self, tab: SideTab) {
        self.tab = tab;
    }

    /// The style selected in the Styles tab's list.
    pub fn selected_style(&self) -> Option<&str> {
        self.styles_tab.selected.as_deref()
    }

    /// Select a style in the Styles tab's list, the window's own state: it shows nothing on the
    /// slide, and needs no authority to edit.
    pub fn select_style(&mut self, name: &str) {
        if self.styles_tab.selected.as_deref() != Some(name) {
            self.styles_tab.name_edit = None;
        }
        self.styles_tab.selected = Some(name.to_owned());
    }

    /// The dialog of a style's removal, while it is open.
    pub fn removal(&self) -> Option<&Removal> {
        self.styles_tab.removal.as_ref()
    }

    /// Where the last frame placed the button or the tab labelled `label`, in points.
    pub fn button_rect(&self, label: &str) -> Option<Rect> {
        self.view
            .buttons
            .iter()
            .find(|(l, _)| l == label)
            .map(|(_, r)| *r)
    }

    /// Where the last frame placed each row of the outline, in points.
    pub fn outline_rows(&self) -> &[OutlineRow] {
        &self.view.outline
    }

    /// Where the last frame placed the side panel, in points.
    pub fn side_panel_rect(&self) -> Option<Rect> {
        self.view.side_panel
    }

    /// The width of the side panel's content in the last frame, and of the scroll area that shows
    /// it, in points: a content wider than the area scrolls sideways.
    pub fn side_panel_content_width(&self) -> Option<(f32, f32)> {
        self.view.side_content
    }

    /// Where the last frame placed the button "Take control", in points: shown while the socket
    /// holds control, in the editor's banner and in a corner of presenter mode.
    pub fn take_control_rect(&self) -> Option<Rect> {
        self.view.take_control
    }

    /// The button "Take control", placed where the frame records it.
    fn take_control_button(&mut self, ui: &mut Ui) {
        let response = ui.button("Take control");
        self.view.take_control = Some(response.rect);
        if response.clicked() {
            self.take_control();
        }
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
    /// render's report that names something not drawn as the document asks, so that a missing
    /// image or a missing glyph reaches the author, which
    /// `tripwire@rootpoint-render@glyph-missing-everywhere` watches through
    /// `Report::missing_glyphs`.
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
            "missing styles, drawn in the default style",
            report
                .missing_styles
                .iter()
                .map(|(area, name)| format!("{area} {name}"))
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
        #[cfg(unix)]
        self.serve_socket();
        #[cfg(target_arch = "wasm32")]
        self.serve_pickers(ui.ctx());
        self.view.take_control = None;
        // A loss of focus releases a drag and leaves a stay in editing and its group, per
        // `design@rootpoint-app@gesture-groups`.
        if !ui.input(|i| i.focused) {
            self.release_drag();
        }
        if self.session.view().presenting.is_some() {
            // Presenter mode shows no menu, and leaves none open behind it.
            self.close_node_menu();
            self.presenter(ui);
        } else {
            self.editor(ui);
        }
        // Presenter mode is full screen, and the editor is not.
        let presenting = self.session.view().presenting.is_some();
        if presenting != self.fullscreen {
            // The browser build asks the page, per `design@rootpoint-app@page-full-screen`.
            #[cfg(target_arch = "wasm32")]
            self.web.set_fullscreen(presenting);
            #[cfg(not(target_arch = "wasm32"))]
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Fullscreen(presenting));
            self.fullscreen = presenting;
        }
    }

    /// The browser build: open what the pickers read since the last frame; and leave presenter
    /// mode when the page left full screen, per `design@rootpoint-app@page-full-screen`.
    #[cfg(target_arch = "wasm32")]
    fn serve_pickers(&mut self, ctx: &Context) {
        self.web.set_context(ctx);
        for picked in self.web.take() {
            match picked {
                web::Picked::Directory(files) => self.open_directory(files),
                web::Picked::Image(name, bytes) => self.add_picked_image(name, bytes),
                web::Picked::Failed(reason) => self.error = Some(reason),
            }
        }
        if self.fullscreen && self.web.left_fullscreen() {
            self.stop_presenting();
        }
    }

    /// The editor: the toolbar, the status, the slide list, the side panel and the slide view.
    fn editor(&mut self, ui: &mut Ui) {
        self.view.buttons.clear();
        // Editing in place ends when its area is no longer the selection, as after a click on
        // another node, an undo that removed it, or another slide shown.
        if self.edit.is_some() && self.editing() != self.selected_area() {
            self.stop_editing();
        }
        // While a text is edited its keys go to it. During a drag only Esc acts, which the slide
        // view takes; a key whose edit joined the drag's group would make the drag's step hold
        // another edit.
        if self.edit.is_some() {
            self.editing_keys(ui);
        } else if !ui.ctx().egui_wants_keyboard_input() && self.gesture == Gesture::Idle {
            self.shortcuts(ui);
        }
        // While the socket holds control, a banner over the toolbar says so, with the button that
        // takes control back.
        if self.socket_holds() {
            Panel::top("control").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("The text protocol has control");
                    self.take_control_button(ui);
                });
            });
        }
        Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        Panel::bottom("status").show(ui, |ui| {
            for line in self.status() {
                ui.label(line);
            }
        });
        Panel::left("slides").show(ui, |ui| self.slide_list(ui));
        // The content is a scroll area the size of the panel, so the panel keeps the width its edge
        // was dragged to, whatever it shows, `design@rootpoint-app@sidebar-width`.
        let side = Panel::right("selection")
            .default_size(260.0)
            .show(ui, |ui| {
                let out = egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.side_panel(ui));
                (out.content_size.x, out.inner_rect.width())
            });
        self.view.side_panel = Some(side.response.rect);
        self.view.side_content = Some(side.inner);
        CentralPanel::no_frame().show(ui, |ui| self.slide_view(ui));
        let ctx = ui.ctx().clone();
        self.removal_dialog(&ctx);
        self.node_menu(&ctx);
    }

    /// Presenter mode: the presented slide alone, scaled uniformly to the largest size that fits
    /// the window and letterboxed, rendered as the editor's slide view renders, with no mark over
    /// it but, while the socket holds control, a frame in a corner with its label and the button
    /// "Take control". →, Space and Page Down show the next slide, ← and Page Up the previous, and
    /// Esc leaves.
    fn presenter(&mut self, ui: &mut Ui) {
        let key = |ui: &mut Ui, key| ui.input_mut(|i| i.consume_key(Modifiers::NONE, key));
        if key(ui, Key::Escape) {
            self.stop_presenting();
        } else if key(ui, Key::ArrowRight) || key(ui, Key::Space) || key(ui, Key::PageDown) {
            self.next_slide();
        } else if key(ui, Key::ArrowLeft) || key(ui, Key::PageUp) {
            self.previous_slide();
        }
        // A key may have left presenter mode, which the next frame shows, or moved to another
        // slide.
        let Some(slide) = self.session.view().presenting else {
            return;
        };
        CentralPanel::no_frame()
            .frame(Frame::NONE.fill(Color32::BLACK))
            .show(ui, |ui| {
                let rect = ui.max_rect();
                let size = texture_size(ui.ctx(), rect);
                if size[0] == 0 || size[1] == 0 || self.session.document().slide(slide).is_none() {
                    return;
                }
                if self.view.drawn != Some((slide, size)) {
                    let ctx = ui.ctx().clone();
                    self.draw(&ctx, slide, size);
                }
                if let Some(texture) = &self.view.texture {
                    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
                    ui.painter().image(texture.id(), rect, uv, Color32::WHITE);
                }
            });
        // The button over the slide's texture, never in its image, per
        // `design@rootpoint-app@take-over-at-once`.
        if self.socket_holds() {
            egui::Area::new(egui::Id::unique("take control"))
                .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-8.0, 8.0))
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("The text protocol has control");
                            self.take_control_button(ui);
                        });
                    });
                });
        }
    }

    /// The keys of the editor, while no text is edited. `consume_shortcut` ignores extra
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
        if shortcut(ui, Modifiers::NONE, Key::F5) {
            self.present();
        }
        // Alt with an arrow moves as the outline reads, `design@rootpoint-app@selection-keys`: ↑ and ↓
        // to the sibling above and below, ← to the parent, → to a child. Shift with an arrow splits.
        for (key, m, edge) in [
            (Key::ArrowUp, Move::Previous, Edge::Up),
            (Key::ArrowDown, Move::Next, Edge::Down),
            (Key::ArrowLeft, Move::Parent, Edge::Left),
            (Key::ArrowRight, Move::Child, Edge::Right),
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

    /// The input while a text is edited in place, taken in the order it came: typed text, the IME,
    /// the clipboard, and the keys that move the caret, extend the selection with Shift, delete,
    /// undo and redo, or leave with Esc. The editor's other shortcuts do not act, so Delete and
    /// Shift with an arrow reach the text.
    fn editing_keys(&mut self, ui: &mut Ui) {
        use editing::CaretMove;
        let events = ui.input(|i| i.events.clone());
        for event in events {
            if self.edit.is_none() {
                return;
            }
            match event {
                egui::Event::Text(t) => self.type_str(&t),
                egui::Event::Ime(egui::ImeEvent::Preedit { text, .. }) => self.set_preedit(&text),
                egui::Event::Ime(egui::ImeEvent::Commit(t)) => self.commit_ime(&t),
                egui::Event::Copy => {
                    if let Some(t) = self.selected_text().filter(|t| !t.is_empty()) {
                        ui.ctx().copy_text(t);
                    }
                }
                egui::Event::Cut => {
                    if let Some(t) = self.cut() {
                        ui.ctx().copy_text(t);
                    }
                }
                egui::Event::Paste(t) => self.type_str(&t),
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    let shift = modifiers.shift;
                    let word = modifiers.command;
                    let moved = match key {
                        Key::ArrowLeft if word => Some(CaretMove::WordLeft),
                        Key::ArrowRight if word => Some(CaretMove::WordRight),
                        Key::ArrowLeft => Some(CaretMove::Left),
                        Key::ArrowRight => Some(CaretMove::Right),
                        Key::ArrowUp => Some(CaretMove::Up),
                        Key::ArrowDown => Some(CaretMove::Down),
                        Key::Home if word => Some(CaretMove::TextStart),
                        Key::End if word => Some(CaretMove::TextEnd),
                        Key::Home => Some(CaretMove::LineStart),
                        Key::End => Some(CaretMove::LineEnd),
                        _ => None,
                    };
                    match (moved, key) {
                        (Some(m), _) => self.move_caret(m, shift),
                        (None, Key::Escape) => self.stop_editing(),
                        (None, Key::Backspace) => self.delete_backward(),
                        (None, Key::Delete) => self.delete_forward(),
                        (None, Key::Enter) => self.type_str("\n"),
                        (None, Key::A) if word => self.select_all(),
                        (None, Key::Z) if word && shift => self.redo(),
                        (None, Key::Z) if word => self.undo(),
                        (None, Key::Y) if word => self.redo(),
                        _ => {}
                    }
                }
                _ => {}
            }
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
