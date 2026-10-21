//! Runs protocol commands with no window: one command per line from stdin, one reply per command
//! to stdout, until the end of its input. The paths of `save`, `load` and `render` are relative to
//! the working directory; an asset's path, to the directory of the presentation last saved or
//! loaded, or to the working directory before either. A `save` copies beside the saved file every
//! asset whose file lies outside its directory, and every embedded one.

use std::io::{BufRead, Write};

use rootpoint_protocol::{Files, Reply, Session};

// Ties this binary's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

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
        let reply = match std::str::from_utf8(&bytes) {
            Ok(line) => session.execute(line, &mut files),
            Err(_) => Some(Reply::Err("the line is not UTF-8".to_owned())),
        };
        if let Some(reply) = reply
            && writeln!(out, "{reply}").is_err()
        {
            break;
        }
    }
}
