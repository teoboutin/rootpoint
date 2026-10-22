//! A dropped `Listener` of `design@rootpoint-app@control-socket` closes its listening socket and
//! ends its accept thread, so a process that binds and drops listeners keeps no descriptor of
//! theirs. In a file of its own, so no other test opens descriptors while it counts them.

#![cfg(target_os = "linux")]

use std::time::Duration;

use rootpoint_app::control::Listener;

#[test]
fn a_dropped_listener_keeps_no_descriptor() {
    let count = || std::fs::read_dir("/proc/self/fd").unwrap().count();
    let base = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let dir = if base.as_os_str().len() < 90 {
        base.join("listener-drop")
    } else {
        std::path::PathBuf::from("/tmp").join(format!("rootpoint-{}-drop", std::process::id()))
    };
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("rp.sock");
    let before = count();
    for _ in 0..20 {
        drop(Listener::bind(&path, || {}).unwrap());
    }
    std::thread::sleep(Duration::from_millis(100));
    let after = count();
    assert!(after <= before + 1, "descriptors {before} -> {after}");
}
