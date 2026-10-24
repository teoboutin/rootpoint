//! Sends commands to a running editor's control socket and prints their replies, as
//! `path@rootpoint-protocol@README.md` describes: `rootpoint-ctl <socket> <words>…` sends one
//! line, and `rootpoint-ctl <socket>` sends the lines of its stdin, a batch, over one connection.
//! The socket is given by its path, or by a workspace, whose `.rootpoint/socket` names the socket
//! of the window that holds it; `rootpoint-ctl list` prints the sockets that answer.

use std::io::BufRead;
use std::process::ExitCode;

// Ties this binary's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

#[cfg(unix)]
fn main() -> ExitCode {
    use rootpoint_protocol::control::Connection;

    let mut args = std::env::args().skip(1);
    let Some(target) = args.next() else {
        eprintln!("usage: rootpoint-ctl <workspace-dir | socket | list> [<words>…]");
        return ExitCode::from(2);
    };
    if target == "list" {
        return list();
    }
    // A workspace names its window's socket in its pointer file.
    let socket = if std::path::Path::new(&target).is_dir() {
        let pointer = rootpoint_protocol::workspace::pointer_file(std::path::Path::new(&target));
        match std::fs::read_to_string(&pointer) {
            Ok(path) => path.trim_end_matches('\n').to_owned(),
            Err(e) => {
                eprintln!("{}: {e}: no window holds this workspace", pointer.display());
                return ExitCode::from(2);
            }
        }
    } else {
        target
    };
    let words: Vec<String> = args.collect();
    // Stdin is read as bytes, so a line that is not UTF-8 gets the window's reply to it and the
    // batch goes on, as `rootpoint-headless` reads its own.
    let lines: Box<dyn Iterator<Item = std::io::Result<Vec<u8>>>> = if words.is_empty() {
        Box::new(std::io::stdin().lock().split(b'\n'))
    } else {
        Box::new(std::iter::once(Ok(words.join(" ").into_bytes())))
    };
    let mut connection = match Connection::open(std::path::Path::new(&socket)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{socket}: {e}");
            return ExitCode::from(2);
        }
    };
    // 0 when every reply is `ok`; 1 when one is `err`, also when the connection then ends, as
    // after the refusal of a take-over; 2 when stdin or the connection fails before any `err`.
    let mut refused = false;
    for line in lines {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                eprintln!("stdin: {e}");
                return ExitCode::from(if refused { 1 } else { 2 });
            }
        };
        match connection.send_bytes(&line) {
            Ok(Some(reply)) => {
                println!("{reply}");
                refused |= reply.starts_with("err");
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("{socket}: {e}");
                return ExitCode::from(if refused { 1 } else { 2 });
            }
        }
    }
    ExitCode::from(if refused { 1 } else { 0 })
}

/// Print the file name of each socket of the runtime directory that accepts a connection, one a
/// line, sorted; a socket nothing answers is left by a window that ended, and is not printed.
#[cfg(unix)]
fn list() -> ExitCode {
    let dir = rootpoint_protocol::workspace::runtime_dir();
    let mut names: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| std::os::unix::net::UnixStream::connect(e.path()).is_ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => {
            eprintln!("{}: {e}", dir.display());
            return ExitCode::from(2);
        }
    };
    names.sort();
    for name in names {
        println!("{name}");
    }
    ExitCode::SUCCESS
}

/// The control socket is a Unix domain socket; other platforms have no client.
#[cfg(not(unix))]
fn main() -> ExitCode {
    eprintln!("rootpoint-ctl runs on Unix only");
    ExitCode::from(2)
}
