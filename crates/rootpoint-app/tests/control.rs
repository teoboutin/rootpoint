//! The window's control socket, per `spec@first-prototype@live-control`: lines sent by
//! `rootpoint_protocol::control::send` reach the app's frames and get their replies, the listener
//! wakes the window once for each line, and a socket left by an earlier run is replaced while one
//! another window listens on is refused.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui::{Context, Pos2, RawInput, Rect, Vec2};
use rootpoint_app::App;
use rootpoint_app::control::Listener;
use rootpoint_protocol::Files;
use rootpoint_protocol::control::send;

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
/// GUI's thread does, until the client ends; a client still running after 5 s fails the test.
fn with_frames<T: Send + 'static>(app: &mut App, client: impl FnOnce() -> T + Send + 'static) -> T {
    let ctx = Context::default();
    let handle = thread::spawn(client);
    let start = Instant::now();
    while !handle.is_finished() {
        assert!(
            start.elapsed() < Duration::from_secs(5),
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
                let reply = send(&client_path, "add-slide").unwrap();
                (reply, client_wakes.load(Ordering::SeqCst) - before)
            })
            .collect::<Vec<_>>()
    });
    for (reply, woken) in seen {
        assert_eq!(reply.as_deref(), Some("ok"));
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
