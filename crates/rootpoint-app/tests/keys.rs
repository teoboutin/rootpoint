//! The math keys: each sequence of the table runs its action through `App::frame`, given as the
//! events egui gives for it, a chord, then the text of each key after it; the expected actions are
//! restated here from the milestone's table "A LyX math key, to the action it runs", so a table
//! with one binding swapped fails.

use eframe::egui::{Context, Event, Key, Modifiers, Pos2, RawInput, Rect, Vec2};
use rootpoint_app::{App, MemoryFiles};
use rootpoint_math::{CellPath, Formula, MatrixEdit};
use rootpoint_model::{Content, Node, NodeId};
use rootpoint_protocol::{Reply, parse_command};

fn ok(app: &mut App, line: &str) {
    match app.run(parse_command(line).unwrap()) {
        Reply::Ok(_) => {}
        Reply::Err(reason) => panic!("{line}: {reason}"),
    }
}

/// One frame of a window of 1280×800 points, with input events.
fn frame(ctx: &Context, app: &mut App, events: Vec<Event>, modifiers: Modifiers) {
    let mut events = events;
    events.insert(0, Event::ModifiersChanged(modifiers));
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
        events,
        ..Default::default()
    };
    let mut out = ctx.run_ui(input, |ui| app.frame(ui));
    out.textures_delta.clear();
}

fn key(key: Key, modifiers: Modifiers) -> Vec<Event> {
    [true, false]
        .into_iter()
        .map(|pressed| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        })
        .collect()
}

/// The events of a character typed with no modifier but Shift: its key, when it has one, then its
/// text.
fn typed(ch: char) -> (Vec<Event>, Modifiers) {
    let modifiers = if ch.is_ascii_uppercase() {
        Modifiers::SHIFT
    } else {
        Modifiers::NONE
    };
    let mut events = Key::from_name(&ch.to_ascii_uppercase().to_string())
        .map(|k| key(k, modifiers))
        .unwrap_or_default();
    events.insert(1.min(events.len()), Event::Text(ch.to_string()));
    (events, modifiers)
}

/// Alt+M then the characters of `after`, each in a frame of its own; Alt+M's own frame carries the
/// `m` that egui can give with it.
fn alt_m(ctx: &Context, app: &mut App, after: &str) {
    let mut events = key(Key::M, Modifiers::ALT);
    events.insert(1, Event::Text("m".to_owned()));
    frame(ctx, app, events, Modifiers::ALT);
    for ch in after.chars() {
        let (events, modifiers) = typed(ch);
        frame(ctx, app, events, modifiers);
    }
}

const MATRIX: &str = r"\begin{matrix} a & b \\ c & d \end{matrix}";

/// An app editing n3, an equation holding `MATRIX`, its caret after the `a`.
fn editing_matrix() -> (App, Context, CellPath) {
    let mut app = App::new(Box::new(MemoryFiles::default()));
    let ctx = Context::default();
    ok(&mut app, "sample");
    ok(&mut app, "select n3");
    ok(&mut app, &format!("equation n3 ${MATRIX}$"));
    frame(&ctx, &mut app, Vec::new(), Modifiers::NONE);
    let layout = app.layout().unwrap().clone();
    let at = layout.rect(NodeId(3)).unwrap().center();
    app.edit_at(NodeId(3), at, &layout);
    let caret = CellPath {
        steps: vec![(0, 0)],
        offset: 1,
        ..CellPath::default()
    };
    app.set_math_caret(caret.clone());
    frame(&ctx, &mut app, Vec::new(), Modifiers::NONE);
    (app, ctx, caret)
}

fn formula_of(app: &App) -> Formula {
    match app.session().document().node(NodeId(3)) {
        Some(Node::Area(a)) => match &a.content {
            Content::Equation(e) => e.formula.clone(),
            other => panic!("n3 holds {other:?}"),
        },
        _ => panic!("no n3"),
    }
}

