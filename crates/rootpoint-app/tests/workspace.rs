//! The native window's hold on a workspace: the state label names each state of the store, a close
//! in memory with an unsaved edit asks first, a socket renamed while bound still answers, and two
//! opens of one workspace at one moment bind one socket, the race of the closed
//! `two-windows-bind-one-path` gone.

#![cfg(unix)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;

use eframe::egui::{
    Context, Event, Modifiers, PointerButton, RawInput, ViewportCommand, ViewportEvent, ViewportId,
};
use rootpoint_app::App;
use rootpoint_app::control::Listener;
use rootpoint_protocol::{Files, Reply, Session, parse_command};

/// Files held in memory, by path, shared with the test.
#[derive(Clone, Default)]
struct MemoryFiles(Rc<RefCell<BTreeMap<String, Vec<u8>>>>);

impl Files for MemoryFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        self.0
            .borrow()
            .get(path)
            .cloned()
            .ok_or(format!("{path}: no such file"))
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        self.0.borrow_mut().insert(path.to_owned(), bytes.to_vec());
        Ok(())
    }

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        Ok(self.0.borrow().contains_key(path))
    }

    fn canonical(&mut self, path: &str) -> Result<String, String> {
        Ok(path.to_owned())
    }
}

/// The native file system, paths relative to the working directory.
struct NativeFiles;

impl Files for NativeFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|e| format!("{path}: {e}"))
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        std::fs::write(path, bytes).map_err(|e| format!("{path}: {e}"))
    }

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        // Anything at the path, a symlink whose target is missing included, as `Files::exists` asks.
        match std::fs::symlink_metadata(path) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(format!("{path}: {e}")),
        }
    }

    fn canonical(&mut self, path: &str) -> Result<String, String> {
        let canonical = std::fs::canonicalize(path).map_err(|e| format!("{path}: {e}"))?;
        Ok(canonical.to_str().unwrap().to_owned())
    }
}

fn ok(app: &mut App, line: &str) {
    let reply = app.run(parse_command(line).unwrap());
    assert!(matches!(reply, Reply::Ok(_)), "{line}: {reply}");
}

/// One frame, with the close of the window asked when `close` is set; whether the frame cancelled
/// the close.
fn frame(ctx: &Context, app: &mut App, close: bool) -> bool {
    frame_with(ctx, app, close, Vec::new())
}

/// One frame with these events.
fn frame_with(ctx: &Context, app: &mut App, close: bool, events: Vec<Event>) -> bool {
    let mut input = RawInput {
        events,
        ..Default::default()
    };
    if close {
        input
            .viewports
            .entry(ViewportId::ROOT)
            .or_default()
            .events
            .push(ViewportEvent::Close);
    }
    let mut out = ctx.run_ui(input, |ui| app.frame(ui));
    // No painter applies the texture deltas, and egui refuses to drop them unapplied.
    out.textures_delta.clear();
    out.viewport_output
        .get(&ViewportId::ROOT)
        .is_some_and(|v| v.commands.contains(&ViewportCommand::CancelClose))
}

/// The label of `design@rootpoint-app@visible-mode` names each state of a window with no autosave:
/// in memory with no pack, with a pack saved or changed since, and a workspace saved or changed
/// since its save; read after a frame drew it.
#[test]
fn the_state_label_names_each_state_of_the_store() {
    let ctx = Context::default();
    let mut app = App::new(Box::new(MemoryFiles::default()));
    let label = |app: &mut App, line: Option<&str>| {
        if let Some(line) = line {
            ok(app, line);
        }
        frame(&ctx, app, false);
        app.state_label()
    };
    assert_eq!(label(&mut app, Some("sample")), "In memory, not saved");
    assert_eq!(label(&mut app, Some("add-slide")), "In memory, not saved");
    assert_eq!(
        label(&mut app, Some("pack s.rpk")),
        "In memory, saved to s.rpk"
    );
    assert_eq!(
        label(&mut app, Some("add-slide")),
        "In memory, saved to s.rpk, 1 change since"
    );
    assert_eq!(
        label(&mut app, Some("add-slide")),
        "In memory, saved to s.rpk, 2 changes since"
    );
    assert_eq!(label(&mut app, Some("open w")), "Workspace w, saved");
    assert_eq!(
        label(&mut app, Some("add-slide")),
        "Workspace w, 1 change since the save"
    );
    assert_eq!(label(&mut app, Some("save")), "Workspace w, saved");
    assert!(
        app.button_rect("state").is_some(),
        "the frame draws the label"
    );
}

/// A click of the button at `label`, by a press and a release over it.
fn click(ctx: &Context, app: &mut App, label: &str) {
    let at = app.button_rect(label).unwrap().center();
    let button = |pressed| Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame_with(ctx, app, false, vec![Event::PointerMoved(at), button(true)]);
    frame_with(ctx, app, false, vec![button(false)]);
}

