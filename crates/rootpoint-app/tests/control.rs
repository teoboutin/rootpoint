//! The window's control socket, per `spec@first-prototype@live-control`: lines sent by
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

/// A socket path in a scratch directory of its own.
fn socket(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
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
        assert_eq!(reply.as_deref(), Some("ok selected=- shown=- presenting=-"));
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

/// Claim 1: after `control`, a socket command changes the document and the view as the window's
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

/// Claim 3: every script of the protocol's tests gives through the socket the replies a session
/// gives it, the function `rootpoint-headless` runs.
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
            // The golden script's asset, read beside the presentation it saves first.
            std::fs::create_dir_all(d.join("talk")).unwrap();
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

/// Claim 5, the window's control: at start the window holds it, a socket edit is refused and
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

/// Claim 5, `control` during a stay: the stay ends, its edits one undo step, and the socket's
/// command on another area does not join it. The case T5 watches, from the milestone's premortem.
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

/// Claim 5, `control` during a divider drag ends it as its release does, and during a grip drag
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

/// Claim 5, `release` with a group open is refused, and the take-over closes the socket's open
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
    assert!(matches!(app.session().document().slides.len(), 3));
    let _ = Reply::Ok(String::new());
}
