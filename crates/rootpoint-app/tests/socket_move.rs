//! The native window's socket follows its store, by the table "The store's change to the socket's
//! name" of the milestone: bound at its store's name, renamed when an open changes the store, with
//! the pointer file of the workspace it holds. In a file of its own, since it sets the runtime
//! directory of its process.

#![cfg(unix)]

use rootpoint_app::App;
use rootpoint_protocol::workspace::{pointer_file, socket_path};
use rootpoint_protocol::{Files, Reply, parse_command};

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

#[test]
fn the_socket_follows_the_store() {
    let base = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let runtime = if base.as_os_str().len() < 40 {
        base.join("socket-move")
    } else {
        std::path::PathBuf::from("/tmp").join(format!("rootpoint-{}-move", std::process::id()))
    };
    let _ = std::fs::remove_dir_all(&runtime);
    std::fs::create_dir_all(&runtime).unwrap();
    let runtime = std::fs::canonicalize(runtime).unwrap();
    // SAFETY: this test binary runs this one test, on one thread, before any other reads the
    // environment.
    unsafe { std::env::set_var("XDG_RUNTIME_DIR", &runtime) };
    let (a, b) = (runtime.join("a"), runtime.join("b"));
    std::fs::create_dir_all(&a).unwrap();
    std::fs::create_dir_all(&b).unwrap();
    let mut app = App::native(Box::new(NativeFiles));
    app.listen_at_store(|| {});
    let memory = socket_path(None);
    assert!(memory.exists());
    let run = |app: &mut App, line: &str| {
        let reply = app.run(parse_command(line).unwrap());
        assert!(matches!(reply, Reply::Ok(_)), "{line}: {reply}");
    };
    run(&mut app, &format!("open {}", a.display()));
    let at_a = socket_path(Some(a.to_str().unwrap()));
    assert!(at_a.exists() && !memory.exists());
    assert_eq!(
        std::fs::read_to_string(pointer_file(&a)).unwrap(),
        at_a.to_str().unwrap()
    );
    // Opening the workspace it holds again keeps its lock, which a second lock of the file in
    // this process would refuse.
    run(&mut app, &format!("open {}", a.display()));
    run(&mut app, &format!("open {}", b.display()));
    let at_b = socket_path(Some(b.to_str().unwrap()));
    assert!(at_b.exists() && !at_a.exists());
    assert!(!pointer_file(&a).exists());
    // The lock of a is dropped with the store.
    assert!(rootpoint_protocol::workspace::WorkspaceLock::take(&a).is_ok());
    assert!(rootpoint_protocol::workspace::WorkspaceLock::take(&b).is_err());
    run(&mut app, "sample");
    assert!(memory.exists() && !at_b.exists());
    assert!(
        rootpoint_protocol::workspace::WorkspaceLock::take(&b).is_ok(),
        "the lock goes with the workspace"
    );
    // A line of the socket that opens a workspace moves the socket as the window's own command.
    let ctx = eframe::egui::Context::default();
    let send = |app: &mut App, path: std::path::PathBuf, line: String| {
        let reply =
            std::thread::spawn(move || rootpoint_protocol::control::send(&path, &line).unwrap());
        while !reply.is_finished() {
            let mut out = ctx.run_ui(Default::default(), |ui| app.frame(ui));
            out.textures_delta.clear();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        reply.join().unwrap().unwrap()
    };
    assert_eq!(send(&mut app, memory.clone(), "control".to_owned()), "ok");
    assert_eq!(
        send(&mut app, memory.clone(), format!("open {}", a.display())),
        "ok"
    );
    assert!(at_a.exists() && !memory.exists(), "the socket moved");
    assert!(pointer_file(&a).exists());
    // A rename that fails leaves the socket where it was, and no pointer names the window from the
    // workspace it left.
    std::fs::write(&at_b, b"no socket").unwrap();
    assert_eq!(
        send(&mut app, at_a.clone(), format!("open {}", b.display())),
        "ok"
    );
    assert!(at_a.exists());
    assert!(
        !pointer_file(&a).exists(),
        "the old pointer goes with the old lock"
    );
    std::fs::remove_file(&at_b).unwrap();
    assert_eq!(send(&mut app, at_a.clone(), "sample".to_owned()), "ok");
    drop(app);
    assert!(
        !memory.exists(),
        "the socket is removed when the window closes"
    );
}