/// #unsaved-close-warning, natively: a close in memory with an unsaved edit is cancelled and asks;
/// "Keep open" keeps the window; a close with nothing unsaved, or of a workspace, does not ask.
#[test]
fn a_close_in_memory_with_an_unsaved_edit_asks() {
    let ctx = Context::default();
    let mut app = App::new(Box::new(MemoryFiles::default()));
    ok(&mut app, "sample");
    assert!(!frame(&ctx, &mut app, true), "nothing unsaved asks nothing");
    ok(&mut app, "add-slide");
    assert!(frame(&ctx, &mut app, true), "the close is cancelled");
    assert!(app.close_question());
    frame(&ctx, &mut app, false);
    assert!(app.close_question(), "the question stays until answered");
    click(&ctx, &mut app, "Keep open");
    assert!(!app.close_question(), "Keep open keeps the window");
    assert!(frame(&ctx, &mut app, true), "a new close asks again");
    frame(&ctx, &mut app, false);
    click(&ctx, &mut app, "Close");
    assert!(
        !frame(&ctx, &mut app, true),
        "after Close, the close goes on"
    );
    ok(&mut app, "open w");
    ok(&mut app, "add-slide");
    let mut fresh = App::new(Box::new(MemoryFiles::default()));
    ok(&mut fresh, "open w");
    ok(&mut fresh, "add-slide");
    assert!(!frame(&ctx, &mut fresh, true), "a workspace never asks");
    assert!(!fresh.close_question());
}

/// A directory of the build's scratch space, made empty, short enough for a socket's path.
fn scratch(name: &str) -> PathBuf {
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let dir = if base.as_os_str().len() + name.len() + 20 < 100 {
        base.join(name)
    } else {
        PathBuf::from("/tmp").join(format!("rootpoint-{}-{name}", std::process::id()))
    };
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::canonicalize(dir).unwrap()
}

/// AC5 of `milestone@plans@asset-storage`: a socket file renamed while bound accepts connections at its new path,
/// and the listener dropped after the rename removes the renamed file.
#[test]
fn a_renamed_socket_answers_at_its_new_path() {
    let dir = scratch("renamed-socket");
    let (old, new) = (dir.join("a.sock"), dir.join("b.sock"));
    let mut listener = Listener::bind(&old, || {}).unwrap();
    listener.rename_to(&new).unwrap();
    assert!(!old.exists() && new.exists());
    let mut app = App::new(Box::new(MemoryFiles::default()));
    app.listen(listener);
    let path = new.clone();
    let reply = std::thread::spawn(move || rootpoint_protocol::control::send(&path, "view"));
    let ctx = Context::default();
    while !reply.is_finished() {
        frame(&ctx, &mut app, false);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let reply = reply.join().unwrap().unwrap();
    assert!(reply.is_some_and(|r| r.starts_with("ok selected=")));
    drop(app);
    assert!(!new.exists(), "the renamed file is removed on drop");
}

/// The race of the closed `two-windows-bind-one-path`: two threads behind a barrier each open one
/// workspace through the host's function, then bind its socket where a stale one lies. With the
/// lock, exactly one open of each round succeeds, the other refused before its bind, so no round
/// has two binds; without it, both opens succeed, which the test sees, whether the binds then race
/// or not.
#[test]
fn two_opens_of_one_workspace_bind_one_socket() {
    let dir = scratch("bind-race");
    let workspace = dir.join("ws");
    std::fs::create_dir_all(&workspace).unwrap();
    let socket = dir.join("ws.sock");
    for round in 0..200 {
        // A socket left by a listener dropped before, which a bind removes.
        drop(std::os::unix::net::UnixListener::bind(&socket).unwrap());
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let threads: Vec<_> = (0..2)
            .map(|_| {
                let (barrier, workspace, socket) =
                    (barrier.clone(), workspace.clone(), socket.clone());
                std::thread::spawn(move || {
                    let mut session = Session::new();
                    let mut lock = None;
                    let open = parse_command(&format!("open {}", workspace.display())).unwrap();
                    barrier.wait();
                    let reply = rootpoint_protocol::workspace::run(
                        &mut session,
                        open,
                        &mut NativeFiles,
                        &mut lock,
                    );
                    let bound = match reply {
                        Reply::Ok(_) => Listener::bind(&socket, || {}).ok(),
                        Reply::Err(_) => None,
                    };
                    // Both threads hold what they took until both have tried.
                    barrier.wait();
                    let n = (usize::from(lock.is_some()), usize::from(bound.is_some()));
                    drop((bound, lock));
                    n
                })
            })
            .collect();
        let results: Vec<(usize, usize)> = threads.into_iter().map(|t| t.join().unwrap()).collect();
        let opens: usize = results.iter().map(|r| r.0).sum();
        let binds: usize = results.iter().map(|r| r.1).sum();
        assert_eq!(opens, 1, "round {round}: {opens} opens");
        assert!(binds <= 1, "round {round}: {binds} binds");
        let _ = std::fs::remove_file(&socket);
    }
}
