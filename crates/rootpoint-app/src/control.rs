//! The window's control socket, per `spec@first-prototype@live-control`: a Unix domain socket whose
//! every line goes to the GUI's thread, which runs it on the one session and sends the reply back.
//! The session is touched by the GUI's thread alone; each connection's thread only carries lines.
//! Native only: the browser build has no socket.

use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use rootpoint_protocol::{Command, Reply};

use crate::Controller;

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

impl crate::App {
    /// Take the lines of a control socket, which each frame then runs, at its start.
    pub fn listen(&mut self, listener: Listener) {
        self.listener = Some(listener);
    }

    /// Run every line the control socket received since the last frame, and send each reply.
    pub(crate) fn serve_socket(&mut self) {
        let Some(listener) = self.listener.take() else {
            return;
        };
        for request in listener.pending() {
            let reply = match rootpoint_protocol::read_line(&request.line) {
                None => None,
                Some(Err(reply)) => Some(reply.to_string()),
                Some(Ok(command)) => Some(self.socket_command(command).to_string()),
            };
            let _ = request.reply.send(reply);
        }
        self.listener = Some(listener);
    }

    /// Run one command of the socket, by the rules of control: the queries run whoever holds
    /// control; `control` takes it, ending the window's gestures; `release` gives it back, as the
    /// session allows; every other command needs it.
    fn socket_command(&mut self, command: Command) -> Reply {
        match command {
            // The queries change nothing, so they leave the status line and the texture alone.
            Command::Tree { .. } | Command::View | Command::Render { .. } => {
                self.session.run(command, &mut *self.files)
            }
            Command::Control => {
                if self.controller != Controller::Socket {
                    // A press begun before completes nothing at its release.
                    self.end_gestures();
                    self.press = None;
                    self.edit_on_click = None;
                    self.text_drag = false;
                    self.controller = Controller::Socket;
                }
                Reply::Ok(String::new())
            }
            Command::Release => {
                let reply = self.session.run(Command::Release, &mut *self.files);
                if matches!(reply, Reply::Ok(_)) {
                    self.controller = Controller::Window;
                }
                reply
            }
            _ if self.controller == Controller::Window => {
                Reply::Err("the window has control".to_owned())
            }
            _ if self.controller == Controller::TakenFromSocket => Reply::Err(
                "the author took control: stop, and wait for the author's instructions".to_owned(),
            ),
            // The file Save writes follows the socket's commands as it follows the window's.
            Command::Load { path } => {
                let path = self.relative_if_absolute(path);
                let reply = self.run(Command::Load { path: path.clone() });
                if matches!(reply, Reply::Ok(_)) {
                    self.path = Some(path);
                }
                reply
            }
            Command::Save { path } => {
                let path = self.relative_if_absolute(path);
                let reply = self.run(Command::Save { path: path.clone() });
                if matches!(reply, Reply::Ok(_)) {
                    self.path = Some(path);
                }
                reply
            }
            Command::New | Command::Sample => {
                let reply = self.run(command);
                if matches!(reply, Reply::Ok(_)) {
                    self.path = None;
                }
                reply
            }
            command => self.run(command),
        }
    }
}
