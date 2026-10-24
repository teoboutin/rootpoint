//! The window's control socket, per `design@rootpoint-app@control-socket`: lines sent by
//! `rootpoint_protocol::control::send` reach the app's frames and get their replies, the listener
//! wakes the window once for each line, and a socket left by an earlier run is replaced while one
//! another window listens on is refused. A socket command acts as the window's action does, the
//! protocol's scripts give through the socket the replies a session gives them, and control passes
//! between the window and the socket by `control`, `release` and the take-over.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui::{Context, Pos2, RawInput, Rect, Vec2};
use rootpoint_app::control::Listener;
use rootpoint_app::{App, Controller};
use rootpoint_model::NodeId;
use rootpoint_protocol::control::send;
use rootpoint_protocol::{Command, Files, Reply, Session};

struct NoFiles;

impl Files for NoFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        Err(format!("{path}: no file"))
    }

    fn write(&mut self, path: &str, _: &[u8]) -> Result<(), String> {
        Err(format!("{path}: no file"))
    }

    fn exists(&mut self, _: &str) -> Result<bool, String> {
        Ok(false)
    }
}

/// A directory for a socket: under the build's scratch directory, or under `/tmp` when that path
/// would make a socket path longer than a Unix socket's address holds, 108 bytes.
fn socket_dir(name: &str) -> PathBuf {
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    if base.as_os_str().len() + name.len() + 10 < 100 {
        base.join(name)
    } else {
        PathBuf::from("/tmp").join(format!("rootpoint-{}-{name}", std::process::id()))
    }
}

/// A socket path in a scratch directory of its own.
fn socket(name: &str) -> PathBuf {
    let dir = socket_dir(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("rp.sock")
}

/// Run `client` on a thread of its own while the app is shown frame after frame on this one, as the
/// GUI's thread does, until the client ends; a client still running after 30 s fails the test,
/// a bound that holds a debug build's renders of the slide after each command.
fn with_frames<T: Send + 'static>(app: &mut App, client: impl FnOnce() -> T + Send + 'static) -> T {
    let ctx = Context::default();
    let handle = thread::spawn(client);
    let start = Instant::now();
    while !handle.is_finished() {
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "the client got no reply"
        );
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 700.0))),
            ..Default::default()
        };
        ctx.run_ui(input, |ui| app.frame(ui)).textures_delta.clear();
        thread::sleep(Duration::from_millis(1));
    }
    handle.join().unwrap()
}

#[test]
fn each_line_reaches_a_frame_and_gets_a_reply_and_wakes_the_window_once() {
    let path = socket("echo");
    let wakes = Arc::new(AtomicUsize::new(0));
    let counted = wakes.clone();
    let listener = Listener::bind(&path, move || {
        counted.fetch_add(1, Ordering::SeqCst);
    })
    .unwrap();
    let mut app = App::new(Box::new(NoFiles));
    app.listen(listener);
    let (client_path, client_wakes) = (path.clone(), wakes.clone());
    let seen = with_frames(&mut app, move || {
        (0..3)
            .map(|_| {
                let before = client_wakes.load(Ordering::SeqCst);
                let reply = send(&client_path, "view").unwrap();
                (reply, client_wakes.load(Ordering::SeqCst) - before)
            })
            .collect::<Vec<_>>()
    });
    for (reply, woken) in seen {
        assert_eq!(
            reply.as_deref(),
            Some("ok selected=- shown=- presenting=- preview=off")
        );
        assert_eq!(woken, 1);
    }
}

#[test]
fn a_socket_left_by_an_ended_run_is_replaced_and_its_file_removed_at_the_end() {
    let path = socket("stale");
    // A socket whose listener is gone leaves its file behind.
    drop(std::os::unix::net::UnixListener::bind(&path).unwrap());
    assert!(path.exists());
    let listener = Listener::bind(&path, || {}).unwrap();
    drop(listener);
    assert!(!path.exists());
}

#[test]
fn a_socket_another_window_listens_on_is_refused() {
    let path = socket("taken");
    let _first = Listener::bind(&path, || {}).unwrap();
    let second = Listener::bind(&path, || {});
    assert_eq!(
        second.err().map(|e| e.kind()),
        Some(std::io::ErrorKind::AddrInUse)
    );
}

