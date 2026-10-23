//! The view commands run headless: selection, presenter mode and the preview change the view, and
//! `view` reports it.

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
        "selected=- shown=s1 presenting=- preview=off"
    );
    run(&mut s, &mut files, "present s1");
    ok(&mut s, &mut files, "redo");
    assert_eq!(
        ok(&mut s, &mut files, "view"),
        "selected=- shown=s1 presenting=s1 preview=off"
    );
}

#[test]
fn a_removed_slide_stops_presenter_mode() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(&mut s, &mut files, "add-slide\npresent s1\nremove-slide s1");
    assert_eq!(
        ok(&mut s, &mut files, "view"),
        "selected=- shown=- presenting=- preview=off"
    );
}

/// `preview on` deselects, and `select n<id>` ends the preview, so the view never holds a preview
/// and a selection at once; `select s<id>`, `present` and `deselect` leave the preview as it is.
#[test]
fn the_preview_deselects_and_a_selection_ends_it() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(&mut s, &mut files, "sample\nselect n3\npreview on");
    assert_eq!(
        ok(&mut s, &mut files, "view"),
        "selected=- shown=s1 presenting=- preview=on"
    );
    run(&mut s, &mut files, "select s7\ndeselect\npresent\nstop");
    assert_eq!(
        ok(&mut s, &mut files, "view"),
        "selected=- shown=s7 presenting=- preview=on"
    );
    run(&mut s, &mut files, "select n5");
    assert_eq!(
        ok(&mut s, &mut files, "view"),
        "selected=n5 shown=s1 presenting=- preview=off"
    );
    run(&mut s, &mut files, "preview on\npreview off");
    assert_eq!(
        ok(&mut s, &mut files, "view"),
        "selected=- shown=s1 presenting=- preview=off"
    );
    assert!(rootpoint_protocol::parse_command("preview maybe").is_err());
    assert_eq!(
        rootpoint_protocol::parse_command("preview off")
            .unwrap()
            .to_string(),
        "preview off"
    );
}