/// What a sequence is expected to do to `MATRIX` at the caret after its `a`.
enum Expect {
    Insert(&'static str),
    Script(bool),
    Matrix(bool, MatrixEdit),
}

/// The keys after Alt+M, by the milestone's table and section 1 of the survey it cites.
const AFTER_ALT_M: &[(&str, Expect)] = &[
    ("f", Expect::Insert(r"\frac{}{}")),
    ("s", Expect::Insert(r"\sqrt{}")),
    ("r", Expect::Insert(r"\sqrt[]{}")),
    ("x", Expect::Script(false)),
    ("e", Expect::Script(true)),
    ("u", Expect::Insert(r"\sum")),
    ("i", Expect::Insert(r"\int")),
    ("y", Expect::Insert(r"\oint")),
    ("p", Expect::Insert(r"\partial")),
    ("8", Expect::Insert(r"\infty")),
    ("+", Expect::Insert(r"\pm")),
    ("=", Expect::Insert(r"\neq")),
    ("\"", Expect::Insert(r"\ddot{}")),
    ("h", Expect::Insert(r"\hat{}")),
    ("\\", Expect::Insert(r"\grave{}")),
    ("/", Expect::Insert(r"\acute{}")),
    ("&", Expect::Insert(r"\tilde{}")),
    ("-", Expect::Insert(r"\bar{}")),
    (".", Expect::Insert(r"\dot{}")),
    ("V", Expect::Insert(r"\check{}")),
    ("U", Expect::Insert(r"\breve{}")),
    ("v", Expect::Insert(r"\vec{}")),
    ("b", Expect::Insert(r"\overline{}")),
    ("_", Expect::Insert(r"\underline{}")),
    ("'", Expect::Insert(r"\prime")),
    ("(", Expect::Insert(r"\left(\right)")),
    ("[", Expect::Insert(r"\left[\right]")),
    ("{", Expect::Insert(r"\left\{\right\}")),
    ("<", Expect::Insert(r"\left\langle\right\rangle")),
    (">", Expect::Insert(r"\left\rangle\right\langle")),
    ("|", Expect::Insert(r"\left|\right|")),
    (" ", Expect::Insert(r"\,")),
    ("ci", Expect::Matrix(false, MatrixEdit::Append)),
    ("cd", Expect::Matrix(false, MatrixEdit::Delete)),
    ("cc", Expect::Matrix(false, MatrixEdit::Copy)),
    ("cs", Expect::Matrix(false, MatrixEdit::Swap)),
    ("wi", Expect::Matrix(true, MatrixEdit::Append)),
    ("wd", Expect::Matrix(true, MatrixEdit::Delete)),
    ("wc", Expect::Matrix(true, MatrixEdit::Copy)),
    ("ws", Expect::Matrix(true, MatrixEdit::Swap)),
];

/// Alt+M G then a letter, by section 1 of the survey: LyX's Greek, with its oddities.
const GREEK: &str = "a alpha b beta c chi d delta e epsilon f phi g gamma h eta i iota j varphi \
    k kappa l lambda m mu n nu o omega p pi q vartheta r rho s sigma t tau u upsilon v theta \
    w omega x xi y psi z zeta D Delta E varepsilon F Phi G Gamma I iota J epsilon L Lambda \
    O Omega P Pi Q vartheta R varrho S Sigma T varsigma U Upsilon V Theta W Omega X Xi Y Psi";

/// The formula `MATRIX` would hold after the expected action, by the math crate's edits.
fn expected(expect: &Expect, caret: &CellPath) -> String {
    let mut f = Formula::parse(MATRIX).unwrap();
    match expect {
        Expect::Insert(latex) => {
            let atom = Formula::parse(latex).unwrap().rows[0][0][0].clone();
            rootpoint_math::insert(&mut f, caret, atom).unwrap();
        }
        Expect::Script(sup) => {
            rootpoint_math::script(&mut f, caret, *sup).unwrap();
        }
        Expect::Matrix(rows, edit) => {
            rootpoint_math::edit_matrix(&mut f, caret, *rows, *edit).unwrap();
        }
    }
    f.to_latex()
}

#[test]
fn each_key_after_alt_m_runs_its_action() {
    for (after, expect) in AFTER_ALT_M {
        let (mut app, ctx, caret) = editing_matrix();
        alt_m(&ctx, &mut app, after);
        assert_eq!(
            formula_of(&app).to_latex(),
            expected(expect, &caret),
            "Alt+M {after:?}"
        );
    }
}

#[test]
fn each_letter_after_alt_m_g_inserts_its_greek_letter() {
    let words: Vec<&str> = GREEK.split_whitespace().collect();
    assert_eq!(words.len(), 2 * (26 + 18));
    for pair in words.chunks(2) {
        let (mut app, ctx, caret) = editing_matrix();
        alt_m(&ctx, &mut app, &format!("g{}", pair[0]));
        let latex: &'static str = Box::leak(format!("\\{}", pair[1]).into_boxed_str());
        assert_eq!(
            formula_of(&app).to_latex(),
            expected(&Expect::Insert(latex), &caret),
            "Alt+M G {}",
            pair[0]
        );
    }
}

/// Alt+M N turns the number of the equation edited off, then on.
#[test]
fn alt_m_n_toggles_the_equation_s_number() {
    let (mut app, ctx, _) = editing_matrix();
    let numbered = |app: &App| match app.session().document().node(NodeId(3)) {
        Some(Node::Area(a)) => matches!(&a.content, Content::Equation(e) if e.numbered),
        _ => false,
    };
    let before = numbered(&app);
    alt_m(&ctx, &mut app, "n");
    assert_eq!(numbered(&app), !before);
    alt_m(&ctx, &mut app, "n");
    assert_eq!(numbered(&app), before);
}

/// A sequence that matches none is dropped with its last key, and Esc drops one that waits; the
/// keys typed after either go to the formula as text.
#[test]
fn an_unbound_sequence_is_dropped() {
    let (mut app, ctx, caret) = editing_matrix();
    alt_m(&ctx, &mut app, "k");
    assert_eq!(
        formula_of(&app).to_latex(),
        Formula::parse(MATRIX).unwrap().to_latex()
    );
    alt_m(&ctx, &mut app, "");
    frame(
        &ctx,
        &mut app,
        key(Key::Escape, Modifiers::NONE),
        Modifiers::NONE,
    );
    assert!(
        app.editing().is_some(),
        "Esc dropped the sequence and kept editing"
    );
    let (events, modifiers) = typed('q');
    frame(&ctx, &mut app, events, modifiers);
    assert_eq!(
        formula_of(&app).to_latex(),
        expected(&Expect::Insert("q"), &caret)
    );
}

/// Ctrl+M inserts a formula at the caret of a text, the caret in it; Alt+M F in a text inserts a
/// formula holding a fraction.
#[test]
fn ctrl_m_and_alt_m_f_insert_a_formula_in_a_text() {
    for (name, keys) in [("Ctrl+M", None), ("Alt+M F", Some("f"))] {
        let mut app = App::new(Box::new(MemoryFiles::default()));
        let ctx = Context::default();
        ok(&mut app, "sample");
        ok(&mut app, "select n3");
        ok(&mut app, r#"text n3 ["ab"]"#);
        frame(&ctx, &mut app, Vec::new(), Modifiers::NONE);
        let layout = app.layout().unwrap().clone();
        let at = layout.rect(NodeId(3)).unwrap().center();
        app.edit_at(NodeId(3), at, &layout);
        match keys {
            None => frame(
                &ctx,
                &mut app,
                key(Key::M, Modifiers::COMMAND),
                Modifiers::COMMAND,
            ),
            Some(after) => alt_m(&ctx, &mut app, after),
        }
        assert!(
            app.math_caret().is_some(),
            "{name}: the caret in the formula"
        );
        let latex = app.edited_formula().map(|f| f.to_latex());
        let want = if keys.is_some() { r"\frac{}{}" } else { "" };
        assert_eq!(latex.as_deref(), Some(want), "{name}");
    }
}

/// Ctrl+Shift+M makes the selected area an unnumbered equation, and Ctrl+Alt+N a numbered one, its
/// text the formula where it reads as one.
#[test]
fn ctrl_shift_m_and_ctrl_alt_n_make_an_equation() {
    for (modifiers, k, numbered) in [
        (Modifiers::COMMAND | Modifiers::SHIFT, Key::M, false),
        (Modifiers::COMMAND | Modifiers::ALT, Key::N, true),
    ] {
        let mut app = App::new(Box::new(MemoryFiles::default()));
        let ctx = Context::default();
        ok(&mut app, "sample");
        ok(&mut app, r#"text n3 ["x+1"]"#);
        ok(&mut app, "select n3");
        frame(&ctx, &mut app, Vec::new(), Modifiers::NONE);
        frame(&ctx, &mut app, key(k, modifiers), modifiers);
        match app.session().document().node(NodeId(3)) {
            Some(Node::Area(a)) => match &a.content {
                Content::Equation(e) => {
                    assert_eq!(e.numbered, numbered, "{k:?}");
                    assert_eq!(e.formula.to_latex(), "x + 1");
                }
                other => panic!("{k:?}: n3 holds {other:?}"),
            },
            _ => panic!("no n3"),
        }
    }
}

/// While a sequence waits, the status line names it; a key that types no text, as Backspace or an
/// arrow, drops it and is dropped with it; and leaving editing drops it, so it acts in no later
/// stay.
#[test]
fn a_waiting_sequence_is_shown_and_dropped_by_a_key_with_no_text_or_by_leaving() {
    let (mut app, ctx, caret) = editing_matrix();
    alt_m(&ctx, &mut app, "");
    assert!(
        app.status().iter().any(|l| l == "Alt+M …"),
        "{:?}",
        app.status()
    );
    alt_m(&ctx, &mut app, "g");
    assert!(
        app.status().iter().any(|l| l == "Alt+M G …"),
        "{:?}",
        app.status()
    );
    frame(
        &ctx,
        &mut app,
        key(Key::Backspace, Modifiers::NONE),
        Modifiers::NONE,
    );
    assert!(app.status().iter().all(|l| !l.ends_with('…')));
    assert_eq!(
        formula_of(&app).to_latex(),
        Formula::parse(MATRIX).unwrap().to_latex(),
        "Backspace dropped"
    );
    let (events, modifiers) = typed('f');
    frame(&ctx, &mut app, events, modifiers);
    assert_eq!(
        formula_of(&app).to_latex(),
        expected(&Expect::Insert("f"), &caret)
    );
    alt_m(&ctx, &mut app, "");
    app.stop_editing();
    let layout = app.layout().unwrap().clone();
    let at = layout.rect(NodeId(3)).unwrap().center();
    app.edit_at(NodeId(3), at, &layout);
    assert!(
        app.status().iter().all(|l| !l.ends_with('…')),
        "the sequence left with the stay"
    );
}

/// Ctrl+Shift+M and Ctrl+Alt+N are matched with their modifiers exactly: with Alt or Shift more,
/// they make no equation.
#[test]
fn ctrl_shift_m_and_ctrl_alt_n_take_no_extra_modifier() {
    for (modifiers, k) in [
        (
            Modifiers::COMMAND | Modifiers::SHIFT | Modifiers::ALT,
            Key::M,
        ),
        (
            Modifiers::COMMAND | Modifiers::ALT | Modifiers::SHIFT,
            Key::N,
        ),
    ] {
        let mut app = App::new(Box::new(MemoryFiles::default()));
        let ctx = Context::default();
        ok(&mut app, "sample");
        ok(&mut app, "select n3");
        frame(&ctx, &mut app, Vec::new(), Modifiers::NONE);
        frame(&ctx, &mut app, key(k, modifiers), modifiers);
        assert!(
            matches!(app.session().document().node(NodeId(3)), Some(Node::Area(a)) if matches!(a.content, Content::Text(_))),
            "{k:?} with {modifiers:?}"
        );
    }
}

/// Down from the bottom row of a formula in a text is the text's Down, as LyX passes a move it
/// cannot make in the formula to the text.
#[test]
fn down_at_a_formula_s_bottom_moves_in_the_text() {
    let mut app = App::new(Box::new(MemoryFiles::default()));
    let ctx = Context::default();
    ok(&mut app, "sample");
    ok(&mut app, "select n3");
    ok(&mut app, r#"text n3 ["ab ", $x$, " cd\nef"]"#);
    frame(&ctx, &mut app, Vec::new(), Modifiers::NONE);
    let layout = app.layout().unwrap().clone();
    let at = layout.rect(NodeId(3)).unwrap().center();
    app.edit_at(NodeId(3), at, &layout);
    app.set_caret_pos(rootpoint_model::CaretPos {
        node: rootpoint_model::TextNodeRef::Root,
        offset: 3,
    });
    app.move_caret(rootpoint_app::editing::CaretMove::Right, false);
    assert!(app.math_caret().is_some());
    frame(
        &ctx,
        &mut app,
        key(Key::ArrowDown, Modifiers::NONE),
        Modifiers::NONE,
    );
    assert_eq!(app.math_caret(), None);
    assert!(app.caret().unwrap().offset > 4, "{:?}", app.caret());
}