#[test]
fn a_send_with_no_listener_is_an_error() {
    let path = socket("none");
    assert!(send(&path, "tree").is_err());
}

/// Files under a directory of their own, paths relative to it, as the window's are to its working
/// directory.
struct DirFiles(PathBuf);

impl Files for DirFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(self.0.join(path)).map_err(|e| format!("{path}: {e}"))
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        let path = self.0.join(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))
    }

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        std::fs::exists(self.0.join(path)).map_err(|e| format!("{path}: {e}"))
    }

    fn canonical(&mut self, path: &str) -> Result<String, String> {
        let canonical =
            std::fs::canonicalize(self.0.join(path)).map_err(|e| format!("{path}: {e}"))?;
        Ok(canonical.to_str().unwrap().to_owned())
    }
}

/// An app listening on a socket of its own, on `files`.
fn listening(name: &str, files: Box<dyn Files>) -> (App, PathBuf) {
    let path = socket(name);
    let mut app = App::new(files);
    app.listen(Listener::bind(&path, || {}).unwrap());
    (app, path)
}

/// Send each line over the socket, one connection per line, while the app shows frames, and
/// return the replies.
fn send_all(app: &mut App, path: &Path, lines: &[&str]) -> Vec<Option<String>> {
    let (path, lines) = (
        path.to_path_buf(),
        lines.iter().map(|l| l.to_string()).collect::<Vec<_>>(),
    );
    with_frames(app, move || {
        lines
            .iter()
            .map(|line| send(&path, line).unwrap())
            .collect()
    })
}

fn one(app: &mut App, path: &Path, line: &str) -> String {
    send_all(app, path, &[line]).remove(0).unwrap()
}

/// After `control`, a socket command changes the document and the view as the window's
/// action does, and once the window took control back, its undo undoes the command.
#[test]
fn a_socket_command_acts_as_the_window_action_and_the_window_undoes_it() {
    let (mut app, path) = listening("acts-like-ui", Box::new(NoFiles));
    let replies = send_all(&mut app, &path, &["control", "add-slide", "select n2"]);
    assert!(
        replies
            .iter()
            .all(|r| r.as_deref().is_some_and(|r| r.starts_with("ok"))),
        "{replies:?}"
    );
    let mut window = App::new(Box::new(NoFiles));
    window.add_slide();
    window.select_node(NodeId(2));
    assert_eq!(app.session().document(), window.session().document());
    assert_eq!(app.session().view(), window.session().view());
    app.take_control();
    app.undo();
    window.undo();
    assert_eq!(app.session().document(), window.session().document());
    assert!(app.session().document().slides.is_empty());
}

/// Every script of the protocol's tests gives through the socket the replies a session
/// gives it, by the rule `rootpoint-headless` follows, `read_line` then `Session::run`.
#[test]
fn the_protocol_scripts_give_the_same_replies_through_the_socket() {
    let scripts =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../rootpoint-protocol/tests/scripts");
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("scripts");
    let _ = std::fs::remove_dir_all(&dir);
    for runs in [
        &["golden.txt"][..],
        &["groups.txt"],
        &["ids-build.txt", "ids-readdress.txt"],
        &["view.txt"],
    ] {
        let (window_dir, session_dir) = (
            dir.join(runs[0]).join("window"),
            dir.join(runs[0]).join("session"),
        );
        for d in [&window_dir, &session_dir] {
            // The golden script's workspace and its asset, and the workspace of the ids scripts.
            std::fs::create_dir_all(d.join("talk")).unwrap();
            std::fs::create_dir_all(d.join("deck")).unwrap();
            std::fs::copy(
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/two-colours.png"),
                d.join("talk/two-colours.png"),
            )
            .unwrap();
        }
        for name in runs {
            let script = std::fs::read_to_string(scripts.join(name)).unwrap();
            let lines: Vec<&str> = script.lines().collect();
            let mut session = Session::new();
            let mut files = DirFiles(session_dir.clone());
            let expected: Vec<Option<String>> = lines
                .iter()
                .map(|line| session.execute(line, &mut files).map(|r| r.to_string()))
                .collect();
            let (mut app, path) = listening(
                &format!("script-{name}"),
                Box::new(DirFiles(window_dir.clone())),
            );
            assert_eq!(one(&mut app, &path, "control"), "ok");
            let got = send_all(&mut app, &path, &lines);
            assert_eq!(got, expected, "{name}");
        }
    }
}

