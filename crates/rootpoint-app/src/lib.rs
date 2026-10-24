//! The editor of rootpoint: a window that shows the slides of a presentation, where every action
//! of the editor is a protocol command run on one session.

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

mod actions;
mod assets;
#[cfg(not(target_arch = "wasm32"))]
pub mod autosave;
#[cfg(unix)]
pub mod control;
pub mod editing;
pub mod hit;
mod memory;
mod menu;
mod panels;
pub mod selection;
mod view;
#[cfg(not(target_arch = "wasm32"))]
mod watch;
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
    /// When a click on the slide last left the final view, in egui's seconds: the next press, if
    /// it comes within egui's delay of a double-click, is that double-click's second half.
    left_final_view: Option<f64>,
    /// The control socket, when the window listens on one.
    #[cfg(unix)]
    listener: Option<control::Listener>,
    /// Who holds the authority to edit: the window, or the control socket.
    controller: Controller,
    /// The files, when the app holds them in memory, as the browser build does: the same files
    /// as `files`, which the actions that open a picked directory or add a picked image fill.
    held: Option<MemoryFiles>,
    /// Whether the app is the native window, which takes a workspace's lock and moves its control
    /// socket with the store, `App::native`.
    #[cfg(not(target_arch = "wasm32"))]
    native: bool,
    /// The lock of the workspace the native window holds, `rootpoint_protocol::workspace::run`.
    #[cfg(not(target_arch = "wasm32"))]
    lock: Option<rootpoint_protocol::workspace::WorkspaceLock>,
    /// The autosave of the workspace the native window holds.
    #[cfg(not(target_arch = "wasm32"))]
    autosave: Option<autosave::Autosave>,
    /// The watcher of the directories of that workspace's assets.
    #[cfg(not(target_arch = "wasm32"))]
    watch: Option<watch::Watch>,
    /// The window's context, kept at its first frame, through which the autosave's thread and the
    /// watcher ask for a frame; they may start before it, at an open of the command line.
    #[cfg(not(target_arch = "wasm32"))]
    repaint: std::sync::Arc<std::sync::OnceLock<Context>>,
    /// Why "Load the file" failed, which the banner names until the author answers again.
    #[cfg(not(target_arch = "wasm32"))]
    reload_error: Option<String>,
    /// The close of the window was asked while a presentation held in memory had unsaved edits:
    /// the question shows until the author answers.
    close_question: bool,
    /// The author answered the question by closing: the next close is not cancelled.
    closing: bool,
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
    /// Where the last frame placed each button and tab it records, by label: the toolbar's
    /// "Preview", the side panel's tabs, the Styles tab's controls and the removal dialog's buttons.
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
            preview_hidden: false,
            fullscreen: false,
            edit: None,
            edit_on_click: None,
            text_drag: false,
            left_final_view: None,
            #[cfg(unix)]
            listener: None,
            controller: Controller::Window,
            held: None,
            #[cfg(not(target_arch = "wasm32"))]
            native: false,
            #[cfg(not(target_arch = "wasm32"))]
            lock: None,
            #[cfg(not(target_arch = "wasm32"))]
            autosave: None,
            #[cfg(not(target_arch = "wasm32"))]
            watch: None,
            #[cfg(not(target_arch = "wasm32"))]
            repaint: std::sync::Arc::new(std::sync::OnceLock::new()),
            #[cfg(not(target_arch = "wasm32"))]
            reload_error: None,
            close_question: false,
            closing: false,
            #[cfg(target_arch = "wasm32")]
            web: web::Web::default(),
            tab: SideTab::default(),
            styles_tab: StylesTab::default(),
            menu: None,
            menu_serial: 0,
        }
    }

    /// The native window on `files`, the native file system: its commands take the lock of a
    /// workspace they open and drop the old one, and the control socket it listens on moves with
    /// the store, `design@rootpoint-app@socket-place`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn native(files: Box<dyn Files>) -> Self {
        let mut app = App::new(files);
        app.native = true;
        app
    }

    /// An editor on an empty session whose files are held in memory, as the browser build's are:
    /// `App::open_directory` and `App::add_picked_image` put the picked files in, which `open-copy`
    /// and `import` copy into the session's map, and a save writes none.
    pub fn in_memory() -> Self {
        let files = MemoryFiles::default();
        let mut app = App::new(Box::new(files.clone()));
        app.held = Some(files);
        app
    }

    /// Whether the editor shows the final result, the session's preview: the slide with no mark,
    /// and no gesture on it. "Preview" names the line of a split at the pointer in the window's
    /// code, as `hit::Preview`, so the code names this the final view.
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
        // A command that replaces the store replaces the files the assets are read from, and two
        // maps can hold other bytes at one path; an import can write the file of an asset drawn
        // missing until then. The decoded assets are keyed on the paths, not on the bytes, so they
        // are dropped after either.
        let rereads = matches!(
            command,
            Command::New
                | Command::Sample
                | Command::Open { .. }
                | Command::OpenCopy { .. }
                | Command::OpenPack { .. }
                | Command::Import { .. }
        );
        let root = self.session.root().map(str::to_owned);
        let unsaved = self.session.unsaved();
        #[cfg(not(target_arch = "wasm32"))]
        let reply = if self.native {
            rootpoint_protocol::workspace::run(
                &mut self.session,
                command,
                &mut *self.files,
                &mut self.lock,
            )
        } else {
            self.session.run(command, &mut *self.files)
        };
        #[cfg(target_arch = "wasm32")]
        let reply = self.session.run(command, &mut *self.files);
        #[cfg(unix)]
        if self.native && self.session.root() != root.as_deref() {
            self.move_socket(root.as_deref());
        }
        #[cfg(not(target_arch = "wasm32"))]
        if self.native {
            self.follow_workspace(root.as_deref(), unsaved);
        }
        #[cfg(target_arch = "wasm32")]
        let _ = unsaved;
        #[cfg(not(unix))]
        let _ = root;
        if rereads && matches!(reply, Reply::Ok(_)) {
            self.assets.invalidate();
        }
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

    /// While the guard has stopped the autosave, a banner over the toolbar says that
    /// `presentation.ron` was changed outside rootpoint, with "Load the file" and "Keep mine".
    #[cfg(not(target_arch = "wasm32"))]
    fn outside_change_banner(&mut self, ui: &mut Ui) {
        if !self.autosave.as_ref().is_some_and(|a| a.stopped()) {
            return;
        }
        Panel::top("outside-change").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("presentation.ron was changed outside rootpoint");
                let load = ui.button("Load the file");
                self.view
                    .buttons
                    .push(("Load the file".to_owned(), load.rect));
                if load.clicked() {
                    self.load_outside_file();
                }
                let keep = ui.button("Keep mine");
                self.view.buttons.push(("Keep mine".to_owned(), keep.rect));
                if keep.clicked() {
                    self.keep_mine();
                }
            });
            if let Some(reason) = &self.reload_error {
                ui.label(format!("The file does not load: {reason}"));
            }
        });
    }

    /// What the autosave's thread and the watcher call to ask the window for a frame, once its
    /// first frame has given the context.
    #[cfg(not(target_arch = "wasm32"))]
    fn waker(&self) -> std::sync::Arc<dyn Fn() + Send + Sync> {
        let repaint = self.repaint.clone();
        std::sync::Arc::new(move || {
            if let Some(ctx) = repaint.get() {
                ctx.request_repaint();
            }
        })
    }

    /// The autosave of the workspace the native window holds, if any, for the tests.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn autosave(&self) -> Option<&autosave::Autosave> {
        self.autosave.as_ref()
    }

    /// How many times the assets were decoded, for the tests.
    pub fn asset_decodings(&self) -> u64 {
        self.assets.decodings()
    }

    /// After a command of the native window: a store that became a workspace starts its autosave
    /// and its watcher, one that left it stops them; a command that changed the undo steps sends
    /// the autosave a clone of the document; and the watcher follows the directories of the
    /// assets.
    #[cfg(not(target_arch = "wasm32"))]
    fn follow_workspace(&mut self, old_root: Option<&str>, old_unsaved: u64) {
        let root = self.session.root().map(std::path::PathBuf::from);
        if self.session.root() != old_root {
            self.autosave = root
                .clone()
                .map(|root| autosave::Autosave::start(root, self.waker()));
            self.watch = root
                .as_ref()
                .and_then(|_| watch::Watch::new(self.waker()).ok());
            self.reload_error = None;
        } else if self.session.unsaved() != old_unsaved
            && let Some(autosave) = &self.autosave
        {
            autosave.save(self.session.document().clone());
        }
        if let (Some(watch), Some(root)) = (&mut self.watch, &root) {
            let dirs = self
                .session
                .document()
                .assets
                .values()
                .filter(|a| rootpoint_model::asset_path_is_confined(&a.path))
                .filter_map(|a| root.join(&a.path).parent().map(|d| d.to_path_buf()))
                .collect();
            watch.set(dirs);
        }
    }

    /// At each frame of the native window: an event of the watcher that names the file of an asset
    /// of the table drops the decoded assets, which the frame then decodes again; an event of any
    /// other file decodes nothing.
    #[cfg(not(target_arch = "wasm32"))]
    fn take_watched(&mut self) {
        let (Some(watch), Some(root)) = (&self.watch, self.session.root()) else {
            return;
        };
        let root = std::path::Path::new(root);
        let assets: std::collections::BTreeSet<std::path::PathBuf> = self
            .session
            .document()
            .assets
            .values()
            .map(|a| root.join(&a.path))
            .collect();
        if watch.take().iter().any(|path| assets.contains(path)) {
            self.assets.invalidate();
            // The slide is rendered again with the new bytes, as after a command.
            self.view.drawn = None;
        }
    }

    /// The text of the state label: where the presentation is, and whether its edits are saved,
    /// the store's state as `store` replies it. A workspace is saved by Ctrl+S, so it names the
    /// changes since its save.
    pub fn state_label(&self) -> String {
        let changes = |n: u64| match n {
            1 => "1 change".to_owned(),
            n => format!("{n} changes"),
        };
        let unsaved = self.session.unsaved();
        // A workspace that autosaves is saved, unless the guard stopped the writes.
        #[cfg(not(target_arch = "wasm32"))]
        if let (Some(root), Some(autosave)) = (self.session.root(), &self.autosave) {
            return if autosave.stopped() {
                format!("Workspace {root}, not saved: the file changed outside rootpoint")
            } else {
                format!("Workspace {root}, saved")
            };
        }
        match (self.session.root(), self.session.pack()) {
            (Some(root), _) if unsaved == 0 => format!("Workspace {root}, saved"),
            (Some(root), _) => format!("Workspace {root}, {} since the save", changes(unsaved)),
            (None, None) => "In memory, not saved".to_owned(),
            (None, Some(pack)) if unsaved == 0 => format!("In memory, saved to {pack}"),
            (None, Some(pack)) => format!("In memory, saved to {pack}, {} since", changes(unsaved)),
        }
    }

    /// Whether the question before closing a window held in memory with unsaved edits shows.
    pub fn close_question(&self) -> bool {
        self.close_question
    }

    /// The close of the window was asked: in memory with unsaved edits, which no undo reaches once
    /// the window is gone, the close is cancelled and the question asks first, unless the author
    /// answered it by closing; a workspace never asks.
    fn on_close(&mut self, ctx: &Context) {
        let asked = ctx.input(|i| i.viewport().close_requested());
        if !asked || self.closing {
            return;
        }
        if self.session.root().is_none() && self.session.unsaved() > 0 {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.close_question = true;
        }
    }

    /// The question before a close that would lose unsaved edits: "Close" closes, "Keep open"
    /// keeps the window.
    fn show_close_question(&mut self, ctx: &Context) {
        if !self.close_question {
            return;
        }
        let n = self.session.unsaved();
        egui::Window::new("Close without saving?")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(if n == 1 {
                    "1 change held in memory is not saved.".to_owned()
                } else {
                    format!("{n} changes held in memory are not saved.")
                });
                ui.horizontal(|ui| {
                    let close = ui.button("Close");
                    self.view.buttons.push(("Close".to_owned(), close.rect));
                    if close.clicked() {
                        self.closing = true;
                        self.close_question = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    let keep = ui.button("Keep open");
                    self.view.buttons.push(("Keep open".to_owned(), keep.rect));
                    if keep.clicked() {
                        self.close_question = false;
                    }
                });
            });
    }

    /// Show the editor for one frame. It takes a `Ui`, so a test drives it with
    /// `Context::run_ui`, with no window.
    pub fn frame(&mut self, ui: &mut Ui) {
        // The places of the buttons are this frame's, in presenter mode as in the editor.
        self.view.buttons.clear();
        self.on_close(ui.ctx());
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.repaint.get_or_init(|| ui.ctx().clone());
            self.take_watched();
        }
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
        self.show_close_question(ui.ctx());
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
        if self.final_view() && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            // Esc leaves the final view, with or without a slide shown.
            self.toggle_final_view();
        } else if self.edit.is_some() {
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
        #[cfg(not(target_arch = "wasm32"))]
        self.outside_change_banner(ui);
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
