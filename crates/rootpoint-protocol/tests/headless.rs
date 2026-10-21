//! The headless binary reads commands from stdin and writes one reply per command to stdout.

use std::io::Write;
use std::process::{Command, Stdio};

/// Pipe a script into `rootpoint-headless`, run in `dir`, and return its stdout.
pub fn headless(script: &str, dir: &std::path::Path) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rootpoint-headless"))
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(script.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()
}

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_script_gets_one_reply_per_command() {
    let dir = scratch("one-reply");
    let out = headless(
        "# a presentation of one split slide\nadd-slide\n\nsplit n2 beside 0.5\nnonsense\ntree\n",
        &dir,
    );
    assert_eq!(
        out,
        "ok s1 root=n2\nok new=n3 split=n4\nerr unknown command `nonsense`\nok s1 n4:beside(0.5)[n2 empty | n3 empty]\n"
    );
}

#[test]
fn a_saved_file_is_written_where_the_script_runs() {
    let dir = scratch("save");
    headless("add-slide\nsave deck.ron\n", &dir);
    let text = std::fs::read_to_string(dir.join("deck.ron")).unwrap();
    assert!(text.contains("version: 1"));
}