/// At start the window holds the authority to edit, a socket edit is refused and
/// changes nothing, and the queries are answered.
#[test]
fn the_window_holds_control_at_start_and_the_queries_need_none() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("queries");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (mut app, path) = listening("start", Box::new(DirFiles(dir.clone())));
    app.run(Command::Sample);
    let before = app.session().document().clone();
    assert_eq!(app.controller(), Controller::Window);
    assert_eq!(
        one(&mut app, &path, "add-slide"),
        "err the window has control"
    );
    assert_eq!(
        one(&mut app, &path, "select n2"),
        "err the window has control"
    );
    assert_eq!(app.session().document(), &before);
    assert!(one(&mut app, &path, "tree").starts_with("ok s1"));
    assert!(one(&mut app, &path, "view").starts_with("ok selected="));
    assert!(one(&mut app, &path, "render s1 s1.png 192 108").starts_with("ok"));
    assert!(dir.join("s1.png").exists());
}

/// The sample, shown in a few frames so the slide view has a layout.
fn sample(name: &str) -> (App, PathBuf) {
    let (mut app, path) = listening(name, Box::new(NoFiles));
    app.run(Command::Sample);
    // A query gives the app its frames, so the slide view is laid out.
    one(&mut app, &path, "view");
    one(&mut app, &path, "view");
    (app, path)
}

/// A `control` during a stay: the stay ends, its edits one undo step, and the socket's
/// command on another area does not join it, as `design@rootpoint-app@one-controller-at-a-time`
/// has it.
#[test]
fn control_ends_a_stay_and_the_socket_edit_is_a_step_of_its_own() {
    let (mut app, path) = sample("stay");
    let n3 = NodeId(3);
    app.select_node(n3);
    one(&mut app, &path, "view");
    let layout = app.layout().unwrap().clone();
    app.edit_at(n3, layout.rect(n3).unwrap().center(), &layout);
    assert_eq!(app.editing(), Some(n3));
    app.type_str("typed");
    let typed = app.session().document().clone();
    assert_eq!(one(&mut app, &path, "control"), "ok");
    assert_eq!(app.editing(), None);
    assert!(!app.session().group_open());
    assert_eq!(one(&mut app, &path, "text n5 \"agent\""), "ok");
    app.take_control();
    app.undo();
    assert_eq!(
        app.session().document(),
        &typed,
        "one undo reverts the socket's edit alone"
    );
}

/// A `control` during a divider drag ends it as its release does, and during a grip drag
/// cancels it.
#[test]
fn control_ends_a_divider_drag_as_its_release_and_cancels_a_grip_drag() {
    let (mut app, path) = sample("drags");
    let before = app.session().document().clone();
    let layout = app.layout().unwrap().clone();
    let d = layout.dividers[0].clone();
    app.grab_divider(d.split, d.index);
    app.drag_divider(0.4);
    let dragged = app.session().document().clone();
    assert_ne!(dragged, before);
    assert_eq!(one(&mut app, &path, "control"), "ok");
    assert!(!app.session().group_open());
    assert_eq!(app.session().document(), &dragged);
    assert_eq!(one(&mut app, &path, "undo"), "ok");
    assert_eq!(app.session().document(), &before);
    // Back to the window, then a grip drag, which `control` cancels: no step is left.
    assert_eq!(one(&mut app, &path, "release"), "ok");
    app.grab_grip(NodeId(3));
    assert!(app.session().group_open());
    assert_eq!(one(&mut app, &path, "control"), "ok");
    assert!(!app.session().group_open());
    assert_eq!(app.session().document(), &before);
}

