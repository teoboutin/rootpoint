//! Editing a text in place, with no window: a click on the selected text area enters editing with
//! the caret at the click, and at three sizes of the window the caret stands on the glyph
//! boundary of the layout the render draws, within half a pixel. Claim 1 of the slice that edits
//! text in place, `spec@first-prototype@text-editing`.

use eframe::egui::{Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use parley::PositionedLayoutItem;
use rootpoint_app::App;
use rootpoint_app::editing::CaretMove;
use rootpoint_model::NodeId;
use rootpoint_protocol::{Command, Files, Reply};
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

/// The sample's first slide, with frames shown in a window of `size` points at `ppp` pixels per
/// point.
struct Driver {
    app: App,
    ctx: Context,
    time: f64,
    size: Vec2,
    ppp: f32,
}

impl Driver {
    fn new(size: Vec2, ppp: f32) -> Self {
        let mut app = App::new(Box::new(NoFiles));
        assert!(matches!(app.run(Command::Sample), Reply::Ok(_)));
        let mut d = Driver {
            app,
            ctx: Context::default(),
            time: 0.0,
            size,
            ppp,
        };
        d.settle();
        d
    }

    fn frame(&mut self, mut events: Vec<Event>) {
        self.time += 1.0 / 60.0;
        events.insert(0, Event::ModifiersChanged(Modifiers::NONE));
        let mut input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.size)),
            max_texture_side: Some(8192),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        input
            .viewports
            .entry(input.viewport_id)
            .or_default()
            .native_pixels_per_point = Some(self.ppp);
        let app = &mut self.app;
        // No painter applies the texture deltas, and egui refuses to drop them unapplied.
        self.ctx
            .run_ui(input, |ui| app.frame(ui))
            .textures_delta
            .clear();
    }

    /// egui sizes its panels again on the next frames after a change of the side panel.
    fn settle(&mut self) {
        for _ in 0..3 {
            self.frame(vec![]);
        }
    }

    fn click(&mut self, pos: Pos2) {
        for pressed in [true, false] {
            self.frame(vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ]);
        }
    }

    fn key(&mut self, key: Key) {
        for pressed in [true, false] {
            self.frame(vec![Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Modifiers::NONE,
            }]);
        }
    }

    fn caret(&mut self) -> Rect {
        let layout = self.app.layout().unwrap().clone();
        self.app.caret_rect(&layout).expect("a caret while editing")
    }
}

/// The x of each glyph the render draws for `area`, in slide units of the slide, with the byte
/// offset its character starts at: one glyph per character in the fixture, which is plain ASCII.
fn drawn_glyphs(d: &Driver, area: NodeId) -> Vec<(usize, f64)> {
    let doc = d.app.session().document();
    let slide = d.app.shown_slide().unwrap();
    let text = area_text(doc, slide, area, &mut FontSet::new()).unwrap();
    let mut out = Vec::new();
    for line in text.layout.lines() {
        let mut offsets = line.text_range();
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(run) = item {
                for g in run.positioned_glyphs() {
                    if let Some(i) = offsets.next() {
                        out.push((i, text.origin.0 + f64::from(g.x)));
                    }
                }
            }
        }
    }
    out
}

#[test]
fn a_click_on_the_selected_text_area_enters_editing_and_a_first_click_only_selects() {
    let mut d = Driver::new(Vec2::new(1000.0, 700.0), 1.0);
    let n3 = NodeId(3);
    let r = d.app.layout().unwrap().rect(n3).unwrap();
    let p = r.center();
    d.click(p);
    assert_eq!(d.app.session().view().selected, Some(n3));
    assert_eq!(d.app.editing(), None, "a first click only selects");
    d.settle();
    // A click 2 slide units into the first character of the first line puts the caret before it.
    let doc = d.app.session().document();
    let text = area_text(doc, d.app.shown_slide().unwrap(), n3, &mut FontSet::new()).unwrap();
    let p = d
        .app
        .layout()
        .unwrap()
        .to_points(text.origin.0 + 2.0, text.origin.1 + 10.0);
    d.click(p);
    assert_eq!(
        d.app.editing(),
        Some(n3),
        "a click on the selected text area edits it"
    );
    assert_eq!(d.app.caret_index(), Some(0));
    // The arrow keys move it.
    d.key(Key::ArrowRight);
    d.key(Key::ArrowRight);
    assert_eq!(d.app.caret_index(), Some(2));
    d.key(Key::End);
    let end_of_line = d.app.caret_index().unwrap();
    assert!(end_of_line > 2, "{end_of_line}");
    // Esc leaves editing, and the area stays selected.
    d.key(Key::Escape);
    assert_eq!(d.app.editing(), None);
    assert_eq!(d.app.session().view().selected, Some(n3));
    // While not editing, the keys act on the node again: Alt+Up selects the parent.
    d.frame(vec![Event::Key {
        key: Key::ArrowUp,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::ALT,
    }]);
    assert_eq!(d.app.session().view().selected, Some(NodeId(6)));
}

#[test]
fn the_caret_stands_on_the_drawn_glyph_boundary_at_three_window_sizes() {
    let n3 = NodeId(3);
    for (size, ppp) in [
        (Vec2::new(1000.0, 700.0), 1.0),
        (Vec2::new(1400.0, 900.0), 1.5),
        (Vec2::new(700.0, 500.0), 2.0),
    ] {
        let mut d = Driver::new(size, ppp);
        d.app.select_node(n3);
        d.settle();
        let layout = d.app.layout().unwrap().clone();
        let p = layout.rect(n3).unwrap().center();
        d.app.edit_at(n3, p, &layout);
        assert_eq!(d.app.editing(), Some(n3));
        let glyphs = drawn_glyphs(&d, n3);
        assert!(glyphs.len() > 100, "the fixture's text");
        for (index, x) in glyphs {
            d.app.set_caret(index);
            let caret = d.caret();
            // The caret's left edge, against the glyph's x carried to the view by the slide's
            // transform, in pixels.
            let want = layout.to_points(x, 0.0).x;
            let off = (caret.min.x - want).abs() * ppp;
            assert!(off <= 0.5, "offset {index}: {off} px at {size:?} × {ppp}");
        }
        // The arrows move the caret by one character.
        d.app.set_caret(10);
        d.app.move_caret(CaretMove::Right);
        assert_eq!(d.app.caret_index(), Some(11));
        d.app.move_caret(CaretMove::Left);
        assert_eq!(d.app.caret_index(), Some(10));
    }
}
