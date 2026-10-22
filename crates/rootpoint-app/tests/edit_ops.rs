//! The editing operations at the places where Parley's visual moves and the text disagree: a soft
//! line break, where a visual move only changes the caret's affinity, and a composing text after
//! an edit, whose byte range moves with it. Each was a defect found by the review of the slice that
//! edits text in place, `spec@first-prototype@text-editing`.

use eframe::egui::{Pos2, Rect, Vec2};
use rootpoint_app::App;
use rootpoint_app::editing::CaretMove;
use rootpoint_app::hit::Layout;
use rootpoint_model::{Content, Node, NodeId, SlideId};
use rootpoint_protocol::{Command, Files, Reply, parse_command};
use rootpoint_render::{FontSet, area_text};

struct NoFiles;

impl Files for NoFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        Err(format!("{path}: no file"))
    }

    fn write(&mut self, path: &str, _: &[u8]) -> Result<(), String> {
        Err(format!("{path}: no file"))
    }

    fn exists(&mut self, _: &str) -> Result<bool, String> {
        Ok(false)
    }
}

fn text_of(app: &App, area: NodeId) -> String {
    match app.session().document().node(area) {
        Some(Node::Area(a)) => match &a.content {
            Content::Text(t) => t.text.clone(),
            _ => panic!("{area} holds no text"),
        },
        _ => panic!("{area} is no area"),
    }
}

/// The sample, with n3, a text of several wrapped lines, edited in place.
fn editing_n3() -> App {
    let mut app = App::new(Box::new(NoFiles));
    assert!(matches!(app.run(Command::Sample), Reply::Ok(_)));
    app.run(parse_command("select n3").unwrap());
    let doc = app.session().document();
    let view = Rect::from_min_size(Pos2::ZERO, Vec2::new(960.0, 540.0));
    let layout = Layout::new(doc, SlideId(1), view, [960, 540]).unwrap();
    let p = layout.rect(NodeId(3)).unwrap().center();
    app.edit_at(NodeId(3), p, &layout);
    assert_eq!(app.editing(), Some(NodeId(3)));
    app
}

/// The byte offset where n3's second line starts, after a soft break.
fn first_soft_break(app: &App) -> usize {
    let doc = app.session().document();
    let text = area_text(doc, SlideId(1), NodeId(3), &mut FontSet::new()).unwrap();
    let start = text
        .layout
        .lines()
        .nth(1)
        .expect("n3 wraps")
        .text_range()
        .start;
    assert!(start > 0);
    start
}

#[test]
fn backspace_at_the_start_of_a_wrapped_line_deletes_the_character_before() {
    let mut app = editing_n3();
    let before = text_of(&app, NodeId(3));
    let b = first_soft_break(&app);
    app.set_caret(b);
    app.delete_backward();
    let mut want = before.clone();
    want.remove(b - 1);
    assert_eq!(text_of(&app, NodeId(3)), want);
    assert_eq!(app.caret_index(), Some(b - 1));
}

#[test]
fn delete_at_the_end_of_a_wrapped_line_deletes_the_character_after() {
    let mut app = editing_n3();
    let before = text_of(&app, NodeId(3));
    app.set_caret(0);
    app.move_caret(CaretMove::LineEnd, false);
    let i = app.caret_index().unwrap();
    app.delete_forward();
    let mut want = before.clone();
    want.remove(i);
    assert_eq!(text_of(&app, NodeId(3)), want);
    // And Backspace at the end of a line deletes the character before it.
    app.move_caret(CaretMove::LineEnd, false);
    let j = app.caret_index().unwrap();
    let now = text_of(&app, NodeId(3));
    app.delete_backward();
    assert_eq!(text_of(&app, NodeId(3)).len() + 1, now.len());
    assert_eq!(app.caret_index(), Some(j - 1));
}

#[test]
fn a_composing_text_after_an_edit_moves_with_it() {
    let mut app = editing_n3();
    let before = text_of(&app, NodeId(3));
    app.set_caret(0);
    app.set_preedit("あ");
    assert_eq!(app.preedit_range(), Some(0.."あ".len()));
    app.move_caret(CaretMove::TextStart, false);
    app.type_str("X");
    assert_eq!(text_of(&app, NodeId(3)), format!("Xあ{before}"));
    assert_eq!(app.preedit_range(), Some(1..1 + "あ".len()));
    // Its commit then replaces the composing text, not the X.
    app.commit_ime("亜");
    assert_eq!(text_of(&app, NodeId(3)), format!("X亜{before}"));
}