/// A `release` with a group open is refused, and the take-over closes the socket's open
/// group with `end`, so one undo reverts its edits; after the take-over, a socket edit is told to
/// stop and wait, until its next `control`.
#[test]
fn the_take_over_closes_the_socket_group_and_its_agent_is_told_to_wait() {
    let (mut app, path) = sample("take-over");
    let before = app.session().document().clone();
    let replies = send_all(
        &mut app,
        &path,
        &[
            "control",
            "begin",
            "text n3 \"a\"",
            "text n5 \"b\"",
            "release",
        ],
    );
    assert_eq!(
        replies.last().unwrap().as_deref(),
        Some("err a group is open")
    );
    assert_eq!(app.controller(), Controller::Socket);
    app.take_control();
    assert_eq!(app.controller(), Controller::TakenFromSocket);
    assert!(!app.session().group_open());
    assert_eq!(
        one(&mut app, &path, "text n3 \"c\""),
        "err the author took control: stop, and wait for the author's instructions"
    );
    app.undo();
    assert_eq!(app.session().document(), &before);
    assert_eq!(one(&mut app, &path, "control"), "ok");
    assert_eq!(one(&mut app, &path, "text n3 \"c\""), "ok");
    assert!(matches!(app.session().document().slides.len(), 4));
    let _ = Reply::Ok(String::new());
}

/// While the socket holds control, every action of the window that changes
/// the document leaves the document and the undo stack as they were, and each view action acts.
#[test]
fn while_the_socket_holds_control_no_window_action_edits() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("frozen-actions");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (mut app, path) = listening("frozen-actions", Box::new(DirFiles(dir.clone())));
    app.run(Command::Sample);
    // Frames lay the slide view out, for `edit_at`.
    one(&mut app, &path, "view");
    one(&mut app, &path, "view");
    assert_eq!(one(&mut app, &path, "control"), "ok");
    let before = app.session().document().clone();
    let n3 = NodeId(3);
    app.select_node(n3);
    assert_eq!(
        app.session().view().selected,
        Some(n3),
        "a view action acts"
    );
    app.add_slide();
    app.remove_slide();
    app.duplicate_slide();
    app.move_slide(rootpoint_model::SlideId(1), 2);
    app.split_middle(rootpoint_app::hit::Edge::Left);
    app.remove_selection();
    app.grab_divider(NodeId(4), 0);
    app.drag_divider(0.6);
    app.grab_grip(n3);
    app.reset_divider(NodeId(4), 0);
    app.set_text_content();
    app.set_style(rootpoint_protocol::StyleOption::Bold(true));
    app.add_asset("missing.png".to_owned());
    app.split_at(rootpoint_app::hit::Preview {
        node: n3,
        axis: rootpoint_model::Axis::LeftToRight,
        at: 0.25,
        side: rootpoint_model::Side::Before,
        from: Default::default(),
        to: Default::default(),
    });
    app.set_image(
        rootpoint_model::AssetId(19),
        rootpoint_model::ImageFit::Fill,
    );
    app.drop_grip(Some((NodeId(5), rootpoint_app::hit::Zone::Swap)));
    app.name_style("Title".to_owned());
    app.make_custom();
    app.add_style();
    // A selection in the list of styles is the window's own state, and acts.
    app.select_style("Body");
    assert_eq!(app.selected_style(), Some("Body"));
    app.duplicate_style();
    app.rename_style("Text".to_owned());
    app.set_selected_style(rootpoint_protocol::StyleOption::Size(64.0));
    app.remove_style();
    assert!(app.removal().is_none(), "a removal opened its dialog");
    app.confirm_removal();
    let layout = app.layout().unwrap().clone();
    app.edit_at(n3, layout.rect(n3).unwrap().center(), &layout);
    assert_eq!(app.editing(), None);
    app.undo();
    app.redo();
    app.save();
    app.open_sample();
    std::fs::create_dir_all(dir.join("deck")).unwrap();
    app.open_workspace("deck".to_owned());
    assert_eq!(app.session().document(), &before);
    assert!(!app.session().group_open());
    assert!(app.session().root().is_none(), "the workspace was opened");
    assert!(!app.can_split_middle(rootpoint_model::Axis::LeftToRight));
    // The undo stack is as it was: empty, since the sample was opened.
    app.take_control();
    assert!(matches!(app.run(Command::Undo), Reply::Err(_)));
    // The view actions act.
    app.show_slide(rootpoint_model::SlideId(7));
    assert_eq!(app.shown_slide(), Some(rootpoint_model::SlideId(7)));
}

