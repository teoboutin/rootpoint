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

    /// A directory exists where a file lies below it.
    fn exists(&mut self, path: &str) -> Result<bool, String> {
        let dir = format!("{path}/");
        Ok(self.files.contains_key(path) || self.files.keys().any(|k| k.starts_with(&dir)))
    }

    /// Files in memory have no symlinks: a path is its own canonical form.
    fn canonical(&mut self, path: &str) -> Result<String, String> {
        Ok(path.to_owned())
    }
}

/// The native file system, paths relative to the working directory, which records every path it
/// is given, so that a test can check that none leads out of a root.
#[derive(Default)]
pub struct RecordingFiles {
    pub paths: Vec<String>,
}

impl Files for RecordingFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        self.paths.push(path.to_owned());
        std::fs::read(path).map_err(|e| format!("{path}: {e}"))
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        self.paths.push(path.to_owned());
        if let Some(dir) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{path}: {e}"))?;
        }
        std::fs::write(path, bytes).map_err(|e| format!("{path}: {e}"))
    }

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        self.paths.push(path.to_owned());
        std::fs::exists(path).map_err(|e| format!("{path}: {e}"))
    }

    fn canonical(&mut self, path: &str) -> Result<String, String> {
        self.paths.push(path.to_owned());
        let canonical = std::fs::canonicalize(path).map_err(|e| format!("{path}: {e}"))?;
        Ok(canonical.to_str().unwrap().to_owned())
    }
}

impl RecordingFiles {
    /// The paths given that are not `root` and do not lie below it.
    pub fn outside(&self, root: &std::path::Path) -> Vec<String> {
        let root = root.to_str().unwrap();
        self.paths
            .iter()
            .filter(|p| *p != root && !p.starts_with(&format!("{root}/")))
            .cloned()
            .collect()
    }
}

impl MemoryFiles {
    /// Files at these paths, each holding its own path's bytes, for a test that imports them.
    pub fn holding(paths: &[&str]) -> Self {
        MemoryFiles {
            files: paths
                .iter()
                .map(|p| (p.to_string(), p.as_bytes().to_vec()))
                .collect(),
            read_only: false,
        }
    }
}

/// Write the session's document as `presentation.ron` of the directory `dir`, as `save` would in
/// a workspace there, so that a test edits it by hand and opens it.
pub fn write_presentation(session: &Session, files: &mut MemoryFiles, dir: &str) {
    files.files.insert(
        format!("{dir}/presentation.ron"),
        rootpoint_model::save_ron(session.document()).into_bytes(),
    );
}

/// A 40×20 PNG: its left half red, its right half blue, as the renderer's fixture image.
pub fn two_colour_png() -> Vec<u8> {
    let img = image::RgbaImage::from_fn(40, 20, |x, _| {
        if x < 20 {
            image::Rgba([220, 30, 30, 255])
        } else {
            image::Rgba([30, 30, 220, 255])
        }
    });
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    bytes
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
