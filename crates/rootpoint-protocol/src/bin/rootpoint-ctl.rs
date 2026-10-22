//! Sends one command to a running editor's control socket and prints its reply:
//! `rootpoint-ctl <socket> <words>…`. The words after the socket's path are joined with single
//! spaces into one line, so a line that holds a quoted text is given as one word:
//! `rootpoint-ctl /tmp/rp 'text n3 "two words"'`. The paths of `save`, `load` and `render` are
//! relative to the editor's working directory, not to this one's; an asset's path, to the
//! directory of the presentation last saved or loaded. Exits 0 on an `ok` reply and on a line that
//! gets none, a blank line or a comment; 1 on an `err` reply; 2 when the socket cannot be reached
//! or no command is given.

use std::process::ExitCode;

// Ties this binary's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

#[cfg(unix)]
fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (Some(socket), words) = (args.next(), args.collect::<Vec<_>>()) else {
        eprintln!("usage: rootpoint-ctl <socket> <words>…");
        return ExitCode::from(2);
    };
    if words.is_empty() {
        eprintln!("usage: rootpoint-ctl <socket> <words>…");
        return ExitCode::from(2);
    }
    match rootpoint_protocol::control::send(std::path::Path::new(&socket), &words.join(" ")) {
        Ok(Some(reply)) => {
            println!("{reply}");
            if reply.starts_with("err") {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Ok(None) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{socket}: {e}");
            ExitCode::from(2)
        }
    }
}

/// The control socket is a Unix domain socket; other platforms have no client.
#[cfg(not(unix))]
fn main() -> ExitCode {
    eprintln!("rootpoint-ctl runs on Unix only");
    ExitCode::from(2)
}