/// The socket's `control` closes the dialog of a style's removal, as it ends a gesture, and the
/// removal sends nothing.
#[test]
fn the_socket_s_control_closes_the_dialog_of_a_removal() {
    let (mut app, path) = sample("removal-dialog");
    let before = app.session().document().clone();
    app.select_style("Body");
    app.remove_style();
    assert!(app.removal().is_some(), "the dialog is open");
    assert_eq!(one(&mut app, &path, "control"), "ok");
    assert!(app.removal().is_none(), "the dialog is still open");
    assert_eq!(app.session().document(), &before);
}

/// While the socket holds control, a secondary click still selects and opens the node menu, whose
/// "Select" works and whose other entries are disabled; the socket's `control` closes the menu, as
/// it ends a gesture.
#[test]
fn while_the_socket_holds_control_the_node_menu_only_selects() {
    let (mut app, path) = sample("frozen-menu");
    let (ctx, mut time) = (Context::default(), 0.0);
    for _ in 0..3 {
        frame_with(&mut app, &ctx, &mut time, vec![]);
    }
    let at = app.layout().unwrap().rect(NodeId(3)).unwrap().center();
    let secondary = |pressed| {
        vec![
            eframe::egui::Event::PointerMoved(at),
            eframe::egui::Event::PointerButton {
                pos: at,
                button: eframe::egui::PointerButton::Secondary,
                pressed,
                modifiers: eframe::egui::Modifiers::NONE,
            },
        ]
    };
    // The window's menu closes when the socket takes control.
    frame_with(&mut app, &ctx, &mut time, secondary(true));
    frame_with(&mut app, &ctx, &mut time, secondary(false));
    assert_eq!(app.menu_node(), Some(NodeId(3)));
    assert_eq!(one(&mut app, &path, "control"), "ok");
    assert_eq!(app.menu_node(), None, "control leaves the menu open");
    let before = app.session().document().clone();
    frame_with(&mut app, &ctx, &mut time, secondary(true));
    frame_with(&mut app, &ctx, &mut time, secondary(false));
    frame_with(&mut app, &ctx, &mut time, vec![]);
    assert_eq!(app.menu_node(), Some(NodeId(3)));
    for item in app.menu_items() {
        let select = item.section == rootpoint_app::MenuSection::Select;
        assert_eq!(item.enabled, select, "{}", item.label);
    }
    let remove = app.menu_entry_rect("Remove n3").unwrap().center();
    frame_with(&mut app, &ctx, &mut time, press(remove, true));
    frame_with(&mut app, &ctx, &mut time, press(remove, false));
    assert_eq!(app.session().document(), &before);
    frame_with(&mut app, &ctx, &mut time, secondary(true));
    frame_with(&mut app, &ctx, &mut time, secondary(false));
    frame_with(&mut app, &ctx, &mut time, vec![]);
    let parent = app.menu_entry_rect("n6 beside").unwrap().center();
    frame_with(&mut app, &ctx, &mut time, press(parent, true));
    frame_with(&mut app, &ctx, &mut time, press(parent, false));
    assert_eq!(app.session().view().selected, Some(NodeId(6)));
    assert_eq!(app.session().document(), &before);
}

/// While the socket holds control, the button "Preview" still enters and leaves the final view,
/// which changes only the view.
#[test]
fn while_the_socket_holds_control_the_preview_button_works() {
    let (mut app, path) = sample("frozen-preview");
    assert_eq!(one(&mut app, &path, "control"), "ok");
    let (ctx, mut time) = (Context::default(), 0.0);
    for _ in 0..3 {
        frame_with(&mut app, &ctx, &mut time, vec![]);
    }
    let button = app.button_rect("Preview").unwrap().center();
    frame_with(&mut app, &ctx, &mut time, press(button, true));
    frame_with(&mut app, &ctx, &mut time, press(button, false));
    assert!(app.final_view());
    frame_with(&mut app, &ctx, &mut time, press(button, true));
    frame_with(&mut app, &ctx, &mut time, press(button, false));
    assert!(!app.final_view());
}

/// A frame of the app with input events, and the cursor it asks for.
fn frame_with(
    app: &mut App,
    ctx: &Context,
    time: &mut f64,
    events: Vec<eframe::egui::Event>,
) -> eframe::egui::CursorIcon {
    frame_out(app, ctx, time, events, eframe::egui::Modifiers::NONE)
        .platform_output
        .cursor_icon
}

