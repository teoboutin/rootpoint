//! The lock of a workspace keeps it to one process: a second process is refused, naming the
//! holder's pid, and a crash leaves no stale lock; a socket's name fits a socket's address.

#![cfg(unix)]

mod common;

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use common::*;
use rootpoint_protocol::workspace::{WorkspaceLock, socket_name};

#[test]
fn a_second_process_is_refused_naming_the_holder() {
    let dir = std::fs::canonicalize(scratch("lock-refused")).unwrap();
    std::fs::create_dir_all(dir.join("ws")).unwrap();
    let held = WorkspaceLock::take(&dir.join("ws")).unwrap();
    let out = headless("open ws\n", &dir);
    assert_eq!(
        out,
        format!(
            "err ws is open in another process, pid {}\n",
            std::process::id()
        )
    );
    assert!(dir.join("ws/.rootpoint/.gitignore").exists());
    drop(held);
    assert_eq!(headless("open ws\n", &dir), "ok\n");
}

#[test]
fn a_crash_leaves_no_stale_lock() {
    let dir = std::fs::canonicalize(scratch("lock-crash")).unwrap();
    std::fs::create_dir_all(dir.join("ws")).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_rootpoint-headless"))
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "open ws").unwrap();
    let mut reply = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut reply)
        .unwrap();
    assert_eq!(reply, "ok\n");
    assert!(
        WorkspaceLock::take(&dir.join("ws")).is_err(),
        "the child holds it"
    );
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(WorkspaceLock::take(&dir.join("ws")).is_ok());
}

/// A socket's path stays within the 103 bytes a socket's address holds on Linux and macOS: the
/// workspace's name is cut, at a character's boundary, or left out.
#[test]
fn a_socket_name_fits_a_socket_s_address() {
    let short = std::path::Path::new("/run/user/1000/rootpoint");
    assert_eq!(
        socket_name(short, Some("/home/me/talk")).len(),
        "talk".len() + 14
    );
    assert!(socket_name(short, Some("/a/talk")).starts_with("talk-"));
    assert_ne!(
        socket_name(short, Some("/2025/talk")),
        socket_name(short, Some("/2026/talk")),
        "two workspaces of one name"
    );
    let long_name = format!("/home/me/{}", "é".repeat(40));
    let name = socket_name(short, Some(&long_name));
    assert!(name.len() <= 40 + 14, "{name}");
    let macos = std::path::Path::new("/var/folders/zz/zyxvpxvq6csfxvn_n0000000000000/T/rootpoint");
    for root in ["/a/talk", long_name.as_str()] {
        let name = socket_name(macos, Some(root));
        assert!(macos.join(&name).as_os_str().len() <= 103, "{name}");
    }
    let deep = "/d".repeat(46);
    let deep = std::path::Path::new(&deep);
    assert_eq!(
        socket_name(deep, Some("/a/talk")).len(),
        13,
        "the hash alone"
    );
    assert_eq!(
        socket_name(short, None),
        format!("memory-{}.sock", std::process::id())
    );
}

/// The headless binary takes the lock of an `unpack`'s directory before it writes there, and the
/// directory, holding only the `.rootpoint/` the lock made, counts as empty.
#[test]
fn an_unpack_takes_the_lock_of_its_directory() {
    let dir = std::fs::canonicalize(scratch("lock-unpack")).unwrap();
    let out = headless("sample\nunpack out\nstore\n", &dir);
    let root = dir.join("out");
    assert_eq!(
        out,
        format!("ok\nok\nok workspace {} unsaved=0\n", root.display())
    );
    assert!(root.join(".rootpoint/lock").exists());
    assert!(root.join("presentation.ron").exists());
}

/// A refused `unpack` or `open` leaves no `.rootpoint/` in a directory that is no workspace, and an
/// `unpack` onto a file gets the session's refusal.
#[test]
fn a_refused_open_or_unpack_leaves_no_trace() {
    let dir = std::fs::canonicalize(scratch("lock-refused-trace")).unwrap();
    std::fs::create_dir_all(dir.join("full")).unwrap();
    std::fs::write(dir.join("full/notes.txt"), b"mine").unwrap();
    std::fs::create_dir_all(dir.join("bad")).unwrap();
    std::fs::write(dir.join("bad/presentation.ron"), b"not ron").unwrap();
    std::fs::write(dir.join("afile"), b"x").unwrap();
    let out = headless("sample\nunpack full\nopen bad\nunpack afile\n", &dir);
    let replies: Vec<&str> = out.lines().collect();
    assert_eq!(replies.len(), 4, "{out}");
    assert!(replies[1..].iter().all(|r| r.starts_with("err ")), "{out}");
    assert!(!replies[3].contains(".rootpoint"), "{out}");
    assert!(!dir.join("full/.rootpoint").exists());
    assert!(!dir.join("bad/.rootpoint").exists());
}
