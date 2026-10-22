//! Helpers of the protocol tests: files held in memory, and a session that runs a script.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::io::Write;
use std::process::{Command, Stdio};

use rootpoint_protocol::{Files, Reply, Session};

/// Files held in memory, by path. While `read_only` is set, every write fails.
#[derive(Default)]
pub struct MemoryFiles {
    pub files: BTreeMap<String, Vec<u8>>,
    pub read_only: bool,
}

impl Files for MemoryFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        self.files
            .get(path)
            .cloned()
            .ok_or(format!("{path}: no such file"))
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        if self.read_only {
            return Err(format!("{path}: read-only"));
        }
        self.files.insert(path.to_owned(), bytes.to_vec());
        Ok(())
    }

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        Ok(self.files.contains_key(path))
    }
}

/// Run each line of a script, and return the replies, one per command.
pub fn run(session: &mut Session, files: &mut MemoryFiles, script: &str) -> Vec<Reply> {
    script
        .lines()
        .filter_map(|line| session.execute(line, files))
        .collect()
}

/// Run one line, which must succeed, and return its reply's text.
pub fn ok(session: &mut Session, files: &mut MemoryFiles, line: &str) -> String {
    match session.execute(line, files) {
        Some(Reply::Ok(text)) => text,
        other => panic!("`{line}` gave {other:?}"),
    }
}

/// A script of `path@rootpoint-protocol@tests/scripts/`, which the app's tests also run through
/// the editor's control socket.
pub fn script(name: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/scripts")
            .join(name),
    )
    .unwrap()
}

/// Pipe a script into `rootpoint-headless`, run in `dir`, and return its stdout.
pub fn headless(script: &str, dir: &std::path::Path) -> String {
    headless_bytes(script.as_bytes(), dir)
}

/// Pipe bytes, which need not be UTF-8, into `rootpoint-headless`, run in `dir`.
pub fn headless_bytes(script: &[u8], dir: &std::path::Path) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rootpoint-headless"))
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(script).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()
}

pub fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A directory for a socket, made empty: under the build's scratch directory, or under `/tmp` when that path
/// would make a socket path longer than a Unix socket's address holds, 108 bytes.
pub fn socket_dir(name: &str) -> std::path::PathBuf {
    let base = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let dir = if base.as_os_str().len() + name.len() + 10 < 100 {
        base.join(name)
    } else {
        std::path::PathBuf::from("/tmp").join(format!("rootpoint-{}-{name}", std::process::id()))
    };
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
