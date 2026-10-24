//! Runs protocol commands with no window: one command per line from stdin, one reply per command
//! to stdout, until the end of its input. A host path, the argument of `open`, `open-copy`,
//! `open-pack`, `pack`, `unpack`, `import` and `render`, is relative to the working directory; an
//! asset's path, to the root of the workspace open, or a key of the map of a presentation held in
//! memory. A workspace it opens is locked to it until the store changes or the input ends, so a
//! window or another script cannot open it meanwhile. It has no autosave: a script writes a
//! workspace's `presentation.ron`, or the pack of a presentation held in memory, by `save`.

use std::io::{BufRead, Write};

use rootpoint_protocol::{Files, Session, read_line, workspace};

// Ties this binary's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

/// The native file system, paths relative to the working directory.
struct NativeFiles;

impl Files for NativeFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|e| format!("{path}: {e}"))
    }

    /// Writes the file, creating the directories above it, as an import at `img/cat.png` needs.
    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        if let Some(dir) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{path}: {e}"))?;
        }
        std::fs::write(path, bytes).map_err(|e| format!("{path}: {e}"))
    }

    /// Anything at the path, a symlink whose target is missing included, as `Files::exists` asks.
    fn exists(&mut self, path: &str) -> Result<bool, String> {
        match std::fs::symlink_metadata(path) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(format!("{path}: {e}")),
        }
    }

    fn canonical(&mut self, path: &str) -> Result<String, String> {
        let canonical = std::fs::canonicalize(path).map_err(|e| format!("{path}: {e}"))?;
        canonical
            .into_os_string()
            .into_string()
            .map_err(|p| format!("{} is not UTF-8", p.display()))
    }
    fn list(&mut self, dir: &str) -> Result<Vec<String>, String> {
        /// The files below `dir`, each as `below` and its path under `dir`; a symlink is listed as
        /// a file, never followed.
        fn walk(dir: &std::path::Path, below: &str, out: &mut Vec<String>) -> std::io::Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let name = entry.file_name().into_string().map_err(|name| {
                    std::io::Error::other(format!("{} is not UTF-8", name.display()))
                })?;
                let path = if below.is_empty() {
                    name
                } else {
                    format!("{below}/{name}")
                };
                if entry.file_type()?.is_dir() {
                    walk(&entry.path(), &path, out)?;
                } else {
                    out.push(path);
                }
            }
            Ok(())
        }
        let mut out = Vec::new();
        match walk(std::path::Path::new(dir), "", &mut out) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(format!("{dir}: {e}")),
        }
        out.sort();
        Ok(out)
    }

    /// Renames the file, creating the directories above its new path, as a write does.
    fn rename(&mut self, from: &str, to: &str) -> Result<(), String> {
        if let Some(dir) = std::path::Path::new(to).parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{to}: {e}"))?;
        }
        std::fs::rename(from, to).map_err(|e| format!("{from}: {e}"))
    }

    fn remove(&mut self, path: &str) -> Result<(), String> {
        std::fs::remove_file(path).map_err(|e| format!("{path}: {e}"))
    }
}

fn main() {
    let mut session = Session::new();
    let mut files = NativeFiles;
    // The lock of the workspace open, which keeps it to this process, `workspace::run`.
    let mut lock = None;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    // Lines are read as bytes, so that a line that is not UTF-8 gets its own reply and the
    // session goes on.
    for line in std::io::stdin().lock().split(b'\n') {
        let Ok(bytes) = line else { break };
        let reply = read_line(&bytes).map(|command| match command {
            Ok(command) => workspace::run(&mut session, command, &mut files, &mut lock),
            Err(reply) => reply,
        });
        if let Some(reply) = reply
            && writeln!(out, "{reply}").is_err()
        {
            break;
        }
    }
}
