//! Sends commands to a running editor's control socket and prints their replies, as the crate's
//! `README.md` describes: `rootpoint-ctl <socket> <words>…` sends one line, and
//! `rootpoint-ctl <socket>` sends the lines of its stdin, a batch, over one connection.

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
    let lines: Box<dyn Iterator<Item = String>> = if words.is_empty() {
        Box::new(std::io::stdin().lock().lines().map_while(Result::ok))
    } else {
        Box::new(std::iter::once(words.join(" ")))
    };
    let mut connection = match Connection::open(std::path::Path::new(&socket)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{socket}: {e}");
            return ExitCode::from(2);
        }
    };
    // 0 when every reply is `ok`, 1 when one is `err`, 2 when the connection fails before one.
    let mut refused = false;
    for line in lines {
        match connection.send(&line) {
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
