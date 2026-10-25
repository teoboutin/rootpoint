//! The slide view's gestures, driven by synthetic input through `Context::run_ui`, with no window:
//! a click selects an area, a divider drag is one undo step that Esc cancels, a double-click
//! resets a divider, Alt and the arrows move the selection, a row of the slide list is shown by a
//! click and moved by a drag, a grip drag swaps or moves an area, a click with Ctrl splits at the
//! pointer, and the keys undo, redo, split and remove; the side panel's outline, whose rows list
//! the slide's tree, select a node by a click and outline it on the slide under the pointer; and
//! the node menu, which a secondary click on an area or a row opens; and the final view, where a
//! press starts no gesture.

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
    /// The shapes the last frame drew.
    shapes: Vec<eframe::epaint::ClippedShape>,
}

impl Driver {
    fn new() -> Self {
        let mut app = App::new(Box::new(NoFiles));
        assert!(matches!(app.run(Command::Sample), Reply::Ok(_)));
        let mut d = Driver {
            app,
            ctx: Context::default(),
            time: 0.0,
            shapes: Vec::new(),
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
        let mut out = self.ctx.run_ui(input, |ui| app.frame(ui));
        // No painter applies the texture deltas, and egui refuses to drop them unapplied.
        out.textures_delta.clear();
        self.shapes = out.shapes;
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

    /// A click of the secondary button, as a right-click is.
    fn secondary_click(&mut self, pos: Pos2) {
        for pressed in [true, false] {
            self.frame(
                vec![
                    Event::PointerMoved(pos),
                    Event::PointerButton {
                        pos,
                        button: PointerButton::Secondary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
                Modifiers::NONE,
            );
        }
    }

    /// The middle of the outline's row of `node`, `None` for the slide's row.
    fn row(&self, node: Option<NodeId>) -> Pos2 {
        self.app
            .outline_rows()
            .iter()
            .find(|r| r.node == node)
            .unwrap()
            .rect
            .center()
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

    /// Select a node, and let one frame record the places the selection's marks and rows take.
    fn select(&mut self, node: NodeId) {
        self.app.select_node(node);
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
fn a_divider_dragged_back_to_the_press_returns_to_where_it_was() {
    let mut d = Driver::new();
    let (start, split, index) = d.divider();
    let pos = d.position(split, index);
    d.button(start, true);
    d.move_to(start + Vec2::new(30.0, 30.0));
    assert_ne!(d.position(split, index), pos, "the divider moved");
    // Back at the press, within the rounding of points in f32.
    d.move_to(start);
    assert!((d.position(split, index) - pos).abs() < 1e-6);
    d.button(start, false);
    assert!((d.position(split, index) - pos).abs() < 1e-6);
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

/// The outline lists the shown slide's every node, in depth-first order under the slide's row, each
/// at its depth, and highlights the selection's row.
#[test]
fn the_outline_lists_every_node_with_the_selection_highlighted() {
    let mut d = Driver::new();
    d.app.select_node(NodeId(3));
    d.frame(vec![], Modifiers::NONE);
    d.frame(vec![], Modifiers::NONE);
    let rows = d.app.outline_rows();
    let nodes: Vec<_> = rows
        .iter()
        .map(|r| (r.node.map(|n| n.0), r.depth))
        .collect();
    assert_eq!(
        nodes,
        [
            (None, 0),
            (Some(4), 1),
            (Some(2), 2),
            (Some(6), 2),
            (Some(3), 3),
            (Some(5), 3)
        ]
    );
    let highlighted: Vec<_> = rows.iter().filter(|r| r.selected).map(|r| r.node).collect();
    assert_eq!(highlighted, [Some(NodeId(3))]);
    // The highlight is painted: a filled rectangle on n3's row, and none on another row.
    let filled = |rect: Rect| {
        d.shapes.iter().any(|c| match &c.shape {
            eframe::egui::Shape::Rect(s) => {
                s.rect == rect && s.fill != eframe::egui::Color32::TRANSPARENT
            }
            _ => false,
        })
    };
    for row in rows {
        assert_eq!(filled(row.rect), row.selected, "{:?}", row.node);
    }
}

/// A click on a row selects its node, and a click on the slide's row selects none.
#[test]
fn a_click_on_an_outline_row_selects_its_node() {
    let mut d = Driver::new();
    let row = |d: &Driver, node: Option<NodeId>| {
        d.app
            .outline_rows()
            .iter()
            .find(|r| r.node == node)
            .unwrap()
            .rect
            .center()
    };
    let slide = d.app.shown_slide();
    d.click(row(&d, Some(NodeId(6))));
    assert_eq!(d.selected(), Some(NodeId(6)));
    d.frame(vec![], Modifiers::NONE);
    d.click(row(&d, None));
    assert_eq!(d.selected(), None);
    assert_eq!(d.app.session().view().shown, slide);
}

/// With the pointer on a row, the overlay outlines that row's node on the slide.
#[test]
fn hovering_an_outline_row_outlines_its_node() {
    let mut d = Driver::new();
    let n6 = NodeId(6);
    let outlined = |d: &Driver| {
        let r = d.app.layout().unwrap().rect(n6).unwrap();
        d.shapes.iter().any(|c| match &c.shape {
            eframe::egui::Shape::Rect(s) => s.rect == r && s.stroke.width > 0.0,
            _ => false,
        })
    };
    assert!(!outlined(&d), "n6 is outlined before any hover");
    let at = d
        .app
        .outline_rows()
        .iter()
        .find(|r| r.node == Some(n6))
        .unwrap()
        .rect
        .center();
    d.move_to(at);
    assert!(outlined(&d), "n6 is not outlined while its row is hovered");
}

/// Alt with an arrow moves as the outline reads: ↑ and ↓ to the sibling above and below, ← to the
/// parent, → to the child selected last in it, or its first. The sample's first slide is
/// n4 stacked [n2, n6 beside [n3, n5]].
#[test]
fn alt_and_the_arrows_move_the_selection_as_the_outline_reads() {
    let mut d = Driver::new();
    let alt = Modifiers::ALT;
    let n = NodeId;
    d.app.select_node(n(2));
    d.key(Key::ArrowDown, alt);
    assert_eq!(d.selected(), Some(n(6)));
    d.key(Key::ArrowRight, alt);
    assert_eq!(d.selected(), Some(n(3)));
    // Nothing above the first sibling.
    d.key(Key::ArrowUp, alt);
    assert_eq!(d.selected(), Some(n(3)));
    d.key(Key::ArrowDown, alt);
    assert_eq!(d.selected(), Some(n(5)));
    // Nothing below the last sibling, and nothing to the right of an area.
    d.key(Key::ArrowDown, alt);
    assert_eq!(d.selected(), Some(n(5)));
    d.key(Key::ArrowRight, alt);
    assert_eq!(d.selected(), Some(n(5)));
    d.key(Key::ArrowLeft, alt);
    assert_eq!(d.selected(), Some(n(6)));
    // Right returns to the child selected last, not the first.
    d.key(Key::ArrowRight, alt);
    assert_eq!(d.selected(), Some(n(5)));
    // Nothing to the left of the root.
    d.app.select_node(n(4));
    d.key(Key::ArrowLeft, alt);
    assert_eq!(d.selected(), Some(n(4)));
}

#[test]
fn a_click_on_a_row_shows_its_slide_and_a_drag_moves_it() {
    let mut d = Driver::new();
    let rows = d.app.slide_rows().to_vec();
    let ids: Vec<_> = rows.iter().map(|r| r.0).collect();
    assert_eq!(ids.len(), 6);
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
    assert_eq!(after, [ids[1], ids[2], ids[0], ids[3], ids[4], ids[5]]);
    // The drag showed no slide of its own.
    assert_eq!(d.app.shown_slide(), Some(ids[1]));
}

/// The text of an area of the shown slide.
fn text(d: &Driver, area: NodeId) -> String {
    match d.doc().node(area) {
        Some(Node::Area(a)) => match &a.content {
            rootpoint_model::Content::Text(t) => t.plain_text(),
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
    // The new area is the smaller part, before the line.
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

/// Enter editing the selected text area n3 by a second click on it, the caret at its text's end: the
/// click is inside the area near its bottom right corner, below and after its last line in any
/// built-in font, and clear of the dividers at its edges.
fn edit_n3(d: &mut Driver) {
    let n3 = NodeId(3);
    d.select(n3);
    let r = d.app.layout().unwrap().rect(n3).unwrap();
    let p = Pos2::new(r.right() - 0.05 * r.width(), r.bottom() - 0.05 * r.height());
    d.click(p);
    assert_eq!(d.app.editing(), Some(n3));
}

/// A click on the selected equation area enters no editing: it holds no text, and Delete then
/// removes the area, as a node key does.
#[test]
fn a_second_click_on_an_equation_area_enters_no_editing() {
    let mut d = Driver::new();
    let n3 = NodeId(3);
    assert!(matches!(
        d.app
            .run(rootpoint_protocol::parse_command(r"equation n3 $\alpha$").unwrap()),
        Reply::Ok(_)
    ));
    d.select(n3);
    let r = d.app.layout().unwrap().rect(n3).unwrap();
    d.click(Pos2::new(
        r.right() - 0.05 * r.width(),
        r.bottom() - 0.05 * r.height(),
    ));
    assert_eq!(d.app.editing(), None);
    assert_eq!(d.selected(), Some(n3));
    d.key(Key::Delete, Modifiers::NONE);
    assert!(d.doc().node(n3).is_none());
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
    // A composition of the input method joins the same stay.
    let preedit = |t: &str| {
        Event::Ime(eframe::egui::ImeEvent::Preedit {
            text: t.to_owned(),
            active_range_chars: None,
        })
    };
    d.frame(vec![preedit("´")], Modifiers::NONE);
    d.frame(
        vec![Event::Ime(eframe::egui::ImeEvent::Commit("é".to_owned()))],
        Modifiers::NONE,
    );
    assert_eq!(text(&d, n3), format!("{before}Xé"));
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

/// A secondary click on an area of the slide selects it, and the node menu is open on it on the next
/// frame, `design@rootpoint-app@node-menu`.
#[test]
fn a_secondary_click_on_an_area_selects_it_and_opens_the_node_menu() {
    let mut d = Driver::new();
    let n5 = NodeId(5);
    let at = d.app.layout().unwrap().rect(n5).unwrap().center();
    d.secondary_click(at);
    assert_eq!(d.selected(), Some(n5));
    d.frame(vec![], Modifiers::NONE);
    assert_eq!(d.app.menu_node(), Some(n5));
    assert!(
        d.app.menu_entry_rect("Remove n5").is_some(),
        "no menu drawn"
    );
    // A secondary click on the letterbox, which names no node, opens no menu.
    let slide = d.app.layout().unwrap().nodes[0].1;
    d.key(Key::Escape, Modifiers::NONE);
    assert_eq!(d.app.menu_node(), None, "Esc leaves the menu open");
    d.secondary_click(Pos2::new(slide.center().x, slide.max.y + 5.0));
    d.frame(vec![], Modifiers::NONE);
    assert_eq!(d.app.menu_node(), None);
}

/// A secondary click on a row of the outline selects its node, and the node menu is open on it on
/// the next frame, `design@rootpoint-app@node-menu`; on the slide's row, it opens none.
#[test]
fn a_secondary_click_on_an_outline_row_selects_it_and_opens_the_node_menu() {
    let mut d = Driver::new();
    let n6 = NodeId(6);
    d.secondary_click(d.row(Some(n6)));
    assert_eq!(d.selected(), Some(n6));
    d.frame(vec![], Modifiers::NONE);
    assert_eq!(d.app.menu_node(), Some(n6));
    assert!(
        d.app.menu_entry_rect("n4 stacked").is_some(),
        "no menu drawn"
    );
    d.secondary_click(d.row(None));
    d.frame(vec![], Modifiers::NONE);
    assert_eq!(d.app.menu_node(), None);
    assert_eq!(d.selected(), None);
}

/// The menu's "Select" lists the node's ancestors, and a click on one selects it and closes the
/// menu; a click elsewhere closes it and acts as it would.
#[test]
fn the_node_menu_selects_an_ancestor() {
    let mut d = Driver::new();
    let (n3, n4) = (NodeId(3), NodeId(4));
    let at = d.app.layout().unwrap().rect(n3).unwrap().center();
    d.secondary_click(at);
    d.frame(vec![], Modifiers::NONE);
    let labels: Vec<String> = d
        .app
        .menu_items()
        .into_iter()
        .filter(|i| i.section == rootpoint_app::MenuSection::Select)
        .map(|i| i.label)
        .collect();
    assert_eq!(labels, ["n4 stacked", "n6 beside", "n3 text"]);
    let entry = d.app.menu_entry_rect("n4 stacked").unwrap().center();
    d.click(entry);
    assert_eq!(d.selected(), Some(n4));
    assert_eq!(d.app.menu_node(), None);
    d.secondary_click(at);
    d.frame(vec![], Modifiers::NONE);
    assert_eq!(d.app.menu_node(), Some(n3));
    let other = d.app.layout().unwrap().rect(NodeId(2)).unwrap().center();
    d.click(other);
    assert_eq!(d.app.menu_node(), None);
    assert_eq!(d.selected(), Some(NodeId(2)));
}

/// While editing, a secondary click inside the edited area ends the editing, then opens the menu,
/// `design@rootpoint-app@edit-entry`.
#[test]
fn a_secondary_click_while_editing_ends_it_and_opens_the_menu() {
    let mut d = Driver::new();
    let n3 = NodeId(3);
    edit_n3(&mut d);
    d.pause();
    let at = d.app.layout().unwrap().rect(n3).unwrap().center();
    d.secondary_click(at);
    assert_eq!(d.app.editing(), None);
    d.frame(vec![], Modifiers::NONE);
    assert_eq!(d.app.menu_node(), Some(n3));
}

/// While editing, a secondary click on the letterbox ends the editing and opens no menu; during a
/// divider drag, a secondary click opens none.
#[test]
fn a_secondary_click_on_the_letterbox_ends_editing_and_none_opens_a_menu_mid_drag() {
    let mut d = Driver::new();
    edit_n3(&mut d);
    d.pause();
    let slide = d.app.layout().unwrap().nodes[0].1;
    d.secondary_click(Pos2::new(slide.center().x, slide.max.y + 5.0));
    assert_eq!(d.app.editing(), None);
    assert_eq!(d.app.menu_node(), None);
    d.pause();
    let (start, _, _) = d.divider();
    d.button(start, true);
    d.move_to(start + Vec2::new(20.0, 20.0));
    let on = start + Vec2::new(20.0, 20.0);
    d.secondary_click(on);
    d.frame(vec![], Modifiers::NONE);
    assert_eq!(d.app.menu_node(), None, "a menu opened during the drag");
    d.button(on, false);
}

/// The button "Preview" enters the final view, which deselects; a drag of a divider moves nothing;
/// a click on an area, with Ctrl held, splits nothing, selects the area and ends the final view;
/// Esc leaves it too.
#[test]
fn a_click_in_the_final_view_ends_it_and_selects() {
    let mut d = Driver::new();
    let n3 = NodeId(3);
    d.select(n3);
    let button = d.app.button_rect("Preview").unwrap().center();
    d.click(button);
    assert!(d.app.final_view());
    assert_eq!(d.selected(), None);
    let before = d.doc();
    // A drag of a divider moves nothing, and leaves the final view on.
    let (start, _, _) = d.divider();
    d.button(start, true);
    d.move_to(start + Vec2::new(40.0, 40.0));
    d.button(start + Vec2::new(40.0, 40.0), false);
    assert_eq!(d.doc(), before, "a drag in the final view moved a divider");
    assert!(d.app.final_view());
    d.pause();
    let at = d.app.layout().unwrap().rect(n3).unwrap().center();
    d.button_with(at, true, Modifiers::COMMAND);
    d.button_with(at, false, Modifiers::COMMAND);
    assert!(!d.app.final_view());
    assert_eq!(d.selected(), Some(n3));
    assert_eq!(d.doc(), before, "a Ctrl-click in the final view split");
    assert_eq!(d.app.editing(), None);
    d.pause();
    let button = d.app.button_rect("Preview").unwrap().center();
    d.click(button);
    assert!(d.app.final_view());
    d.key(Key::Escape, Modifiers::NONE);
    assert!(!d.app.final_view());
    // A secondary click in the final view selects, which ends it, and opens the node menu.
    d.pause();
    let button = d.app.button_rect("Preview").unwrap().center();
    d.click(button);
    assert!(d.app.final_view());
    d.secondary_click(at);
    d.frame(vec![], Modifiers::NONE);
    assert!(!d.app.final_view());
    assert_eq!(d.app.menu_node(), Some(n3));
}

/// A double-click in the final view acts once: its first click selects and leaves the final view,
/// and its second press starts nothing, so it resets no divider and enters no editing,
/// `design@rootpoint-app@edit-entry`.
#[test]
fn a_double_click_in_the_final_view_only_selects() {
    let mut d = Driver::new();
    let (start, split, index) = d.divider();
    let pos = d.position(split, index);
    let button = d.app.button_rect("Preview").unwrap().center();
    d.click(button);
    d.pause();
    d.click(start);
    d.click(start);
    assert!(!d.app.final_view());
    assert_eq!(
        d.position(split, index),
        pos,
        "the double-click reset the divider"
    );
    d.pause();
    let button = d.app.button_rect("Preview").unwrap().center();
    d.click(button);
    d.pause();
    let n3 = NodeId(3);
    let at = d.app.layout().unwrap().rect(n3).unwrap().center();
    d.click(at);
    d.click(at);
    assert_eq!(d.selected(), Some(n3));
    assert_eq!(d.app.editing(), None, "the double-click entered editing");
}

/// Esc leaves the final view with no slide shown, where the slide view reads no key.
#[test]
fn esc_leaves_the_final_view_with_no_slide_shown() {
    let mut d = Driver::new();
    assert!(matches!(d.app.run(Command::New), Reply::Ok(_)));
    d.frame(vec![], Modifiers::NONE);
    d.app.toggle_final_view();
    d.frame(vec![], Modifiers::NONE);
    assert!(d.app.final_view());
    d.key(Key::Escape, Modifiers::NONE);
    assert!(!d.app.final_view());
}

/// The marks the overlay drew over an area, read from the last frame's shapes: each rectangle
/// within `around`, with its fill and its stroke.
fn marks_over(d: &Driver, around: Rect) -> Vec<String> {
    d.shapes
        .iter()
        .filter_map(|c| match &c.shape {
            eframe::egui::Shape::Rect(s) if around.contains_rect(s.rect) => {
                Some(format!("{:?} {:?} {:?}", s.rect, s.fill, s.stroke))
            }
            _ => None,
        })
        .collect()
}

/// `design@rootpoint-app@editor-overlay`: the two caret positions at each edge of a span of the
/// sample's styled text show different marks, a frame at one and not at the other, or a caret of
/// another height or colour, as the overlay draws them with no window.
#[test]
fn the_two_positions_at_an_edge_show_different_marks() {
    use rootpoint_model::{CaretPos, Content, TextNodeRef};
    let mut d = Driver::new();
    let area = NodeId(26);
    d.app.show_slide(rootpoint_model::SlideId(24));
    d.frame(vec![], Modifiers::NONE);
    d.app.select_node(area);
    d.frame(vec![], Modifiers::NONE);
    let layout = d.app.layout().unwrap().clone();
    let rect = layout.rect(area).unwrap();
    d.app.edit_at(area, rect.center(), &layout);
    assert_eq!(d.app.editing(), Some(area));
    let Some(Node::Area(a)) = d.doc().node(area).cloned() else {
        panic!("n26 is an area");
    };
    let Content::Text(text) = a.content else {
        panic!("n26 holds a text");
    };
    let spans = text.root.spans();
    assert!(spans.len() >= 4, "the sample's styled text holds spans");
    // The node and the offset in it of each span, found from the root down.
    fn place(
        node: &rootpoint_model::TextNode,
        here: TextNodeRef,
        out: &mut Vec<(rootpoint_model::SpanId, TextNodeRef, usize, usize)>,
    ) {
        let mut offset = 0;
        for item in &node.items {
            match item {
                rootpoint_model::TextItem::Chars(s) => offset += s.chars().count(),
                rootpoint_model::TextItem::Span(s) => {
                    let len = s
                        .text
                        .items
                        .iter()
                        .map(|i| match i {
                            rootpoint_model::TextItem::Chars(c) => c.chars().count(),
                            rootpoint_model::TextItem::Span(_) => 1,
                        })
                        .sum();
                    out.push((s.id, here, offset, len));
                    place(&s.text, TextNodeRef::Span(s.id), out);
                    offset += 1;
                }
            }
        }
    }
    let mut places = Vec::new();
    place(&text.root, TextNodeRef::Root, &mut places);
    let around = rect.expand(20.0);
    let marks = |d: &mut Driver, pos: CaretPos| {
        d.app.set_caret_pos(pos);
        d.frame(vec![], Modifiers::NONE);
        marks_over(d, around)
    };
    for (span, parent, at, len) in places {
        let inside = TextNodeRef::Span(span);
        for (outer, inner) in [
            (
                CaretPos {
                    node: parent,
                    offset: at,
                },
                CaretPos {
                    node: inside,
                    offset: 0,
                },
            ),
            (
                CaretPos {
                    node: inside,
                    offset: len,
                },
                CaretPos {
                    node: parent,
                    offset: at + 1,
                },
            ),
        ] {
            let (a, b) = (marks(&mut d, outer), marks(&mut d, inner));
            assert_ne!(a, b, "{outer:?} and {inner:?} show the same marks");
        }
    }
}

/// While a text is edited, the Selection tab's controls act at the caret and keep the stay: the
/// bold checkbox opens a bold span there; a span of the ancestor row is selected whole by a click,
/// and "Split here" cuts the caret's span; a click on the row's text leaves editing, the area
/// still selected. The stay is one undo step.
#[test]
fn the_selection_tab_acts_at_the_caret_while_editing() {
    use rootpoint_model::{CaretPos, Content, TextNodeRef};
    let mut d = Driver::new();
    let area = NodeId(26);
    d.app.show_slide(rootpoint_model::SlideId(24));
    d.frame(vec![], Modifiers::NONE);
    d.app.select_node(area);
    d.frame(vec![], Modifiers::NONE);
    let before = d.doc();
    let layout = d.app.layout().unwrap().clone();
    d.app
        .edit_at(area, layout.rect(area).unwrap().center(), &layout);
    let spans = |d: &Driver| match d.doc().node(area).cloned() {
        Some(Node::Area(a)) => match a.content {
            Content::Text(t) => t
                .root
                .spans()
                .iter()
                .map(|s| (s.id, s.style.clone()))
                .collect::<Vec<_>>(),
            _ => panic!("n26 holds a text"),
        },
        _ => panic!("n26 is an area"),
    };
    let count = spans(&d).len();
    d.app.set_caret_pos(CaretPos {
        node: TextNodeRef::Root,
        offset: 0,
    });
    d.frame(vec![], Modifiers::NONE);
    d.click(
        d.app
            .button_rect("bold")
            .expect("the bold checkbox")
            .center(),
    );
    assert_eq!(d.app.editing(), Some(area), "the control ended the stay");
    assert_eq!(spans(&d).len(), count + 1, "no span opened at the caret");
    let caret = d.app.caret().unwrap();
    assert_ne!(
        caret.node,
        TextNodeRef::Root,
        "the caret is not in the new span"
    );
    // The row lists the caret's span; a click selects it whole, once it holds a letter, since an
    // empty span the caret leaves is unwrapped.
    d.frame(vec![Event::Text("X".into())], Modifiers::NONE);
    let TextNodeRef::Span(new) = caret.node else {
        unreachable!()
    };
    d.frame(vec![], Modifiers::NONE);
    d.click(
        d.app
            .button_rect(&new.to_string())
            .expect("the row lists the span")
            .center(),
    );
    assert_eq!(d.app.selected_span(), Some(new));
    d.frame(vec![], Modifiers::NONE);
    assert!(
        d.app.button_rect(&new.to_string()).is_some(),
        "the span the row selected left the row"
    );
    assert!(d.app.caret_spans().contains(&new));
    // In the bold span t28, "Split here" cuts it at the caret.
    let t28 = rootpoint_model::SpanId(28);
    d.app.set_caret_pos(CaretPos {
        node: TextNodeRef::Span(t28),
        offset: 2,
    });
    d.frame(vec![], Modifiers::NONE);
    let count = spans(&d).len();
    d.click(d.app.button_rect("Split here").unwrap().center());
    assert_eq!(spans(&d).len(), count + 1, "the span was not cut");
    assert_eq!(d.app.editing(), Some(area));
    d.frame(vec![], Modifiers::NONE);
    d.click(d.app.button_rect("text").unwrap().center());
    assert_eq!(d.app.editing(), None);
    assert_eq!(d.selected(), Some(area));
    assert!(matches!(d.app.run(Command::Undo), Reply::Ok(_)));
    assert!(
        d.doc().same_content(&before),
        "the stay is more than one undo step"
    );
}
