//! Editing a text in place, with no window: a click on the selected text area enters editing with
//! the caret at the click, and at three sizes of the window the caret stands on the glyph
//! boundary of the layout the render draws, within half a pixel, as
//! `design@rootpoint-app@edit-in-place` says.

use eframe::egui::{
    Context, Event, ImeEvent, Key, Modifiers, OutputCommand, PointerButton, Pos2, RawInput, Rect,
    Vec2,
};
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

    fn frame(&mut self, events: Vec<Event>) {
        self.frame_with(events, Modifiers::NONE);
    }

    /// One frame, and what it put on the clipboard and whether it allowed IME input.
    fn frame_with(&mut self, events: Vec<Event>, modifiers: Modifiers) -> (Vec<String>, bool) {
        self.frame_focused(events, modifiers, true)
    }

    fn frame_focused(
        &mut self,
        mut events: Vec<Event>,
        modifiers: Modifiers,
        focused: bool,
    ) -> (Vec<String>, bool) {
        self.time += 1.0 / 60.0;
        events.insert(0, Event::ModifiersChanged(modifiers));
        let mut input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.size)),
            max_texture_side: Some(8192),
            time: Some(self.time),
            events,
            focused,
            ..Default::default()
        };
        input
            .viewports
            .entry(input.viewport_id)
            .or_default()
            .native_pixels_per_point = Some(self.ppp);
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(input, |ui| app.frame(ui));
        // No painter applies the texture deltas, and egui refuses to drop them unapplied.
        out.textures_delta.clear();
        let copied = out
            .platform_output
            .commands
            .iter()
            .filter_map(|c| match c {
                OutputCommand::CopyText(t) => Some(t.clone()),
                _ => None,
            })
            .collect();
        (copied, out.platform_output.ime.is_some())
    }

    fn key_with(&mut self, key: Key, modifiers: Modifiers) {
        for pressed in [true, false] {
            self.frame_with(
                vec![Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers,
                }],
                modifiers,
            );
        }
    }

    fn button(&mut self, pos: Pos2, pressed: bool, modifiers: Modifiers) {
        self.frame_with(
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

    fn text_of(&self, area: NodeId) -> String {
        match self.app.session().document().node(area) {
            Some(rootpoint_model::Node::Area(a)) => match &a.content {
                rootpoint_model::Content::Text(t) => t.text.clone(),
                _ => String::new(),
            },
            _ => panic!("{area} is no area"),
        }
    }

    /// The point of the view where the text of `area` has the slide point `(u, v)` of its layout.
    fn text_point(&self, area: NodeId, u: f64, v: f64) -> Pos2 {
        let doc = self.app.session().document();
        let text = area_text(
            doc,
            self.app.shown_slide().unwrap(),
            area,
            &mut FontSet::new(),
        )
        .unwrap();
        self.app
            .layout()
            .unwrap()
            .to_points(text.origin.0 + u, text.origin.1 + v)
    }

    /// Select n3, then click its first character to edit it with the caret at offset 0.
    fn edit_n3(&mut self) {
        let n3 = NodeId(3);
        self.app.select_node(n3);
        self.settle();
        let p = self.text_point(n3, 2.0, 10.0);
        self.click(p);
        assert_eq!(self.app.editing(), Some(n3));
        assert_eq!(self.app.caret_index(), Some(0));
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
        d.app.move_caret(CaretMove::Right, false);
        assert_eq!(d.app.caret_index(), Some(11));
        d.app.move_caret(CaretMove::Left, false);
        assert_eq!(d.app.caret_index(), Some(10));
    }
}

#[test]
fn a_typed_word_is_undone_by_one_undo() {
    let mut d = Driver::new(Vec2::new(1000.0, 700.0), 1.0);
    let n3 = NodeId(3);
    let before = d.text_of(n3);
    d.edit_n3();
    for c in ["w", "o", "r", "d", " "] {
        d.frame(vec![Event::Text(c.to_owned())]);
    }
    assert_eq!(d.text_of(n3), format!("word {before}"));
    // Ctrl+Z ends the stay in editing and undoes it whole.
    d.key_with(Key::Z, Modifiers::COMMAND);
    assert_eq!(d.app.editing(), None);
    assert_eq!(d.text_of(n3), before);
    // And Ctrl+Shift+Z brings the word back, in one step.
    d.key_with(Key::Z, Modifiers::COMMAND | Modifiers::SHIFT);
    assert_eq!(d.text_of(n3), format!("word {before}"));
}

#[test]
fn shift_and_an_arrow_select_and_typing_replaces_the_selection() {
    let mut d = Driver::new(Vec2::new(1000.0, 700.0), 1.0);
    let n3 = NodeId(3);
    let before = d.text_of(n3);
    d.edit_n3();
    for _ in 0..5 {
        d.key_with(Key::ArrowRight, Modifiers::SHIFT);
    }
    assert_eq!(d.app.selected_text().unwrap(), before[..5]);
    // The selection draws.
    let layout = d.app.layout().unwrap().clone();
    assert_eq!(d.app.selection_rects(&layout).len(), 1);
    d.frame(vec![Event::Text("Z".to_owned())]);
    assert_eq!(d.text_of(n3), format!("Z{}", &before[5..]));
    assert_eq!(d.app.caret_index(), Some(1));
}

#[test]
fn a_drag_inside_the_text_selects_and_a_shift_click_extends() {
    let mut d = Driver::new(Vec2::new(1000.0, 700.0), 1.0);
    let n3 = NodeId(3);
    let before = d.text_of(n3);
    d.edit_n3();
    // Drag from the start of the first line to about 200 slide units along it.
    let from = d.text_point(n3, 2.0, 10.0);
    let to = d.text_point(n3, 200.0, 10.0);
    d.button(from, true, Modifiers::NONE);
    for i in 1..=4 {
        d.frame(vec![Event::PointerMoved(
            from + (to - from) * (i as f32 / 4.0),
        )]);
    }
    d.button(to, false, Modifiers::NONE);
    let dragged = d.app.selected_text().unwrap();
    assert!(
        dragged.len() > 3 && before.starts_with(&dragged),
        "{dragged:?}"
    );
    assert_eq!(d.app.editing(), Some(n3), "the drag stays in editing");
    // A Shift+click farther along extends the selection from the same anchor.
    let farther = d.text_point(n3, 400.0, 10.0);
    d.button(farther, true, Modifiers::SHIFT);
    d.button(farther, false, Modifiers::SHIFT);
    let extended = d.app.selected_text().unwrap();
    assert!(
        extended.len() > dragged.len() && before.starts_with(&extended),
        "{extended:?}"
    );
}

#[test]
fn a_click_where_the_grip_would_stand_moves_the_caret() {
    let mut d = Driver::new(Vec2::new(1000.0, 700.0), 1.0);
    let n3 = NodeId(3);
    d.edit_n3();
    d.key(Key::End);
    let doc = d.app.session().document();
    let text = area_text(doc, d.app.shown_slide().unwrap(), n3, &mut FontSet::new()).unwrap();
    let line = text.layout.lines().next().unwrap();
    let m = line.metrics();
    let middle = f64::from((m.block_min_coord + m.block_max_coord) / 2.0);
    // One unit right of the third glyph's left edge, on the middle of the first line.
    let x = drawn_glyphs(&d, n3)[2].1 - text.origin.0;
    let p = d.text_point(n3, x + 1.0, middle);
    let layout = d.app.layout().unwrap().clone();
    assert!(
        layout.grip(n3).is_some_and(|g| g.contains(p)),
        "the point is on the grip's square"
    );
    d.click(p);
    assert_eq!(d.app.editing(), Some(n3));
    assert_eq!(d.app.caret_index(), Some(2));
}

#[test]
fn copy_cut_and_paste_go_through_the_clipboard_events() {
    let mut d = Driver::new(Vec2::new(1000.0, 700.0), 1.0);
    let n3 = NodeId(3);
    let before = d.text_of(n3);
    d.edit_n3();
    for _ in 0..2 {
        d.key_with(Key::ArrowRight, Modifiers::SHIFT);
    }
    let (copied, _) = d.frame_with(vec![Event::Copy], Modifiers::NONE);
    assert_eq!(copied, [before[..2].to_owned()]);
    let (cut, _) = d.frame_with(vec![Event::Cut], Modifiers::NONE);
    assert_eq!(cut, [before[..2].to_owned()]);
    assert_eq!(d.text_of(n3), before[2..]);
    d.frame(vec![Event::Paste("pasted ".to_owned())]);
    assert_eq!(d.text_of(n3), format!("pasted {}", &before[2..]));
}

#[test]
fn an_ime_composition_stands_in_the_text_until_it_is_committed() {
    let mut d = Driver::new(Vec2::new(1000.0, 700.0), 1.0);
    let n3 = NodeId(3);
    let before = d.text_of(n3);
    d.edit_n3();
    // While editing, every frame allows IME input.
    let (_, ime) = d.frame_with(vec![], Modifiers::NONE);
    assert!(ime);
    // A dead key's composition: the accent, then the composed letter.
    let preedit = |t: &str| {
        Event::Ime(ImeEvent::Preedit {
            text: t.to_owned(),
            active_range_chars: None,
        })
    };
    d.frame(vec![preedit("´")]);
    assert_eq!(d.text_of(n3), format!("´{before}"));
    assert_eq!(d.app.preedit_range(), Some(0.."´".len()));
    d.frame(vec![preedit("é")]);
    assert_eq!(d.text_of(n3), format!("é{before}"));
    d.frame(vec![Event::Ime(ImeEvent::Commit("é".to_owned()))]);
    assert_eq!(d.text_of(n3), format!("é{before}"));
    assert_eq!(d.app.preedit_range(), None);
    assert_eq!(d.app.caret_index(), Some("é".len()));
    // An abandoned composition leaves the text as it was.
    d.frame(vec![preedit("あ")]);
    d.frame(vec![preedit("")]);
    assert_eq!(d.text_of(n3), format!("é{before}"));
    // A dead key that arrives as typed text is typed.
    d.frame(vec![Event::Text("è".to_owned())]);
    assert_eq!(d.text_of(n3), format!("éè{before}"));
    // Leaving editing allows no IME input any more.
    d.key(Key::Escape);
    let (_, ime) = d.frame_with(vec![], Modifiers::NONE);
    assert!(!ime);
}

#[test]
fn a_loss_of_the_window_s_focus_keeps_editing_in_one_undo_step() {
    let mut d = Driver::new(Vec2::new(1000.0, 700.0), 1.0);
    let n3 = NodeId(3);
    let before = d.text_of(n3);
    d.edit_n3();
    d.frame(vec![Event::Text("X".to_owned())]);
    // A resize on a desktop whose window manager takes the keyboard: the window is unfocused for
    // the frames of the resize, at another size.
    d.size = Vec2::new(900.0, 640.0);
    d.frame_focused(vec![], Modifiers::NONE, false);
    d.frame_focused(vec![], Modifiers::NONE, false);
    d.size = Vec2::new(1100.0, 760.0);
    d.frame(vec![]);
    assert_eq!(d.app.editing(), Some(n3), "the resize keeps editing");
    assert_eq!(d.app.caret_index(), Some(1), "and the caret");
    // Typing goes on in the same stay, whose edits stay one undo step.
    d.frame(vec![Event::Text("Y".to_owned())]);
    assert_eq!(d.text_of(n3), format!("XY{before}"));
    d.key(Key::Escape);
    assert!(matches!(d.app.run(Command::Undo), Reply::Ok(_)));
    assert_eq!(d.text_of(n3), before);
}

#[test]
fn an_empty_composition_started_with_a_selection_deletes_nothing() {
    let mut d = Driver::new(Vec2::new(1000.0, 700.0), 1.0);
    let n3 = NodeId(3);
    let before = d.text_of(n3);
    d.edit_n3();
    for _ in 0..3 {
        d.key_with(Key::ArrowRight, Modifiers::SHIFT);
    }
    // winit's X11 backend sends an empty preedit when a composition starts.
    d.frame(vec![Event::Ime(ImeEvent::Preedit {
        text: String::new(),
        active_range_chars: None,
    })]);
    assert_eq!(d.text_of(n3), before);
    assert_eq!(d.app.selected_text().unwrap(), before[..3]);
}
