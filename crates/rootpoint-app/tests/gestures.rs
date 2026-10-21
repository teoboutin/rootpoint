//! The slide view's gestures, driven by synthetic input through `Context::run_ui`, with no window:
//! a click selects an area, a divider drag is one undo step that Esc cancels, a double-click
//! resets a divider, Alt and the arrows move the selection, a row of the slide list is shown by a
//! click and moved by a drag, a grip drag swaps or moves an area, a click with Ctrl splits at the
//! pointer, and the keys undo, redo, split and remove.

use eframe::egui::{Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use rootpoint_app::App;
use rootpoint_app::hit::Press;
use rootpoint_model::{Document, Node, NodeId};
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

/// An app on the built-in sample, and the frames it is shown in, 1/60 s apart.
struct Driver {
    app: App,
    ctx: Context,
    time: f64,
}

impl Driver {
    fn new() -> Self {
        let mut app = App::new(Box::new(NoFiles));
        assert!(matches!(app.run(Command::Sample), Reply::Ok(_)));
        let mut d = Driver {
            app,
            ctx: Context::default(),
            time: 0.0,
        };
        // egui sizes its panels again on the second frame.
        d.frame(vec![], Modifiers::NONE);
        d.frame(vec![], Modifiers::NONE);
        d
    }

    fn frame(&mut self, mut events: Vec<Event>, modifiers: Modifiers) {
        self.time += 1.0 / 60.0;
        events.insert(0, Event::ModifiersChanged(modifiers));
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 700.0))),
            max_texture_side: Some(8192),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let app = &mut self.app;
        // No painter applies the texture deltas, and egui refuses to drop them unapplied.
        self.ctx
            .run_ui(input, |ui| app.frame(ui))
            .textures_delta
            .clear();
    }

    fn button(&mut self, pos: Pos2, pressed: bool) {
        self.button_with(pos, pressed, Modifiers::NONE);
    }

    fn button_with(&mut self, pos: Pos2, pressed: bool, modifiers: Modifiers) {
        self.frame(
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers,
                },
            ],
            modifiers,
        );
    }

    /// Half a second with no input, longer than egui's 0.3 s between the clicks of a double-click.
    fn pause(&mut self) {
        self.time += 0.5;
        self.frame(vec![], Modifiers::NONE);
    }

    fn move_to(&mut self, pos: Pos2) {
        self.frame(vec![Event::PointerMoved(pos)], Modifiers::NONE);
    }

    fn click(&mut self, pos: Pos2) {
        self.button(pos, true);
        self.button(pos, false);
    }

    fn key(&mut self, key: Key, modifiers: Modifiers) {
        let event = |pressed| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        };
        self.frame(vec![event(true)], modifiers);
        self.frame(vec![event(false)], modifiers);
    }

    /// Select a node, and let the side panel, which then shows the selection's actions, settle.
    fn select(&mut self, node: NodeId) {
        self.app.select_node(node);
        self.frame(vec![], Modifiers::NONE);
        self.frame(vec![], Modifiers::NONE);
    }

    fn doc(&self) -> Document {
        self.app.session().document().clone()
    }

    fn selected(&self) -> Option<NodeId> {
        self.app.session().view().selected
    }

    /// The middle of the first divider of the shown slide, and the split it divides.
    fn divider(&self) -> (Pos2, NodeId, usize) {
        let layout = self.app.layout().unwrap();
        let d = &layout.dividers[0];
        (d.from + (d.to - d.from) / 2.0, d.split, d.index)
    }

    fn position(&self, split: NodeId, index: usize) -> f64 {
        let slide = self.app.shown_slide().unwrap();
        let doc = self.app.session().document();
        match doc.slide(slide).unwrap().root.find(split) {
            Some(Node::Split(s)) => s.dividers[index],
            _ => panic!("{split} is no split"),
        }
    }
}

fn undo(app: &mut App) -> Reply {
    app.run(Command::Undo)
}

#[test]
fn a_click_on_an_area_selects_it() {
    let mut d = Driver::new();
    let layout = d.app.layout().unwrap().clone();
    let (area, rect) = layout
        .nodes
        .iter()
        .rev()
        .find(|(id, r)| layout.press(r.center(), None, None) == Press::Area(*id))
        .copied()
        .unwrap();
    d.click(rect.center());
    assert_eq!(d.selected(), Some(area));
    // A click in the letterbox, below the slide, selects nothing new.
    let slide = layout.nodes[0].1;
    d.click(Pos2::new(slide.center().x, slide.max.y + 5.0));
    assert_eq!(d.selected(), Some(area));
}

