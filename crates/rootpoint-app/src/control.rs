//! The window's control socket, per `spec@first-prototype@live-control`: a Unix domain socket whose
//! every line goes to the GUI's thread, which runs it on the one session and sends the reply back.
//! The session is touched by the GUI's thread alone; each connection's thread only carries lines.
//! Native only: the browser build has no socket.

use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// One line received, as bytes, and where its reply goes: `None` for a line that is no command.
pub(crate) struct Request {
    pub line: Vec<u8>,
    pub reply: Sender<Option<String>>,
}

/// A socket the window listens on. Dropping it removes the socket's file; its threads end when
/// their connections do.
pub struct Listener {
    path: PathBuf,
    requests: Receiver<Request>,
}

impl Listener {
    /// Listen at `path`, calling `wake` once for each line received, before the GUI's thread reads
    /// it, so a window idle on its event loop draws a frame for it. A path that accepts a
    /// connection is another window's, and is refused; a path that refuses one is a socket left by
    /// a run that ended, and is removed first.
    pub fn bind(path: &Path, wake: impl Fn() + Send + Sync + 'static) -> io::Result<Listener> {
        if path.exists() {
            if UnixStream::connect(path).is_ok() {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    format!("another window listens at {}", path.display()),
                ));
            }
            std::fs::remove_file(path)?;
        }
        let socket = UnixListener::bind(path)?;
        let (sender, requests) = mpsc::channel();
        let wake = std::sync::Arc::new(wake);
        thread::spawn(move || {
            for stream in socket.incoming() {
                let Ok(stream) = stream else { continue };
                let (sender, wake) = (sender.clone(), wake.clone());
                thread::spawn(move || serve(stream, &sender, &*wake));
            }
        });
        Ok(Listener {
            path: path.to_owned(),
            requests,
        })
    }

    /// The lines received since the last call, in the order they came.
    pub(crate) fn pending(&self) -> impl Iterator<Item = Request> + '_ {
        self.requests.try_iter()
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Carry one connection's lines to the GUI's thread, one at a time: a line is sent, then its reply
/// is awaited and written before the next line is read. The connection closes at its end, or
/// when the window is gone.
fn serve(stream: UnixStream, sender: &Sender<Request>, wake: &(dyn Fn() + Send + Sync)) {
    let Ok(mut out) = stream.try_clone() else {
        return;
    };
    for line in BufReader::new(stream).split(b'\n') {
        let Ok(line) = line else { return };
        let (reply, answer) = mpsc::channel();
        if sender.send(Request { line, reply }).is_err() {
            return;
        }
        wake();
        match answer.recv() {
            Ok(Some(text)) => {
                if writeln!(out, "{text}").is_err() {
                    return;
                }
            }
            Ok(None) => {}
            Err(_) => return,
        }
    }
}
