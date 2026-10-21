//! Runs protocol commands with no window: one command per line from stdin, one reply per command
//! to stdout, until the end of its input. Paths are relative to the working directory.

use std::io::{BufRead, Write};

use rootpoint_protocol::{Files, Session};

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
}

fn main() {
    let mut session = Session::new();
    let mut files = NativeFiles;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        if let Some(reply) = session.execute(&line, &mut files)
            && writeln!(out, "{reply}").is_err()
        {
            break;
        }
    }
}