#[test]
fn a_divider_drag_is_one_undo_step() {
    let mut d = Driver::new();
    let before = d.doc();
    let (start, split, index) = d.divider();
    let pos = d.position(split, index);
    d.button(start, true);
    for i in 1..=5 {
        d.move_to(start + Vec2::new(8.0 * i as f32, 8.0 * i as f32));
    }
    d.button(start + Vec2::new(40.0, 40.0), false);
    assert_ne!(d.position(split, index), pos, "the divider moved");
    assert!(matches!(undo(&mut d.app), Reply::Ok(_)));
    assert_eq!(d.doc(), before);
}

#[test]
fn esc_cancels_a_divider_drag() {
    let mut d = Driver::new();
    let before = d.doc();
    let (start, _, _) = d.divider();
    d.button(start, true);
    d.move_to(start + Vec2::new(30.0, 30.0));
    assert_ne!(d.doc(), before);
    d.key(Key::Escape, Modifiers::NONE);
    assert_eq!(d.doc(), before);
    // The rest of the drag sends nothing, and the cancelled group left no step.
    d.move_to(start + Vec2::new(60.0, 60.0));
    d.button(start + Vec2::new(60.0, 60.0), false);
    assert_eq!(d.doc(), before);
    assert!(matches!(undo(&mut d.app), Reply::Err(_)));
}

#[test]
fn a_double_click_sets_a_divider_halfway_between_its_neighbours() {
    let mut d = Driver::new();
    let (start, split, index) = d.divider();
    // Move it off the middle first.
    d.button(start, true);
    d.move_to(start + Vec2::new(0.0, 60.0));
    d.move_to(start + Vec2::new(60.0, 60.0));
    d.button(start + Vec2::new(60.0, 60.0), false);
    let moved = d.doc();
    let (start, _, _) = d.divider();
    d.click(start);
    d.click(start);
    let Some(Node::Split(s)) = d
        .doc()
        .slide(d.app.shown_slide().unwrap())
        .unwrap()
        .root
        .find(split)
        .cloned()
    else {
        panic!("{split} is no split");
    };
    let want = (s.child_extent(index).0 + s.child_extent(index + 1).1) / 2.0;
    assert_eq!(d.position(split, index), want);
    // The reset is one step of its own.
    assert!(matches!(undo(&mut d.app), Reply::Ok(_)));
    assert_eq!(d.doc(), moved);
}

#[test]
fn alt_and_the_arrows_move_the_selection_through_the_tree() {
    let mut d = Driver::new();
    // The deepest area of the first slide: a child of a split that is itself a child.
    let slide = d.app.shown_slide().unwrap();
    let doc = d.doc();
    let root = &doc.slide(slide).unwrap().root;
    let Node::Split(top) = root else {
        panic!("the sample's first slide is split");
    };
    let (inner, child) = top
        .children
        .iter()
        .find_map(|c| match c {
            Node::Split(s) => Some((s.id, s.children[1].id())),
            Node::Area(_) => None,
        })
        .unwrap();
    d.app.select_node(child);
    let alt = Modifiers::ALT;
    d.key(Key::ArrowUp, alt);
    assert_eq!(d.selected(), Some(inner));
    // Down returns to the child selected last, not the first.
    d.key(Key::ArrowDown, alt);
    assert_eq!(d.selected(), Some(child));
    d.key(Key::ArrowLeft, alt);
    let Some(Node::Split(s)) = root.find(inner) else {
        panic!("{inner} is a split");
    };
    assert_eq!(d.selected(), Some(s.children[0].id()));
    // Nothing before the first sibling, nothing below an area, nothing above the root.
    d.key(Key::ArrowLeft, alt);
    assert_eq!(d.selected(), Some(s.children[0].id()));
    d.key(Key::ArrowDown, alt);
    assert_eq!(d.selected(), Some(s.children[0].id()));
    d.app.select_node(root.id());
    d.key(Key::ArrowUp, alt);
    assert_eq!(d.selected(), Some(root.id()));
}

