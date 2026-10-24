//! The window's control socket, per `design@rootpoint-app@control-socket`: a Unix domain socket whose
//! every line goes to the GUI's thread, which runs it on the one session and sends the reply back.
//! The session is touched by the GUI's thread alone; each connection's thread only carries lines.
//! Unix only: the socket is a Unix domain socket, and the browser build has none.

use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use rootpoint_protocol::{Command, Reply};

use crate::Controller;

/// One line received, as bytes, and where its answer goes.
pub(crate) struct Request {
    pub line: Vec<u8>,
    pub reply: Sender<Answer>,
}

/// The answer to one line: its reply, `None` for a line that is no command, and whether the
/// connection ends after it, as it does after the refusal of a take-over, so that the rest of a
/// batch is dropped.
pub(crate) struct Answer {
    pub reply: Option<String>,
    pub close: bool,
}

/// A socket the window listens on. Dropping it ends its accept thread and removes the socket's
/// file, if the file is still this socket's; the connections' threads end with their connections.
pub struct Listener {
    path: PathBuf,
    /// The socket file's inode, so that dropping the listener removes no other window's socket.
    inode: u64,
    requests: Receiver<Request>,
    /// Set when the listener is dropped, so its accept thread stops at its next connection.
    closed: Arc<AtomicBool>,
}

impl Listener {
    /// Listen at `path`, calling `wake` once for each line received, after the line is queued for
    /// the GUI's thread and before its reply, so a window idle on its event loop draws a frame for
    /// it. A socket that accepts a connection is another window's, and is refused; a socket that
    /// refuses one is left by a run that ended, as one killed by a signal leaves it, and is
    /// removed first. Any other file at the path is refused, and left as it is.
    pub fn bind(path: &Path, wake: impl Fn() + Send + Sync + 'static) -> io::Result<Listener> {
        clear_for_bind(path)?;
        let socket = UnixListener::bind(path)?;
        let inode = std::fs::symlink_metadata(path)?.ino();
        let (sender, requests) = mpsc::channel();
        let wake = Arc::new(wake);
        let closed = Arc::new(AtomicBool::new(false));
        let stop = closed.clone();
        thread::spawn(move || {
            for stream in socket.incoming() {
                if stop.load(Ordering::SeqCst) {
                    return;
                }
                let Ok(stream) = stream else { continue };
                let (sender, wake) = (sender.clone(), wake.clone());
                thread::spawn(move || serve(stream, &sender, &*wake));
            }
        });
        Ok(Listener {
            path: path.to_owned(),
            inode,
            requests,
            closed,
        })
    }

    /// Move the socket's file to `to`, by the rules of a bind there: a socket that answers at `to`
    /// is another window's, and refused; one that refuses is removed first; any other file is
    /// refused. The socket keeps its connections, and the file removed on drop is the new one.
    pub fn rename_to(&mut self, to: &Path) -> io::Result<()> {
        clear_for_bind(to)?;
        std::fs::rename(&self.path, to)?;
        self.path = to.to_owned();
        Ok(())
    }

    /// The path of the socket's file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The lines queued when it is called, in the order they came: at most one per connection,
    /// since a connection's thread waits for a line's reply before it reads the next, so a batch
    /// runs one line a frame. Taken at once, not read lazily while the frame runs them, since a
    /// lazy read takes a connection's next line in the same frame when it comes in time.
    pub(crate) fn pending(&self) -> Vec<Request> {
        self.requests.try_iter().collect()
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        // A connection of its own wakes the accept thread, which then sees the flag and ends,
        // closing the listening socket.
        self.closed.store(true, Ordering::SeqCst);
        let _ = UnixStream::connect(&self.path);
        if std::fs::symlink_metadata(&self.path).is_ok_and(|m| m.ino() == self.inode) {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// Make `path` free for a socket: a socket that accepts a connection is another window's, and is
/// refused; a socket that refuses one is left by a run that ended, as one killed by a signal leaves
/// it, and is removed. Any other file at the path is refused, and left as it is.
fn clear_for_bind(path: &Path) -> io::Result<()> {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return Ok(());
    };
    if !meta.file_type().is_socket() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} exists and is no socket", path.display()),
        ));
    }
    match UnixStream::connect(path) {
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AddrInUse,
            format!("another window listens at {}", path.display()),
        )),
        // Only a socket that refuses connections is a stale one; any other error, as a socket the
        // user may not open, leaves the file as it is.
        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => std::fs::remove_file(path),
        Err(e) => Err(e),
    }
}

/// Carry one connection's lines to the GUI's thread, one at a time: a line is sent, then its reply
/// is awaited and written before the next line is read. The connection closes at its end, after
/// an answer that ends it, or when the window is gone.
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
        let Ok(answer) = answer.recv() else { return };
        if let Some(text) = answer.reply
            && writeln!(out, "{text}").is_err()
        {
            return;
        }
        if answer.close {
            return;
        }
    }
}