/// A frame of the app with input events and held modifiers, and its whole output.
fn frame_out(
    app: &mut App,
    ctx: &Context,
    time: &mut f64,
    mut events: Vec<eframe::egui::Event>,
    modifiers: eframe::egui::Modifiers,
) -> eframe::egui::FullOutput {
    events.insert(0, eframe::egui::Event::ModifiersChanged(modifiers));
    *time += 1.0 / 60.0;
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 700.0))),
        max_texture_side: Some(8192),
        time: Some(*time),
        events,
        ..Default::default()
    };
    let mut out = ctx.run_ui(input, |ui| app.frame(ui));
    out.textures_delta.clear();
    out
}

fn press(pos: Pos2, pressed: bool) -> Vec<eframe::egui::Event> {
    vec![
        eframe::egui::Event::PointerMoved(pos),
        eframe::egui::Event::PointerButton {
            pos,
            button: eframe::egui::PointerButton::Primary,
            pressed,
            modifiers: eframe::egui::Modifiers::NONE,
        },
    ]
}

/// While the socket holds control, a divider drag, a grip drag and a click
/// on the selected text area change nothing and start no editing, the divider under the pointer
/// asks for no resize cursor, and a click on an area still selects it.
#[test]
fn while_the_socket_holds_control_no_pointer_gesture_edits() {
    let (mut app, path) = sample("frozen-pointer");
    assert_eq!(one(&mut app, &path, "control"), "ok");
    let (ctx, mut time) = (Context::default(), 0.0);
    for _ in 0..3 {
        frame_with(&mut app, &ctx, &mut time, vec![]);
    }
    let before = app.session().document().clone();
    let layout = app.layout().unwrap().clone();
    let d = &layout.dividers[0];
    let on_divider = d.from + (d.to - d.from) / 2.0;
    let hover = frame_with(
        &mut app,
        &ctx,
        &mut time,
        vec![eframe::egui::Event::PointerMoved(on_divider)],
    );
    assert_eq!(hover, eframe::egui::CursorIcon::Default, "no resize cursor");
    frame_with(&mut app, &ctx, &mut time, press(on_divider, true));
    frame_with(
        &mut app,
        &ctx,
        &mut time,
        vec![eframe::egui::Event::PointerMoved(
            on_divider + Vec2::new(40.0, 40.0),
        )],
    );
    frame_with(
        &mut app,
        &ctx,
        &mut time,
        press(on_divider + Vec2::new(40.0, 40.0), false),
    );
    assert_eq!(app.session().document(), &before);
    // A click selects an area; a second click on it, a text area, enters no editing.
    let n3 = NodeId(3);
    let centre = layout.rect(n3).unwrap().center();
    for _ in 0..2 {
        frame_with(&mut app, &ctx, &mut time, press(centre, true));
        frame_with(&mut app, &ctx, &mut time, press(centre, false));
        frame_with(&mut app, &ctx, &mut time, vec![]);
    }
    assert_eq!(app.session().view().selected, Some(n3));
    assert_eq!(app.editing(), None);
    // The selected area shows no grip, and a press where the grip stands drags nothing.
    let grip = layout.grip(n3).unwrap().center();
    frame_with(&mut app, &ctx, &mut time, press(grip, true));
    frame_with(
        &mut app,
        &ctx,
        &mut time,
        vec![eframe::egui::Event::PointerMoved(
            grip + Vec2::new(200.0, 0.0),
        )],
    );
    frame_with(
        &mut app,
        &ctx,
        &mut time,
        press(grip + Vec2::new(200.0, 0.0), false),
    );
    assert_eq!(app.session().document(), &before);
    assert!(!app.session().group_open());
}

/// Whether a frame's output fills a rectangle at `rect`, as the grip is drawn.
fn fills(out: &eframe::egui::FullOutput, rect: Rect) -> bool {
    out.shapes.iter().any(|c| match &c.shape {
        eframe::egui::Shape::Rect(r) => {
            r.rect == rect && r.fill != eframe::egui::Color32::TRANSPARENT
        }
        _ => false,
    })
}

/// Whether a frame's output draws the line from `from` to `to`, as the preview of a split is.
fn draws_line(out: &eframe::egui::FullOutput, from: Pos2, to: Pos2) -> bool {
    out.shapes.iter().any(|c| match &c.shape {
        eframe::egui::Shape::LineSegment { points, .. } => *points == [from, to],
        _ => false,
    })
}

