//! The headless binary reads commands from stdin and writes one reply per command to stdout.

mod common;

use common::{headless, headless_bytes, scratch};
use rootpoint_model::FORMAT_VERSION;

#[test]
fn a_script_gets_one_reply_per_command() {
    let dir = scratch("one-reply");
    let out = headless(
        "# a presentation of one split slide\nadd-slide\n\nsplit n2 beside 0.5\nnonsense\ntree\n",
        &dir,
    );
    assert_eq!(
        out,
        "ok s1 root=n2\nok new=n3 split=n4\nerr unknown command `nonsense`\nok s1 n4:beside(0.5)[n2 text \"\" style=\"Body\" | n3 text \"\" style=\"Body\"]\n"
    );
}

#[test]
fn a_saved_file_is_written_where_the_script_runs() {
    let dir = scratch("save");
    headless("add-slide\nsave deck.ron\n", &dir);
    let text = std::fs::read_to_string(dir.join("deck.ron")).unwrap();
    assert!(text.contains(&format!("version: {FORMAT_VERSION},")));
}

#[test]
fn a_line_that_is_not_utf8_is_refused_and_the_session_goes_on() {
    let dir = scratch("not-utf8");
    let out = headless_bytes(b"add-slide\n\xff\xfe\ntree\n", &dir);
    assert_eq!(
        out,
        "ok s1 root=n2\nerr the line is not UTF-8\nok s1 n2 text \"\" style=\"Body\"\n"
    );
}

#[test]
fn control_and_release_reply_ok_headless_and_change_nothing() {
    let dir = scratch("control-release");
    let with = headless(
        "control\nadd-slide\nbegin\nsplit n2 beside 0.5\nrelease\nend\nrelease\ncontrol\ntree\n",
        &dir,
    );
    assert_eq!(
        with,
        "ok\nok s1 root=n2\nok\nok new=n3 split=n4\nerr a group is open\nok\nok\nok\nok s1 n4:beside(0.5)[n2 text \"\" style=\"Body\" | n3 text \"\" style=\"Body\"]\n"
    );
    let without = headless("add-slide\nbegin\nsplit n2 beside 0.5\nend\ntree\n", &dir);
    assert_eq!(with.lines().last(), without.lines().last());
}