#[test]
fn a_click_on_a_row_shows_its_slide_and_a_drag_moves_it() {
    let mut d = Driver::new();
    let rows = d.app.slide_rows().to_vec();
    let ids: Vec<_> = rows.iter().map(|r| r.0).collect();
    assert_eq!(ids.len(), 3);
    d.click(rows[1].1.center());
    assert_eq!(d.app.shown_slide(), Some(ids[1]));
    // Drag the first row onto the third.
    let (from, to) = (rows[0].1.center(), rows[2].1.center());
    d.button(from, true);
    for i in 1..=6 {
        d.move_to(from + (to - from) * (i as f32 / 6.0));
    }
    d.button(to, false);
    let after: Vec<_> = d.doc().slides.iter().map(|s| s.id).collect();
    assert_eq!(after, [ids[1], ids[2], ids[0]]);
    // The drag showed no slide of its own.
    assert_eq!(d.app.shown_slide(), Some(ids[1]));
}

/// The text of an area of the shown slide.
fn text(d: &Driver, area: NodeId) -> String {
    match d.doc().node(area) {
        Some(Node::Area(a)) => match &a.content {
            rootpoint_model::Content::Text(t) => t.text.clone(),
            _ => String::new(),
        },
        _ => panic!("{area} is no area"),
    }
}

#[test]
fn a_grip_dropped_on_the_middle_of_another_area_swaps_them() {
    let mut d = Driver::new();
    // The sample's first slide: n4 stacked[n2 | n6 beside[n3 | n5]].
    let (n3, n5) = (NodeId(3), NodeId(5));
    let (t3, t5) = (text(&d, n3), text(&d, n5));
    let before = d.doc();
    d.select(n3);
    let layout = d.app.layout().unwrap().clone();
    let grip = layout.grip(n3).unwrap().center();
    let target = layout.rect(n5).unwrap().center();
    d.button(grip, true);
    for i in 1..=5 {
        d.move_to(grip + (target - grip) * (i as f32 / 5.0));
    }
    d.button(target, false);
    assert_eq!((text(&d, n3), text(&d, n5)), (t5, t3));
    assert!(matches!(undo(&mut d.app), Reply::Ok(_)));
    assert_eq!(d.doc(), before);
}

#[test]
fn a_grip_dropped_near_an_edge_moves_the_area_there_and_esc_cancels() {
    let mut d = Driver::new();
    let (n2, n3) = (NodeId(2), NodeId(3));
    let t3 = text(&d, n3);
    let before = d.doc();
    d.select(n3);
    let layout = d.app.layout().unwrap().clone();
    let grip = layout.grip(n3).unwrap().center();
    let title = layout.rect(n2).unwrap();
    // Near the title's right edge, at its middle height: its move zone for the right edge.
    let target = Pos2::new(title.max.x - 5.0, title.center().y);
    d.button(grip, true);
    d.move_to(target);
    d.button(target, false);
    // n3 is gone, and its text is in a new area right of the title.
    let doc = d.doc();
    assert!(doc.node(n3).is_none());
    let slide = &doc.slides[0];
    let Node::Split(top) = &slide.root else {
        panic!("the root is a split");
    };
    let Node::Split(row) = &top.children[0] else {
        panic!("the title was split beside");
    };
    assert_eq!(row.children[0].id(), n2);
    let moved = row.children[1].id();
    assert_eq!(text(&d, moved), t3);
    assert!(matches!(undo(&mut d.app), Reply::Ok(_)));
    // Undo leaves the id counter where it is, so the documents are compared by their slides.
    assert_eq!(d.doc().slides, before.slides);
    // Esc during a grip drag sends nothing more, and leaves no step.
    d.select(n3);
    let after_select = d.doc();
    d.button(grip, true);
    d.move_to(target);
    d.key(Key::Escape, Modifiers::NONE);
    d.button(target, false);
    assert_eq!(d.doc(), after_select);
}

