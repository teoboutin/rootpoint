//! The client side of a running editor's control socket: one line sent, one reply read. Unix only,
//! since the socket is a Unix domain socket and the browser build has none.

use std::io::{self, BufRead, BufReader, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::Path;

/// Send one command line to the socket at `path` and read its reply: `Ok(Some(reply))`, or
/// `Ok(None)` when the listener closed the connection with no reply, as it does for a blank line
/// or a comment; `Err` when the socket cannot be reached. The client shuts down its side after the
/// line, so the listener sees the end of the connection after it.
pub fn send(path: &Path, line: &str) -> io::Result<Option<String>> {
    let mut stream = UnixStream::connect(path)?;
    stream.write_all(line.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.shutdown(Shutdown::Write)?;
    let mut reply = String::new();
    if BufReader::new(stream).read_line(&mut reply)? == 0 {
        return Ok(None);
    }
    Ok(Some(reply.trim_end_matches('\n').to_owned()))
}
