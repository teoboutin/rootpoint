//! The client side of a running editor's control socket: lines sent, one reply read for each
//! command. Unix only, since the socket is a Unix domain socket and the browser build has none.

use std::io::{self, BufRead, BufReader, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::Path;

use crate::read_line;

/// One connection to a control socket, which carries any number of lines, a batch, one at a time.
pub struct Connection {
    out: UnixStream,
    replies: BufReader<UnixStream>,
}

impl Connection {
    /// Connect to the socket at `path`; `Err` when it cannot be reached.
    pub fn open(path: &Path) -> io::Result<Connection> {
        let out = UnixStream::connect(path)?;
        let replies = BufReader::new(out.try_clone()?);
        Ok(Connection { out, replies })
    }

    /// Send one line and read its reply: `Ok(None)` for a blank line or a comment, which gets
    /// none, by the rule of `read_line`. A command whose connection ends with no reply is an
    /// error, as when the window was closed, or ended the connection after the refusal of a
    /// take-over, so a batch stops there.
    pub fn send(&mut self, line: &str) -> io::Result<Option<String>> {
        self.send_bytes(line.as_bytes())
    }

    /// `send` for a line given as bytes, which need not be UTF-8: the window replies to such a
    /// line with the `err` reply `read_line` gives it. A line holding a newline is refused, since
    /// the window would read two lines where the client waits for one reply.
    pub fn send_bytes(&mut self, line: &[u8]) -> io::Result<Option<String>> {
        if line.contains(&b'\n') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "a line holds a newline",
            ));
        }
        self.out.write_all(line)?;
        self.out.write_all(b"\n")?;
        if read_line(line).is_none() {
            return Ok(None);
        }
        let mut reply = String::new();
        if self.replies.read_line(&mut reply)? == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "the window closed the connection with no reply",
            ));
        }
        Ok(Some(reply.trim_end_matches('\n').to_owned()))
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.out.shutdown(Shutdown::Both);
    }
}

/// Send one line to the socket at `path` over a connection of its own, and read its reply, as
/// `Connection::send` does.
pub fn send(path: &Path, line: &str) -> io::Result<Option<String>> {
    Connection::open(path)?.send(line)
}
