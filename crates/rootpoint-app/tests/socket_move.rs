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
        std::fs::exists(path).map_err(|e| format!("{path}: {e}"))
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
    app.listen_at_store(|| {}).unwrap();
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
    drop(app);
    assert!(
        !memory.exists(),
        "the socket is removed when the window closes"
    );
}
