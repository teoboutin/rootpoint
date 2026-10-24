//! The runtime directory of the control sockets is the user's alone: one another user made, or one
//! any user may enter, is refused; and a relative variable is passed over, so a pointer file always
//! names an absolute socket. In a file of its own, since it sets its process's environment.

#![cfg(unix)]

mod common;

use std::os::unix::fs::PermissionsExt;

use rootpoint_protocol::workspace::{make_runtime_dir, runtime_dir};

#[test]
fn the_runtime_directory_is_the_user_s_alone() {
    let base = std::fs::canonicalize(common::scratch("runtime-dir")).unwrap();
    // SAFETY: this test binary runs this one test, on one thread.
    unsafe {
        std::env::set_var("XDG_RUNTIME_DIR", "relative");
        std::env::set_var("TMPDIR", &base);
    }
    assert_eq!(
        runtime_dir(),
        base.join("rootpoint"),
        "a relative variable is passed over"
    );
    let dir = make_runtime_dir().unwrap();
    let mode = std::fs::metadata(&dir).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o700);
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o777)).unwrap();
    assert!(
        make_runtime_dir().is_err(),
        "a directory any user may enter is refused"
    );
}