#[test]
fn a_click_with_ctrl_held_splits_the_selection_at_the_pointer() {
    let mut d = Driver::new();
    let n2 = NodeId(2);
    d.select(n2);
    let title = d.app.layout().unwrap().rect(n2).unwrap();
    let p = Pos2::new(title.min.x + title.width() * 0.25, title.center().y);
    let ctrl = Modifiers::COMMAND;
    d.button_with(p, true, ctrl);
    d.button_with(p, false, ctrl);
    let doc = d.doc();
    let Node::Split(top) = &doc.slides[0].root else {
        panic!("the root is a split");
    };
    let Node::Split(row) = &top.children[0] else {
        panic!("the title was split beside");
    };
    assert_eq!(row.axis, rootpoint_model::Axis::LeftToRight);
    assert!((row.dividers[0] - 0.25).abs() < 1e-3, "{:?}", row.dividers);
    // The new empty area is the smaller part, before the line.
    assert_eq!(row.children[1].id(), n2);
}

#[test]
fn the_keys_split_remove_undo_and_redo() {
    let mut d = Driver::new();
    let n3 = NodeId(3);
    // Undo leaves the id counter where it is, so the documents are compared by their slides.
    let slides = |d: &Driver| d.doc().slides;
    let before = slides(&d);
    d.app.select_node(n3);
    d.key(Key::ArrowRight, Modifiers::SHIFT);
    let split = slides(&d);
    assert_ne!(split, before);
    let Some(Node::Split(s)) = d.doc().node(NodeId(6)).cloned() else {
        panic!("n6 is a split");
    };
    assert_eq!(s.children.len(), 3, "n3 split beside, merged into n6");
    d.key(Key::Delete, Modifiers::NONE);
    assert!(d.doc().node(n3).is_none());
    d.key(Key::Z, Modifiers::COMMAND);
    assert_eq!(slides(&d), split);
    d.key(Key::Z, Modifiers::COMMAND);
    assert_eq!(slides(&d), before);
    d.key(Key::Z, Modifiers::COMMAND | Modifiers::SHIFT);
    assert_eq!(slides(&d), split);
    d.key(Key::Y, Modifiers::COMMAND);
    assert!(d.doc().node(n3).is_none());
}

/// Enter editing the selected text area n3 by a second click on it, the caret at its text's end.
fn edit_n3(d: &mut Driver) {
    let n3 = NodeId(3);
    d.select(n3);
    let p = d.app.layout().unwrap().rect(n3).unwrap().center();
    d.click(p);
    assert_eq!(d.app.editing(), Some(n3));
}

#[test]
fn the_edits_of_one_stay_in_editing_are_one_undo_step() {
    let mut d = Driver::new();
    let n3 = NodeId(3);
    let before = text(&d, n3);
    edit_n3(&mut d);
    for word in ["X", "Y"] {
        d.frame(vec![Event::Text(word.to_owned())], Modifiers::NONE);
    }
    let typed = text(&d, n3);
    assert_eq!(typed, format!("{before}XY"));
    // While editing, Backspace edits the text and Delete removes no area.
    d.key(Key::Backspace, Modifiers::NONE);
    d.key(Key::Delete, Modifiers::NONE);
    assert_eq!(text(&d, n3), format!("{before}X"));
    // A click on another area leaves editing, and the group closes: one undo reverts every edit.
    let elsewhere = d.app.layout().unwrap().rect(NodeId(5)).unwrap().center();
    d.click(elsewhere);
    assert_eq!(d.app.editing(), None);
    assert_eq!(d.selected(), Some(NodeId(5)));
    assert!(matches!(undo(&mut d.app), Reply::Ok(_)));
    assert_eq!(text(&d, n3), before);
}

/// A point a quarter along the first divider, `off` points across its line: at its middle, the
/// sample's first slide has another divider's end.
fn beside_divider(d: &Driver, off: f32) -> Pos2 {
    let layout = d.app.layout().unwrap();
    let divider = &layout.dividers[0];
    let across = match divider.axis {
        rootpoint_model::Axis::LeftToRight => Vec2::new(off, 0.0),
        rootpoint_model::Axis::TopToBottom => Vec2::new(0.0, off),
    };
    divider.from + (divider.to - divider.from) / 4.0 + across
}

