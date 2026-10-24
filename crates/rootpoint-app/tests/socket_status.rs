//! The status of a window whose control socket could not be bound, in a file of its own since it
//! sets its process's environment.

#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;

use rootpoint_app::App;
use rootpoint_protocol::{Files, parse_command};

struct NoFiles;

impl Files for NoFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        Err(format!("{path}: no such file"))
    }
    fn write(&mut self, _: &str, _: &[u8]) -> Result<(), String> {
        Ok(())
    }
    fn exists(&mut self, _: &str) -> Result<bool, String> {
        Ok(false)
    }
    fn canonical(&mut self, path: &str) -> Result<String, String> {
        Ok(path.to_owned())
    }
}

/// A socket that cannot be bound leaves the window without one, and the status says why for as
/// long as the window has none, past the reply of the next command.
#[test]
fn the_reason_a_window_has_no_socket_outlives_the_next_command() {
    let base = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("socket-status");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("rootpoint")).unwrap();
    std::fs::set_permissions(
        base.join("rootpoint"),
        std::fs::Permissions::from_mode(0o777),
    )
    .unwrap();
    // SAFETY: this test binary runs this one test, on one thread.
    unsafe { std::env::set_var("XDG_RUNTIME_DIR", &base) };
    let mut app = App::native(Box::new(NoFiles));
    app.listen_at_store(|| {});
    let says = |app: &App| app.status().iter().any(|l| l.contains("no control socket"));
    assert!(says(&app), "{:?}", app.status());
    let _ = app.run(parse_command("add-slide").unwrap());
    assert!(says(&app), "after a command: {:?}", app.status());
}
