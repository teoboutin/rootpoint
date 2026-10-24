//! Runs protocol commands with no window: one command per line from stdin, one reply per command
//! to stdout, until the end of its input. A host path, the argument of `open`, `open-copy`,
//! `import` and `render`, is relative to the working directory; an asset's path, to the root of the
//! workspace `open` opened, or a key of the map of a presentation held in memory. It has no
//! autosave: a script writes `presentation.ron` by `save`.

use std::io::{BufRead, Write};

use rootpoint_protocol::{Files, Session, read_line};

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

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        std::fs::exists(path).map_err(|e| format!("{path}: {e}"))
    }

    fn canonical(&mut self, path: &str) -> Result<String, String> {
        let canonical = std::fs::canonicalize(path).map_err(|e| format!("{path}: {e}"))?;
        canonical
            .into_os_string()
            .into_string()
            .map_err(|p| format!("{} is not UTF-8", p.display()))
    }
}

fn main() {
    let mut session = Session::new();
    let mut files = NativeFiles;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    // Lines are read as bytes, so that a line that is not UTF-8 gets its own reply and the
    // session goes on.
    for line in std::io::stdin().lock().split(b'\n') {
        let Ok(bytes) = line else { break };
        let reply = read_line(&bytes).map(|command| match command {
            Ok(command) => session.run(command, &mut files),
            Err(reply) => reply,
        });
        if let Some(reply) = reply
            && writeln!(out, "{reply}").is_err()
        {
            break;
        }
    }
}