/// While the socket holds control, the selected area's
/// grip is not drawn, Ctrl over the selection draws no preview line and its click splits nothing,
/// a double-click on a divider resets nothing, and a drag of a row of the slide list moves no
/// slide. Each mark is drawn while the window holds control, which the test checks first.
#[test]
fn while_the_socket_holds_control_no_mark_of_a_gesture_is_drawn() {
    use eframe::egui::{Event, Modifiers};
    let (mut app, path) = sample("frozen-marks");
    let (ctx, mut time) = (Context::default(), 0.0);
    let n3 = NodeId(3);
    app.select_node(n3);
    // Whether a frame with Ctrl held over n3 fills its grip and draws the preview line, the
    // places read from the layout of the frame before, since the banner moves the slide view.
    let marks = |app: &mut App, time: &mut f64| {
        for _ in 0..3 {
            frame_with(app, &ctx, time, vec![]);
        }
        let layout = app.layout().unwrap().clone();
        let r = layout.rect(n3).unwrap();
        let at = Pos2::new(r.min.x + r.width() * 0.3, r.center().y);
        let line = layout
            .preview(n3, rootpoint_model::Axis::LeftToRight, at, false)
            .unwrap();
        let out = frame_out(
            app,
            &ctx,
            time,
            vec![Event::PointerMoved(at)],
            Modifiers::COMMAND,
        );
        let drawn = (
            fills(&out, layout.grip(n3).unwrap()),
            draws_line(&out, line.from, line.to),
        );
        (drawn, at, layout)
    };
    assert_eq!(
        marks(&mut app, &mut time).0,
        (true, true),
        "the window draws both marks"
    );
    assert_eq!(one(&mut app, &path, "control"), "ok");
    let (drawn, at, layout) = marks(&mut app, &mut time);
    assert_eq!(drawn, (false, false));
    let before = app.session().document().clone();
    let click = |pressed| Event::PointerButton {
        pos: at,
        button: eframe::egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::COMMAND,
    };
    frame_out(
        &mut app,
        &ctx,
        &mut time,
        vec![click(true)],
        Modifiers::COMMAND,
    );
    frame_out(
        &mut app,
        &ctx,
        &mut time,
        vec![click(false)],
        Modifiers::COMMAND,
    );
    let d = &layout.dividers[0];
    let on_divider = d.from + (d.to - d.from) / 2.0;
    for _ in 0..2 {
        frame_with(&mut app, &ctx, &mut time, press(on_divider, true));
        frame_with(&mut app, &ctx, &mut time, press(on_divider, false));
    }
    let rows = app.slide_rows().to_vec();
    let (from, to) = (rows[0].1.center(), rows[2].1.center());
    frame_with(&mut app, &ctx, &mut time, press(from, true));
    for i in 1..=10 {
        let p = from + (to - from) * (i as f32 / 10.0);
        frame_with(&mut app, &ctx, &mut time, vec![Event::PointerMoved(p)]);
    }
    frame_with(&mut app, &ctx, &mut time, press(to, false));
    frame_with(&mut app, &ctx, &mut time, vec![]);
    assert_eq!(app.session().document(), &before);
}

/// The listener refuses a path that holds another kind of file, and leaves the file as it is.
#[test]
fn a_path_that_holds_a_file_is_refused_and_the_file_kept() {
    let path = socket("a-file");
    std::fs::write(&path, b"precious").unwrap();
    let bound = Listener::bind(&path, || {});
    assert_eq!(
        bound.err().map(|e| e.kind()),
        Some(std::io::ErrorKind::AlreadyExists)
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"precious");
}

/// A listener whose socket file another window has replaced leaves that window's file at its end.
#[test]
fn a_listener_removes_no_other_windows_socket() {
    let path = socket("replaced");
    let first = Listener::bind(&path, || {}).unwrap();
    std::fs::remove_file(&path).unwrap();
    let second = Listener::bind(&path, || {}).unwrap();
    drop(first);
    assert!(path.exists(), "the second window's socket is kept");
    drop(second);
    assert!(!path.exists());
}

