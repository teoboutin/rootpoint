//! The slide view's gestures, driven by synthetic input through `Context::run_ui`, with no window:
//! a click selects an area, a divider drag is one undo step that Esc cancels, a double-click
//! resets a divider, Alt and the arrows move the selection, and a row of the slide list is shown
//! by a click and moved by a drag.

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
        self.frame(
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
            Modifiers::NONE,
        );
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
        .find(|(id, r)| layout.press(r.center()) == Press::Area(*id))
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
