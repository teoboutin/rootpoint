//! The view commands run headless: selection and presenter mode change the view, and `view`
//! reports it.

mod common;

use common::*;
use rootpoint_protocol::Session;

#[test]
fn selection_and_presenter_mode_run_headless() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    let replies: Vec<String> = run(&mut s, &mut files, &script("view.txt"))
        .iter()
        .map(|r| format!("{r}\n"))
        .collect();
    assert_eq!(replies.concat(), script("view.replies"));
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
    // The undo removed the split, and with it the selected area; its slide stays shown.
    assert_eq!(
        ok(&mut s, &mut files, "view"),
        "selected=- shown=s1 presenting=-"
    );
    run(&mut s, &mut files, "present s1");
    ok(&mut s, &mut files, "redo");
    assert_eq!(
        ok(&mut s, &mut files, "view"),
        "selected=- shown=s1 presenting=s1"
    );
}

#[test]
fn a_removed_slide_stops_presenter_mode() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(&mut s, &mut files, "add-slide\npresent s1\nremove-slide s1");
    assert_eq!(
        ok(&mut s, &mut files, "view"),
        "selected=- shown=- presenting=-"
    );
}