/// A command the window never ran, since it was closed first, is an error for its client, never
/// the empty answer of a comment.
#[test]
fn a_command_the_window_closed_on_is_an_error() {
    let (app, path) = listening("closed", Box::new(NoFiles));
    let client = thread::spawn(move || send(&path, "add-slide"));
    thread::sleep(Duration::from_millis(100));
    drop(app);
    let reply = client.join().unwrap();
    assert_eq!(
        reply.err().map(|e| e.kind()),
        Some(std::io::ErrorKind::UnexpectedEof)
    );
}

/// The refusal after a take-over lasts until the socket's next `control`, a `release` included,
/// and a `release` while the window holds control, its own group open, is no error.
#[test]
fn the_refusal_after_a_take_over_lasts_until_control() {
    let (mut app, path) = sample("refusal-lasts");
    assert_eq!(one(&mut app, &path, "control"), "ok");
    app.take_control();
    assert_eq!(one(&mut app, &path, "release"), "ok");
    assert_eq!(
        one(&mut app, &path, "add-slide"),
        "err the author took control: stop, and wait for the author's instructions"
    );
    assert_eq!(app.controller(), Controller::TakenFromSocket);
    app.run(Command::Begin);
    assert_eq!(one(&mut app, &path, "release"), "ok");
    app.run(Command::End);
}

/// A batch runs over one connection, and a take-over between two of its lines ends it
/// with one refusal; the lines after it reach no frame.
#[test]
fn a_take_over_ends_a_batch_with_one_refusal() {
    use rootpoint_protocol::control::Connection;
    let (mut app, path) = sample("batch");
    let slides = app.session().document().slides.len();
    let connection = Connection::open(&path).unwrap();
    let (connection, first) = with_frames(&mut app, move || {
        let mut connection = connection;
        let replies: Vec<_> = ["control", "add-slide"]
            .iter()
            .map(|line| connection.send(line).unwrap())
            .collect();
        (connection, replies)
    });
    assert_eq!(first[0].as_deref(), Some("ok"));
    assert!(first[1].as_deref().is_some_and(|r| r.starts_with("ok")));
    app.take_control();
    let rest = with_frames(&mut app, move || {
        let mut connection = connection;
        let refused = connection.send("add-slide");
        let after = connection.send("add-slide");
        (refused.unwrap(), after.err().map(|e| e.kind()))
    });
    assert_eq!(
        rest.0.as_deref(),
        Some("err the author took control: stop, and wait for the author's instructions")
    );
    assert!(
        rest.1.is_some(),
        "the connection is closed after the refusal"
    );
    assert_eq!(app.session().document().slides.len(), slides + 1);
}

/// After a take-over, a query that fails for a reason of its own is answered and ends nothing: the
/// connection stays open for the batch's next query, since only the take-over's refusal ends it.
#[test]
fn a_query_error_after_a_take_over_keeps_the_connection() {
    use rootpoint_protocol::control::Connection;
    let (mut app, path) = sample("query-after-take-over");
    let connection = Connection::open(&path).unwrap();
    let connection = with_frames(&mut app, move || {
        let mut connection = connection;
        assert_eq!(connection.send("control").unwrap().as_deref(), Some("ok"));
        connection
    });
    app.take_control();
    let replies = with_frames(&mut app, move || {
        let mut connection = connection;
        let failed = connection.send("tree s99").unwrap();
        let next = connection.send("view").map(|r| r.is_some());
        (failed, next.ok())
    });
    assert!(replies.0.as_deref().is_some_and(|r| r.starts_with("err")));
    assert_eq!(replies.1, Some(true), "the connection stays open");
}

/// Over one connection, a comment gets no reply, so the reply that follows is the next command's:
/// a reply sent for a line with no command would be read as the answer to the next.
#[test]
fn a_comment_in_a_batch_gets_no_reply() {
    use rootpoint_protocol::control::Connection;
    let (mut app, path) = sample("comment-in-batch");
    let connection = Connection::open(&path).unwrap();
    let replies = with_frames(&mut app, move || {
        let mut connection = connection;
        ["# a comment", "", "view"]
            .iter()
            .map(|line| connection.send(line).unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(replies[0], None);
    assert_eq!(replies[1], None);
    assert!(
        replies[2]
            .as_deref()
            .is_some_and(|r| r.starts_with("ok selected="))
    );
}
