//! `rootpoint-ctl` sends one line from its arguments, or a batch from its stdin, to a control
//! socket, prints each reply, and exits by them, per
//! `design@rootpoint-protocol@ctl-sends-a-line-or-a-batch`. The listeners here are the tests' own,
//! which answer each line with a reply fixed by the line, so the binary is judged alone, with no
//! window.

mod common;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::process::Command;
use std::thread;

use common::socket_dir;

/// Run `rootpoint-ctl` on `args`, and return its exit code and its stdout.
fn ctl(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_rootpoint-ctl"))
        .args(args)
        .output()
        .unwrap();
    (
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap(),
    )
}

/// A listener that reads one line per connection and answers `ok <line>` to a line that starts
/// with `good`, `err <line>` to one that starts with `bad`, and nothing to any other, then closes.
fn listener(name: &str) -> std::path::PathBuf {
    let path = socket_dir(name).join("rp.sock");
    let socket = UnixListener::bind(&path).unwrap();
    thread::spawn(move || {
        for stream in socket.incoming() {
            let mut stream = stream.unwrap();
            let mut line = String::new();
            BufReader::new(&stream).read_line(&mut line).unwrap();
            let line = line.trim_end();
            if line.starts_with("good") {
                writeln!(stream, "ok {line}").unwrap();
            } else if line.starts_with("bad") {
                writeln!(stream, "err {line}").unwrap();
            }
        }
    });
    path
}

#[test]
fn it_prints_the_reply_and_exits_by_it() {
    let path = listener("ctl-replies");
    let socket = path.to_str().unwrap();
    assert_eq!(ctl(&[socket, "good", "n3"]), (0, "ok good n3\n".to_owned()));
    assert_eq!(ctl(&[socket, "bad"]), (1, "err bad\n".to_owned()));
    assert_eq!(ctl(&[socket, "# a comment"]), (0, String::new()));
}

#[test]
fn it_exits_2_when_the_socket_cannot_be_reached() {
    let path = socket_dir("ctl-none").join("rp.sock");
    assert_eq!(ctl(&[path.to_str().unwrap(), "tree"]).0, 2);
    assert_eq!(ctl(&[path.to_str().unwrap()]).0, 2);
}

/// `rootpoint-ctl <socket>` sends the lines of its stdin over one
/// connection and prints each reply; when the listener ends the connection after a refusal, the
/// batch stops there, its later lines unsent, and the exit code is 1.
#[test]
fn a_batch_on_stdin_goes_over_one_connection_and_stops_at_its_end() {
    use std::io::Write as _;
    use std::sync::{Arc, Mutex};
    let path = socket_dir("ctl-batch").join("rp.sock");
    let socket = UnixListener::bind(&path).unwrap();
    let (received, seen) = (Arc::new(Mutex::new(Vec::new())), Arc::new(Mutex::new(0)));
    let (lines, connections) = (received.clone(), seen.clone());
    thread::spawn(move || {
        for stream in socket.incoming() {
            *connections.lock().unwrap() += 1;
            let mut stream = stream.unwrap();
            for line in BufReader::new(stream.try_clone().unwrap()).lines() {
                let line = line.unwrap();
                lines.lock().unwrap().push(line.clone());
                if line == "stop" {
                    writeln!(stream, "err the author took control").unwrap();
                    break;
                }
                if !line.starts_with('#') {
                    writeln!(stream, "ok {line}").unwrap();
                }
            }
        }
    });
    let mut child = Command::new(env!("CARGO_BIN_EXE_rootpoint-ctl"))
        .arg(&path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"one\n# a comment\ntwo\nstop\nthree\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "ok one\nok two\nerr the author took control\n"
    );
    assert_eq!(*seen.lock().unwrap(), 1, "one connection");
    assert!(!received.lock().unwrap().contains(&"three".to_owned()));
}

/// A line of the batch that is not UTF-8 is sent as it is, and gets its reply; the batch goes on.
/// A line that holds a newline is refused by the client before anything is sent.
#[test]
fn a_line_that_is_not_utf8_goes_on_and_a_line_with_a_newline_is_refused() {
    use std::io::Write as _;
    use std::sync::{Arc, Mutex};
    let path = socket_dir("ctl-bytes").join("rp.sock");
    let socket = UnixListener::bind(&path).unwrap();
    let received = Arc::new(Mutex::new(Vec::new()));
    let lines = received.clone();
    thread::spawn(move || {
        for stream in socket.incoming() {
            let mut stream = stream.unwrap();
            for line in BufReader::new(stream.try_clone().unwrap()).split(b'\n') {
                let line = line.unwrap();
                let text = String::from_utf8_lossy(&line).into_owned();
                lines.lock().unwrap().push(text.clone());
                let reply = match String::from_utf8(line) {
                    Ok(_) => format!("ok {text}"),
                    Err(_) => "err the line is not UTF-8".to_owned(),
                };
                writeln!(stream, "{reply}").unwrap();
            }
        }
    });
    let mut child = Command::new(env!("CARGO_BIN_EXE_rootpoint-ctl"))
        .arg(&path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"one\n\xff\ntwo\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "ok one\nerr the line is not UTF-8\nok two\n"
    );
    let mut connection = rootpoint_protocol::control::Connection::open(&path).unwrap();
    assert_eq!(
        connection.send("# note\nadd-slide").err().map(|e| e.kind()),
        Some(std::io::ErrorKind::InvalidInput)
    );
    assert!(
        !received
            .lock()
            .unwrap()
            .iter()
            .any(|l| l.contains("add-slide"))
    );
}
