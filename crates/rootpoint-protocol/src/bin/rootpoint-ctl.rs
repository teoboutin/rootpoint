//! Sends commands to a running editor's control socket and prints their replies, as
//! `path@rootpoint-protocol@README.md` describes: `rootpoint-ctl <socket> <words>…` sends one
//! line, and `rootpoint-ctl <socket>` sends the lines of its stdin, a batch, over one connection.

use std::io::BufRead;
use std::process::ExitCode;

// Ties this binary's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

#[cfg(unix)]
fn main() -> ExitCode {
    use rootpoint_protocol::control::Connection;

    let mut args = std::env::args().skip(1);
    let Some(socket) = args.next() else {
        eprintln!("usage: rootpoint-ctl <socket> [<words>…]");
        return ExitCode::from(2);
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

/// The control socket is a Unix domain socket; other platforms have no client.
#[cfg(not(unix))]
fn main() -> ExitCode {
    eprintln!("rootpoint-ctl runs on Unix only");
    ExitCode::from(2)
}