impl crate::App {
    /// Take the lines of a control socket, which each frame then runs, at its start.
    pub fn listen(&mut self, listener: Listener) {
        self.listener = Some(listener);
    }

    /// The native window's socket: bind it in the runtime directory, created with mode 0700, at
    /// the name of the store, and, in a workspace, write the pointer file that names it,
    /// `design@rootpoint-app@control-socket`.
    /// A socket that cannot be bound leaves the window without one, and the status line says why,
    /// rather than keeping the window from starting.
    pub fn listen_at_store(&mut self, wake: impl Fn() + Send + Sync + 'static) {
        if let Err(e) = self.try_listen_at_store(wake) {
            self.error = Some(format!("the window has no control socket: {e}"));
        }
    }

    fn try_listen_at_store(&mut self, wake: impl Fn() + Send + Sync + 'static) -> io::Result<()> {
        rootpoint_protocol::workspace::make_runtime_dir()?;
        let path = rootpoint_protocol::workspace::socket_path(self.session.root());
        self.listener = Some(Listener::bind(&path, wake)?);
        if let Some(root) = self.session.root() {
            std::fs::write(
                rootpoint_protocol::workspace::pointer_file(Path::new(root)),
                path.as_os_str().as_encoded_bytes(),
            )?;
        }
        Ok(())
    }

    /// After a command changed the store, move the socket's file to the new store's name, write the
    /// new workspace's pointer file and remove the old one's; the status line names a failure, and
    /// the socket then stays where it was.
    pub(crate) fn move_socket(&mut self, old_root: Option<&str>) {
        let Some(listener) = self.listener.as_mut() else {
            return;
        };
        let path = rootpoint_protocol::workspace::socket_path(self.session.root());
        // The old workspace's pointer goes whatever the rename does: its lock is dropped, and its
        // pointer would lead a client to the window that now holds another store.
        if let Some(old) = old_root {
            let _ =
                std::fs::remove_file(rootpoint_protocol::workspace::pointer_file(Path::new(old)));
        }
        if path != listener.path()
            && let Err(e) = listener.rename_to(&path)
        {
            self.error = Some(format!(
                "the control socket stays at {}: {e}",
                listener.path().display()
            ));
            return;
        }
        if let Some(root) = self.session.root()
            && let Err(e) = std::fs::write(
                rootpoint_protocol::workspace::pointer_file(Path::new(root)),
                path.as_os_str().as_encoded_bytes(),
            )
        {
            self.error = Some(format!("the pointer to the control socket: {e}"));
        }
    }

    /// Run every line the control socket received since the last frame, and send each reply.
    pub(crate) fn serve_socket(&mut self) {
        let Some(listener) = &self.listener else {
            return;
        };
        // The listener stays in place while its lines run, so a line that changes the store moves
        // the socket as a command of the window does.
        for request in listener.pending() {
            let answer = match rootpoint_protocol::read_line(&request.line) {
                None => Answer {
                    reply: None,
                    close: false,
                },
                Some(Err(reply)) => Answer {
                    reply: Some(reply.to_string()),
                    close: false,
                },
                Some(Ok(command)) => {
                    // The refusal of a take-over, and it alone, ends its connection, dropping the
                    // rest of a batch; a query's own error, after a take-over, ends nothing.
                    let close = self.controller == Controller::TakenFromSocket
                        && !matches!(
                            command,
                            Command::Tree { .. }
                                | Command::View
                                | Command::Store
                                | Command::Styles
                                | Command::Render { .. }
                                | Command::Control
                                | Command::Release
                        );
                    let reply = self.socket_command(command);
                    Answer {
                        reply: Some(reply.to_string()),
                        close,
                    }
                }
            };
            let _ = request.reply.send(answer);
        }
    }

    /// Run one command of the socket, by the rules of control: the queries run whoever holds
    /// control; `control` takes it, ending the window's gestures; `release` gives it back, as the
    /// session allows; every other command needs it.
    fn socket_command(&mut self, command: Command) -> Reply {
        match command {
            // The queries change nothing, so they leave the status line and the texture alone.
            Command::Tree { .. }
            | Command::View
            | Command::Store
            | Command::Styles
            | Command::Render { .. } => self.session.run(command, &mut *self.files),
            Command::Control => {
                if self.controller != Controller::Socket {
                    // A press begun before completes nothing at its release.
                    self.end_gestures();
                    self.press = None;
                    self.edit_on_click = None;
                    self.text_drag = false;
                    // The dialog of a removal and the node menu close, the dialog removing
                    // nothing, as a gesture ends, `design@rootpoint-app@control-passing`.
                    self.styles_tab.removal = None;
                    self.close_node_menu();
                    self.controller = Controller::Socket;
                }
                Reply::Ok(String::new())
            }
            // A socket that holds no control has none to give back, and no group of its own; a
            // refusal after a take-over lasts until the socket's next `control`.
            Command::Release if self.controller != Controller::Socket => Reply::Ok(String::new()),
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
            command => self.run(command),
        }
    }
}
