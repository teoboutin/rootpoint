//! The slide list's actions and a node's path from the slide's root, with no window.

use rootpoint_app::App;
use rootpoint_app::selection::path;
use rootpoint_model::{Node, SlideId};
use rootpoint_protocol::{Command, Files, Reply};

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

fn sample() -> App {
    let mut app = App::new(Box::new(NoFiles));
    assert!(matches!(app.run(Command::Sample), Reply::Ok(_)));
    app
}

fn slides(app: &App) -> Vec<SlideId> {
    app.session()
        .document()
        .slides
        .iter()
        .map(|s| s.id)
        .collect()
}

#[test]
fn a_slide_is_added_after_the_shown_one_and_the_shown_one_is_removed_or_duplicated() {
    let mut app = sample();
    let [a, b, c, d, e, f] = slides(&app)[..] else {
        panic!("the sample has six slides");
    };
    // With none shown, the first is.
    assert_eq!(app.shown_slide(), Some(a));
    app.show_slide(b);
    app.add_slide();
    let after = slides(&app);
    assert_eq!(after.len(), 7);
    assert_eq!(
        (after[0], after[1], after[3], after[4], after[5], after[6]),
        (a, b, c, d, e, f)
    );
    app.duplicate_slide();
    let after = slides(&app);
    assert_eq!(after.len(), 8);
    assert_eq!(after[1], b);
    assert_eq!(
        app.session().document().slides[2].root.areas().len(),
        app.session().document().slides[1].root.areas().len()
    );
    app.remove_slide();
    assert!(!slides(&app).contains(&b));
    // The removed slide was shown, so the first is shown now.
    assert_eq!(app.shown_slide(), Some(a));
}

#[test]
fn a_slide_dropped_on_a_row_takes_its_place() {
    let mut app = sample();
    let [a, b, c, d, e, f] = slides(&app)[..] else {
        panic!("the sample has six slides");
    };
    app.move_slide(a, 2);
    assert_eq!(slides(&app), [b, c, a, d, e, f]);
    app.move_slide(a, 0);
    assert_eq!(slides(&app), [a, b, c, d, e, f]);
}

#[test]
fn a_node_s_path_runs_from_the_root_to_it() {
    let app = sample();
    let slide = &app.session().document().slides[0];
    let Node::Split(top) = &slide.root else {
        panic!("the sample's first slide is split");
    };
    let Some(Node::Split(inner)) = top.children.iter().find(|c| matches!(c, Node::Split(_))) else {
        panic!("the first slide holds a nested split");
    };
    let leaf = inner.children[1].id();
    let ids: Vec<_> = path(slide, leaf).unwrap().iter().map(|n| n.id()).collect();
    assert_eq!(ids, [top.id, inner.id, leaf]);
    assert_eq!(
        path(slide, slide.root.id()).unwrap().len(),
        1,
        "the root's path is the root alone"
    );
    assert!(path(slide, rootpoint_model::NodeId(999)).is_none());
}
