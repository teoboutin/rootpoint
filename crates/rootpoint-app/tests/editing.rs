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
use rootpoint_app::hit::Layout;
use rootpoint_model::{CaretPos, NodeId, TextContent, TextItem, TextNode, TextNodeRef};
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
                rootpoint_model::Content::Text(t) => t.plain_text(),
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

/// A new area holds an empty text, so a second click on it enters editing with no "text" button
/// first, per `design@rootpoint-app@edit-entry`.
#[test]
fn a_second_click_on_a_new_area_enters_editing() {
    let mut d = Driver::new(Vec2::new(1000.0, 700.0), 1.0);
    d.app.add_slide();
    let doc = d.app.session().document();
    let shown = d.app.shown_slide().unwrap();
    let at = doc.slides.iter().position(|s| s.id == shown).unwrap();
    let slide = &doc.slides[at + 1];
    let (slide, area) = (slide.id, slide.root.id());
    d.app.show_slide(slide);
    d.settle();
    let p = d.app.layout().unwrap().rect(area).unwrap().center();
    d.click(p);
    assert_eq!(d.app.session().view().selected, Some(area));
    assert_eq!(d.app.editing(), None, "a first click only selects");
    d.settle();
    d.click(p);
    assert_eq!(
        d.app.editing(),
        Some(area),
        "a second click edits the new area"
    );
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
    // While not editing, the keys act on the node again: Alt+Left selects the parent.
    d.frame(vec![Event::Key {
        key: Key::ArrowLeft,
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
        assert!(glyphs.len() > 100, "the sample's text");
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

/// Set the text of n3 by a `text` command, as an agent writes a styled text.
fn set_text(d: &mut Driver, tree: &str) {
    let line = format!("text n3 {tree}");
    let command = parse_command(&line).unwrap_or_else(|e| panic!("{line}: {e}"));
    assert!(matches!(d.app.run(command), Reply::Ok(_)), "{line}");
}

/// The text of `area` in a short form: its characters, each span in braces.
fn shape(d: &Driver, area: NodeId) -> String {
    fn of(node: &TextNode, out: &mut String) {
        for item in &node.items {
            match item {
                TextItem::Chars(s) => out.push_str(s),
                TextItem::Span(span) => {
                    out.push('{');
                    of(&span.text, out);
                    out.push('}');
                }
            }
        }
    }
    let mut out = String::new();
    of(&content(d, area).root, &mut out);
    out
}

fn content(d: &Driver, area: NodeId) -> TextContent {
    match d.app.session().document().node(area) {
        Some(rootpoint_model::Node::Area(a)) => match &a.content {
            rootpoint_model::Content::Text(t) => t.clone(),
            _ => panic!("{area} holds no text"),
        },
        _ => panic!("{area} is no area"),
    }
}

/// The id of the `i`-th span of n3's text, in the order of the text.
fn span(d: &Driver, i: usize) -> TextNodeRef {
    TextNodeRef::Span(content(d, NodeId(3)).root.spans()[i].id)
}

fn at(node: TextNodeRef, offset: usize) -> CaretPos {
    CaretPos { node, offset }
}

const ROOT: TextNodeRef = TextNodeRef::Root;

/// Edit n3, holding `tree`, by a call with no window.
fn edit_tree(tree: &str) -> Driver {
    let mut d = Driver::new(Vec2::new(1000.0, 700.0), 1.0);
    set_text(&mut d, tree);
    d.app.select_node(NodeId(3));
    d.settle();
    let layout = d.app.layout().unwrap().clone();
    let p = layout.rect(NodeId(3)).unwrap().center();
    d.app.edit_at(NodeId(3), p, &layout);
    assert_eq!(d.app.editing(), Some(NodeId(3)));
    d
}

/// Each edge of a span is two caret positions, and typing goes into the caret's text node, per
/// `design@rootpoint-app@boundary-caret-positions`: before the span and at its start, at its end
/// and after it.
#[test]
fn typing_at_each_position_of_an_edge_goes_into_the_caret_s_text_node() {
    let mut d = edit_tree(r#"["ab", {bold "cd"}, "ef"]"#);
    let s = span(&d, 0);
    for (pos, typed, want) in [
        (at(ROOT, 2), "1", "ab1{cd}ef"),
        (at(s, 0), "2", "ab1{2cd}ef"),
        (at(s, 3), "3", "ab1{2cd3}ef"),
        (at(ROOT, 4), "4", "ab1{2cd3}4ef"),
    ] {
        d.app.set_caret_pos(pos);
        d.app.type_str(typed);
        assert_eq!(shape(&d, NodeId(3)), want, "typed at {pos:?}");
    }
}

/// Right and Left walk `design@rootpoint-app@boundary-caret-positions`: into a span at its start,
/// out of it at its end, and both positions of an edge stand at one byte index.
#[test]
fn the_arrows_enter_and_leave_a_span_at_its_edges() {
    let mut d = edit_tree(r#"["a", {bold "b", {italic "c"}}, "d"]"#);
    let (outer, inner) = (span(&d, 0), span(&d, 1));
    d.app.set_caret_pos(at(ROOT, 0));
    let mut walk = vec![d.app.caret().unwrap()];
    for _ in 0..8 {
        d.app.move_caret(CaretMove::Right, false);
        walk.push(d.app.caret().unwrap());
    }
    assert_eq!(
        walk,
        [
            at(ROOT, 0),
            at(ROOT, 1),
            at(outer, 0),
            at(outer, 1),
            at(inner, 0),
            at(inner, 1),
            at(outer, 2),
            at(ROOT, 2),
            at(ROOT, 3),
        ]
    );
    // The two positions of the edge before the outer span stand at one byte index.
    d.app.set_caret_pos(at(ROOT, 1));
    let before = d.app.caret_index();
    d.app.set_caret_pos(at(outer, 0));
    assert_eq!(d.app.caret_index(), before);
}

/// A click puts the caret in the deepest text node that owns the clicked glyph, on the side of
/// the glyph clicked, `design@rootpoint-app@boundary-caret-positions`: the left half of
/// the span's first glyph is inside the span, the right half of the glyph before it outside.
#[test]
fn a_click_goes_into_the_text_node_of_the_glyph_on_its_side() {
    let mut d = edit_tree(r#"["ab", {bold "cd"}, "ef"]"#);
    let s = span(&d, 0);
    let glyphs = drawn_glyphs(&d, NodeId(3));
    let origin = {
        let doc = d.app.session().document();
        area_text(
            doc,
            d.app.shown_slide().unwrap(),
            NodeId(3),
            &mut FontSet::new(),
        )
        .unwrap()
        .origin
    };
    let (b, c) = (glyphs[1].1 - origin.0, glyphs[2].1 - origin.0);
    let layout = d.app.layout().unwrap().clone();
    for (x, want) in [
        (c + 1.0, at(s, 0)),
        (c - 1.0, at(ROOT, 2)),
        (b + 1.0, at(ROOT, 1)),
    ] {
        let p = d.text_point(NodeId(3), x, 10.0);
        d.app.place_caret(p, &layout);
        assert_eq!(d.app.caret(), Some(want), "a click at {x}");
    }
}

/// A selection is a range of one text node: Shift with an arrow passes a span whole, and a
/// selection begun inside a span and extended past its end rises to its parent and takes the span
/// whole, `design@rootpoint-app@selection-one-level`; typing then replaces the span.
#[test]
fn a_selection_stays_in_one_text_node_and_takes_a_span_whole() {
    let mut d = edit_tree(r#"["ab", {bold "cd"}, "ef"]"#);
    d.app.set_caret_pos(at(ROOT, 2));
    d.app.move_caret(CaretMove::Right, true);
    assert_eq!(
        d.app.caret(),
        Some(at(ROOT, 3)),
        "Shift+Right passes the span"
    );
    assert_eq!(d.app.selected_text().as_deref(), Some("cd"));
    let s = span(&d, 0);
    d.app.set_caret_pos(at(s, 1));
    d.app.move_caret(CaretMove::Right, true);
    assert_eq!(
        d.app.selected_text().as_deref(),
        Some("d"),
        "inside the span"
    );
    d.app.move_caret(CaretMove::Right, true);
    assert_eq!(
        d.app.caret(),
        Some(at(ROOT, 3)),
        "past its end, in the root"
    );
    assert_eq!(
        d.app.selected_text().as_deref(),
        Some("cd"),
        "the span whole"
    );
    d.app.move_caret(CaretMove::Right, true);
    assert_eq!(d.app.selected_text().as_deref(), Some("cde"));
    let layout = d.app.layout().unwrap().clone();
    assert_eq!(
        d.app.selection_rects(&layout).len(),
        1,
        "the selection draws"
    );
    d.app.type_str("Z");
    assert_eq!(shape(&d, NodeId(3)), "abZf");
    assert_eq!(d.app.caret(), Some(at(ROOT, 3)));
}

/// A cut takes a span inside the selection whole, and the clipboard its characters alone,
/// `issue@rootpoint-app@styled-copy-paste`; a paste inserts plain text into the caret's text node.
#[test]
fn a_cut_takes_a_span_whole_and_a_paste_inserts_plain_text_at_the_caret() {
    let mut d = edit_tree(r#"["ab", {bold "cd"}, "ef"]"#);
    d.app.set_caret_pos(at(ROOT, 1));
    for _ in 0..3 {
        d.app.move_caret(CaretMove::Right, true);
    }
    assert_eq!(d.app.cut().as_deref(), Some("bcde"));
    assert_eq!(shape(&d, NodeId(3)), "af");
    set_text(&mut d, r#"["ab", {bold "cd"}, "ef"]"#);
    let s = span(&d, 0);
    d.app.set_caret_pos(at(s, 1));
    d.frame(vec![Event::Paste("xy".to_owned())]);
    assert_eq!(shape(&d, NodeId(3)), "ab{cxyd}ef");
}

/// The editor unwraps the empty span the caret leaves, and each empty span of the text when the
/// stay ends, `design@rootpoint-app@empty-span-unwrapped`, inside the stay's one undo step.
#[test]
fn an_empty_span_is_unwrapped_when_the_caret_leaves_it_and_at_the_stay_s_end() {
    let mut d = edit_tree(r#"["ab", {bold "c"}, "d"]"#);
    let s = span(&d, 0);
    d.app.set_caret_pos(at(s, 1));
    d.app.delete_backward();
    assert_eq!(
        shape(&d, NodeId(3)),
        "ab{}d",
        "the span stays while the caret is in it"
    );
    d.app.move_caret(CaretMove::Right, false);
    assert_eq!(shape(&d, NodeId(3)), "abd");
    assert_eq!(d.app.caret(), Some(at(ROOT, 2)), "where the span stood");
    d.app.stop_editing();
    let mut d = edit_tree(r#"["ab", {bold ""}, "d", {italic {bold ""}}]"#);
    let before = shape(&d, NodeId(3));
    d.app.set_caret_pos(at(ROOT, 0));
    d.app.type_str("X");
    d.app.stop_editing();
    assert_eq!(shape(&d, NodeId(3)), "Xabd");
    assert!(matches!(d.app.run(Command::Undo), Reply::Ok(_)));
    assert_eq!(shape(&d, NodeId(3)), before, "one undo step");
}

/// Backspace and Delete stay in the caret's text node, `design@rootpoint-app@delete-in-caret-node`:
/// a span is selected by a first press and deleted by a second, and at a span's own edge the key
/// unwraps it, its characters kept.
#[test]
fn backspace_and_delete_at_an_edge_select_a_span_or_unwrap_it() {
    let mut d = edit_tree(r#"["ab", {bold "cd"}, "ef"]"#);
    d.app.set_caret_pos(at(ROOT, 3));
    d.app.delete_backward();
    assert_eq!(
        d.app.selected_text().as_deref(),
        Some("cd"),
        "a first press selects"
    );
    assert_eq!(shape(&d, NodeId(3)), "ab{cd}ef");
    d.app.delete_backward();
    assert_eq!(shape(&d, NodeId(3)), "abef", "a second deletes");
    assert_eq!(d.app.caret(), Some(at(ROOT, 2)));
    set_text(&mut d, r#"["ab", {bold "cd"}, "ef"]"#);
    d.app.set_caret_pos(at(ROOT, 2));
    d.app.delete_forward();
    assert_eq!(
        d.app.selected_text().as_deref(),
        Some("cd"),
        "Delete selects too"
    );
    let s = span(&d, 0);
    for (pos, forward, want, caret) in [
        (at(s, 1), false, "ab{d}ef", at(s, 0)),
        (at(s, 0), true, "ab{}ef", at(s, 0)),
        (at(s, 0), false, "abef", at(ROOT, 2)),
    ] {
        d.app.set_caret_pos(pos);
        if forward {
            d.app.delete_forward();
        } else {
            d.app.delete_backward();
        }
        assert_eq!(shape(&d, NodeId(3)), want, "at {pos:?}");
        assert_eq!(d.app.caret(), Some(caret), "at {pos:?}");
    }
    // Delete at a span's end unwraps it, the caret after its characters.
    set_text(&mut d, r#"["ab", {bold "cd"}, "ef"]"#);
    let s = span(&d, 0);
    d.app.set_caret_pos(at(s, 2));
    d.app.delete_forward();
    assert_eq!(shape(&d, NodeId(3)), "abcdef");
    assert_eq!(d.app.caret(), Some(at(ROOT, 4)));
    // At the root's start and end the keys do nothing.
    d.app.set_caret_pos(at(ROOT, 0));
    d.app.delete_backward();
    d.app.move_caret(CaretMove::TextEnd, false);
    d.app.delete_forward();
    assert_eq!(shape(&d, NodeId(3)), "abcdef");
}

/// The caret positions of `design@rootpoint-app@boundary-caret-positions`, mapped to the layout
/// as `design@rootpoint-app@edit-in-place` says: over generated trees, Right walks from the text's
/// start to its end and Left back, each caret position visited once each way, and each drawn on its
/// glyph's boundary within half a pixel at three sizes of the view, but inside a ligature, which has
/// none between its characters. The trees hold runs of ASCII letters and spaces, so every offset is
/// a caret position, no character carrying a mark; "ff" is a ligature of the built-in Noto Sans,
/// which the first run of this test met, and `the_walk_crosses_a_ligature` walks one each time.
/// The spans hold one item or more, nested three deep at most, since the walk unwraps an empty span
/// it leaves.
mod caret_walk {
    use super::*;
    use proptest::prelude::*;

    fn tree(depth: u32) -> BoxedStrategy<String> {
        let chars = "[a-z ]{1,4}".prop_map(|s| format!("{s:?}"));
        if depth == 0 {
            return chars.boxed();
        }
        let span = (
            prop::sample::select(vec!["bold", "italic", "size=70"]),
            prop::collection::vec(tree(depth - 1), 1..4),
        )
            .prop_map(|(style, items)| format!("{{{style} {}}}", items.join(", ")));
        prop_oneof![chars, span].boxed()
    }

    fn root() -> impl Strategy<Value = String> {
        prop::collection::vec(tree(3), 1..6).prop_map(|items| format!("[{}]", items.join(", ")))
    }

    /// Every caret position of a text node and of the spans below it.
    fn positions(node: TextNodeRef, text: &TextNode, out: &mut Vec<CaretPos>) {
        let mut offset = 0;
        out.push(at(node, 0));
        for item in &text.items {
            match item {
                TextItem::Chars(s) => {
                    for _ in s.chars() {
                        offset += 1;
                        out.push(at(node, offset));
                    }
                }
                TextItem::Span(span) => {
                    positions(TextNodeRef::Span(span.id), &span.text, out);
                    offset += 1;
                    out.push(at(node, offset));
                }
            }
        }
    }

    /// Walk the caret with `m` until it stops, from where it is, checking each position's drawing.
    fn walk(d: &mut Driver, layout: &Layout, ppp: f32, m: CaretMove) -> Vec<CaretPos> {
        let glyphs = glyph_edges(d);
        let mut seen = vec![d.app.caret().unwrap()];
        loop {
            let index = d.app.caret_index().unwrap();
            let caret = d.app.caret_rect(layout).unwrap();
            // The left edge of the cluster that starts at the caret's byte index, or, at the text's
            // end, the last one's right edge; a caret at a soft line break stands before the next
            // line's first cluster. Inside a ligature, no glyph boundary is there to check.
            let want = match glyphs.iter().find(|g| g.0 == index) {
                Some(g) if g.4 => None,
                Some(g) => Some(g.1),
                None => glyphs.iter().find(|g| g.3 == index).map(|g| g.2),
            };
            if let Some(x) = want {
                let off = (caret.min.x - layout.to_points(x, 0.0).x).abs() * ppp;
                assert!(off <= 0.5, "byte {index}: {off} px");
            }
            d.app.move_caret(m, false);
            let now = d.app.caret().unwrap();
            if Some(&now) == seen.last() {
                return seen;
            }
            seen.push(now);
        }
    }

    /// Each cluster the render lays out for n3: the byte it starts at, its left and right x in
    /// slide units, the byte it ends at, and whether it is a character of a ligature after its
    /// first, which has no glyph boundary of its own: Parley 0.12.0 gives each character of a
    /// ligature a cluster, with a share of the ligature's advance.
    fn glyph_edges(d: &Driver) -> Vec<(usize, f64, f64, usize, bool)> {
        let doc = d.app.session().document();
        let slide = d.app.shown_slide().unwrap();
        let text = area_text(doc, slide, NodeId(3), &mut FontSet::new()).unwrap();
        let mut out = Vec::new();
        for line in text.layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(run) = item {
                    let mut x = text.origin.0 + f64::from(run.offset());
                    for c in run.run().visual_clusters() {
                        let r = c.text_range();
                        let end = x + f64::from(c.advance());
                        out.push((r.start, x, end, r.end, c.is_ligature_continuation()));
                        x = end;
                    }
                }
            }
        }
        out
    }

    /// The walk over a ligature: each of its characters is a caret position, and the caret is
    /// checked on the ligature's left edge and after it.
    #[test]
    fn the_walk_crosses_a_ligature() {
        let tree = r#"["off", {bold "ffa"}, "b"]"#;
        let mut d = edit_tree(tree);
        let size = Vec2::new(1000.0, 700.0);
        let doc = d.app.session().document();
        let slide = d.app.shown_slide().unwrap();
        let layout = Layout::new(
            doc,
            slide,
            Rect::from_min_size(Pos2::ZERO, size),
            [1000, 700],
        )
        .unwrap();
        assert!(
            glyph_edges(&d).iter().any(|g| g.4),
            "the text holds a ligature"
        );
        let mut all = Vec::new();
        positions(ROOT, &content(&d, NodeId(3)).root, &mut all);
        d.app.move_caret(CaretMove::TextStart, false);
        let right = walk(&mut d, &layout, 1.0, CaretMove::Right);
        assert_eq!(right, all);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(24))]

        #[test]
        fn every_caret_position_is_visited_once_each_way_and_drawn_on_its_glyph(tree in root()) {
            for (size, ppp) in [
                (Vec2::new(1000.0, 700.0), 1.0),
                (Vec2::new(1400.0, 900.0), 1.5),
                (Vec2::new(700.0, 500.0), 2.0),
            ] {
                let mut d = edit_tree(&tree);
                let doc = d.app.session().document();
                let slide = d.app.shown_slide().unwrap();
                let pixels = [(size.x * ppp) as usize, (size.y * ppp) as usize];
                let layout = Layout::new(doc, slide, Rect::from_min_size(Pos2::ZERO, size), pixels)
                    .unwrap();
                let mut all = Vec::new();
                positions(ROOT, &content(&d, NodeId(3)).root, &mut all);
                d.app.move_caret(CaretMove::TextStart, false);
                let right = walk(&mut d, &layout, ppp, CaretMove::Right);
                let left = walk(&mut d, &layout, ppp, CaretMove::Left);
                let sorted = |mut v: Vec<CaretPos>| {
                    v.sort_by_key(|p| (format!("{:?}", p.node), p.offset));
                    v
                };
                prop_assert_eq!(right.len(), all.len(), "Right visits each once: {}", tree);
                prop_assert_eq!(left.len(), all.len(), "Left visits each once: {}", tree);
                let all = sorted(all);
                prop_assert_eq!(sorted(right.clone()), all.clone());
                prop_assert_eq!(sorted(left.clone()), all);
                let mut back = left;
                back.reverse();
                prop_assert_eq!(right, back, "Left walks Right's path back");
            }
        }
    }
}

/// The small commands of `design@rootpoint-protocol@text-edit-commands`, for a composition: one
/// begun with the caret inside a nested span commits into that span, and one `undo` removes the
/// stay's whole change.
#[test]
fn a_composition_inside_a_nested_span_commits_there_in_one_undo_step() {
    let mut d = edit_tree(r#"["ab", {bold "c", {italic "de"}, "f"}, "g"]"#);
    let before = content(&d, NodeId(3));
    let inner = span(&d, 1);
    d.app.set_caret_pos(at(inner, 1));
    let preedit = |t: &str| {
        Event::Ime(ImeEvent::Preedit {
            text: t.to_owned(),
            active_range_chars: None,
        })
    };
    d.frame(vec![preedit("´")]);
    assert_eq!(
        shape(&d, NodeId(3)),
        "ab{c{d´e}f}g",
        "the composing text stands in the span"
    );
    d.frame(vec![preedit("é")]);
    d.frame(vec![Event::Ime(ImeEvent::Commit("é".to_owned()))]);
    assert_eq!(shape(&d, NodeId(3)), "ab{c{dée}f}g");
    assert_eq!(d.app.caret(), Some(at(inner, 2)));
    d.key(Key::Escape);
    assert!(matches!(d.app.run(Command::Undo), Reply::Ok(_)));
    assert_eq!(
        content(&d, NodeId(3)),
        before,
        "one undo removes the stay's change"
    );
}

/// Shift with an arrow out of an empty span unwraps it, and the selection's anchor, which stood in
/// it, stands at its place in the parent, so typing after it goes in.
#[test]
fn an_anchor_left_in_an_unwrapped_span_stands_at_its_place() {
    let mut d = edit_tree(r#"["ab", {bold "c"}, "d"]"#);
    let s = span(&d, 0);
    d.app.set_caret_pos(at(s, 1));
    d.app.delete_backward();
    d.app.move_caret(CaretMove::Right, true);
    assert_eq!(shape(&d, NodeId(3)), "abd");
    assert_eq!(d.app.caret(), Some(at(ROOT, 2)));
    d.app.type_str("X");
    assert_eq!(shape(&d, NodeId(3)), "abXd");
}

/// A click during a composition moves the caret and leaves the composing text where it was typed,
/// so the commit replaces that text and nothing of the clicked node.
#[test]
fn a_commit_after_the_caret_moved_replaces_the_composing_text() {
    let mut d = edit_tree(r#"["ab", {bold "cd"}, "ef"]"#);
    d.app.set_caret_pos(at(ROOT, 0));
    d.app.set_preedit("xy");
    assert_eq!(shape(&d, NodeId(3)), "xyab{cd}ef");
    let s = span(&d, 0);
    d.app.set_caret_pos(at(s, 1));
    d.app.commit_ime("Z");
    assert_eq!(shape(&d, NodeId(3)), "Zab{cd}ef");
}
