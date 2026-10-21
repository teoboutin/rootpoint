//! The view commands run headless: selection and presenter mode change the view, and `view`
//! reports it.

mod common;

use common::*;
use rootpoint_protocol::{Reply, Session};

#[test]
fn selection_and_presenter_mode_run_headless() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nadd-slide\nsplit n2 beside 0.5",
    );
    assert_eq!(ok(&mut s, &mut files, "view"), "selected=- presenting=-");
    ok(&mut s, &mut files, "select n5");
    assert_eq!(ok(&mut s, &mut files, "view"), "selected=n5 presenting=-");
    assert_eq!(ok(&mut s, &mut files, "present"), "s1");
    assert_eq!(ok(&mut s, &mut files, "next"), "s3");
    assert_eq!(ok(&mut s, &mut files, "view"), "selected=n5 presenting=s3");
    assert!(
        matches!(s.execute("next", &mut files), Some(Reply::Err(_))),
        "at the last slide"
    );
    assert_eq!(ok(&mut s, &mut files, "prev"), "s1");
    assert!(
        matches!(s.execute("prev", &mut files), Some(Reply::Err(_))),
        "at the first slide"
    );
    ok(&mut s, &mut files, "stop");
    ok(&mut s, &mut files, "deselect");
    assert_eq!(ok(&mut s, &mut files, "view"), "selected=- presenting=-");
}

#[test]
fn view_commands_are_not_undone() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nsplit n2 beside 0.5\nselect n3",
    );
    ok(&mut s, &mut files, "undo");
    // The undo removed the split, and with it the selected area.
    assert_eq!(ok(&mut s, &mut files, "view"), "selected=- presenting=-");
    run(&mut s, &mut files, "present s1");
    ok(&mut s, &mut files, "redo");
    assert_eq!(ok(&mut s, &mut files, "view"), "selected=- presenting=s1");
}

#[test]
fn a_removed_slide_stops_presenter_mode() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(&mut s, &mut files, "add-slide\npresent s1\nremove-slide s1");
    assert_eq!(ok(&mut s, &mut files, "view"), "selected=- presenting=-");
}