#[test]
fn a_click_near_a_divider_moves_nothing_and_a_double_click_off_its_line_is_one_step() {
    let mut d = Driver::new();
    let before = d.doc();
    let near = beside_divider(&d, 3.0);
    d.click(near);
    assert_eq!(
        d.doc(),
        before,
        "a click 3 points from a divider moves nothing"
    );
    assert!(
        matches!(undo(&mut d.app), Reply::Err(_)),
        "and leaves no step"
    );
    // Move the divider off its middle, then double-click 3 points beside it.
    d.pause();
    let (start, split, index) = d.divider();
    d.button(start, true);
    d.move_to(start + Vec2::new(0.0, 60.0));
    d.move_to(start + Vec2::new(60.0, 60.0));
    d.button(start + Vec2::new(60.0, 60.0), false);
    let moved = d.position(split, index);
    let near = beside_divider(&d, 3.0);
    d.click(near);
    d.click(near);
    assert_ne!(d.position(split, index), moved);
    assert!(matches!(undo(&mut d.app), Reply::Ok(_)));
    assert_eq!(d.position(split, index), moved, "the reset is one step");
}

#[test]
fn a_divider_dragged_from_off_its_line_keeps_the_offset() {
    let mut d = Driver::new();
    let (_, split, index) = d.divider();
    let pos = d.position(split, index);
    let near = beside_divider(&d, 3.0);
    d.button(near, true);
    // A move along the line, not across it, leaves the divider where it was.
    let along = {
        let layout = d.app.layout().unwrap();
        let divider = &layout.dividers[0];
        (divider.to - divider.from).normalized() * 20.0
    };
    d.move_to(near + along);
    d.button(near + along, false);
    assert!((d.position(split, index) - pos).abs() < 1e-6);
}

#[test]
fn releasing_ctrl_before_the_button_issues_no_split() {
    let mut d = Driver::new();
    let n2 = NodeId(2);
    d.select(n2);
    let before = d.doc();
    let title = d.app.layout().unwrap().rect(n2).unwrap();
    let p = Pos2::new(title.min.x + title.width() * 0.25, title.center().y);
    d.button_with(p, true, Modifiers::COMMAND);
    d.frame(vec![], Modifiers::NONE);
    d.button_with(p, false, Modifiers::NONE);
    assert_eq!(d.doc(), before);
}

#[test]
fn ctrl_z_during_a_divider_drag_leaves_the_drag_one_step() {
    let mut d = Driver::new();
    let before = d.doc();
    let (start, split, index) = d.divider();
    d.button(start, true);
    d.move_to(start + Vec2::new(20.0, 20.0));
    d.key(Key::Z, Modifiers::COMMAND);
    d.move_to(start + Vec2::new(40.0, 40.0));
    d.button(start + Vec2::new(40.0, 40.0), false);
    assert!(d.app.status().is_empty(), "{:?}", d.app.status());
    assert_ne!(
        d.position(split, index),
        before_position(&before, split, index)
    );
    assert!(matches!(undo(&mut d.app), Reply::Ok(_)));
    assert_eq!(d.doc(), before);
}

fn before_position(doc: &Document, split: NodeId, index: usize) -> f64 {
    match doc.node(split) {
        Some(Node::Split(s)) => s.dividers[index],
        _ => panic!("{split} is no split"),
    }
}

#[test]
fn a_slide_list_button_pressed_while_a_text_is_edited_is_a_step_of_its_own() {
    let mut d = Driver::new();
    let n3 = NodeId(3);
    let before = text(&d, n3);
    edit_n3(&mut d);
    d.frame(vec![Event::Text("X".to_owned())], Modifiers::NONE);
    let typed = text(&d, n3);
    assert_ne!(typed, before);
    // The slide list's "Add" button is above its first row: probe up from the row until it adds.
    let rows = d.app.slide_rows().to_vec();
    let slides = d.doc().slides.len();
    for dy in [8.0f32, 12.0, 16.0, 20.0, 24.0, 28.0, 32.0] {
        d.click(Pos2::new(rows[0].1.min.x + 14.0, rows[0].1.min.y - dy));
        d.frame(vec![], Modifiers::NONE);
        if d.doc().slides.len() != slides {
            break;
        }
    }
    assert_eq!(
        d.doc().slides.len(),
        slides + 1,
        "the Add button added a slide"
    );
    // One undo removes the slide alone; the typing is a step of its own.
    assert!(matches!(undo(&mut d.app), Reply::Ok(_)));
    assert_eq!(d.doc().slides.len(), slides);
    assert_eq!(text(&d, n3), typed);
    assert!(matches!(undo(&mut d.app), Reply::Ok(_)));
    assert_eq!(text(&d, n3), before);
}
