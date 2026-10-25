//! The side panel: its width is the same in every state of its content,
//! `design@rootpoint-app@sidebar-width`, the Assets tab's included; and the Styles tab, the only part of the window that
//! changes a named style, with the dialog of a style's removal; and the palette of a formula
//! edited.

use eframe::egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use rootpoint_app::{App, MemoryFiles};
use rootpoint_protocol::{Reply, parse_command};

fn ok(app: &mut App, line: &str) -> String {
    match app.run(parse_command(line).unwrap()) {
        Reply::Ok(text) => text,
        Reply::Err(reason) => panic!("{line}: {reason}"),
    }
}

/// One frame of a window of 1280×800 points.
fn frame(ctx: &Context, app: &mut App) {
    frame_with(ctx, app, Vec::new());
}

/// One frame of a window of 1280×800 points, with input events.
fn frame_with(ctx: &Context, app: &mut App, events: Vec<Event>) {
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
        events,
        ..Default::default()
    };
    let mut out = ctx.run_ui(input, |ui| app.frame(ui));
    // No painter applies the texture deltas, and egui refuses to drop them unapplied.
    out.textures_delta.clear();
}

/// The side panel's width two frames after a change: egui sizes a panel from what its content took
/// in the frame before, so a width that followed the content would change in the second; and that
/// its content fits the scroll area's width, its rows wrapped or cut rather than scrolling sideways.
fn width(ctx: &Context, app: &mut App) -> f32 {
    frame(ctx, app);
    frame(ctx, app);
    let (content, area) = app.side_panel_content_width().unwrap();
    assert!(
        content <= area,
        "the content, {content} points, overflows {area}"
    );
    app.side_panel_rect()
        .expect("the side panel is shown")
        .width()
}

/// A click of the primary button on the button or the tab the last frame placed under `label`, then
/// a frame to show its effect.
fn click(ctx: &Context, app: &mut App, label: &str) {
    let at = app
        .button_rect(label)
        .unwrap_or_else(|| panic!("no button {label}"))
        .center();
    for pressed in [true, false] {
        frame_with(
            ctx,
            app,
            vec![
                Event::PointerMoved(at),
                Event::PointerButton {
                    pos: at,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
    frame(ctx, app);
}

/// The palette lists every command of the math crate's table once, in one category, and no
/// category is empty; every structure is one atom.
#[test]
fn the_palette_lists_every_command_once() {
    use rootpoint_app::palette::{Category, of_category, structures};
    let listed: usize = Category::ALL.iter().map(|c| of_category(*c).count()).sum();
    assert_eq!(listed, rootpoint_math::commands().len());
    for c in Category::ALL {
        assert!(of_category(c).next().is_some(), "{c:?} is empty");
    }
    assert!(structures().len() > 30);
}

/// While a formula is edited, the Selection tab's palette inserts at the caret: a structure, with
/// the caret in its first cell, then a symbol of a category opened by its header; the controls of
/// a matrix are enabled only while a matrix holds the caret.
#[test]
fn the_palette_inserts_at_the_caret_in_a_formula() {
    let mut app = App::new(Box::new(MemoryFiles::default()));
    let ctx = Context::default();
    ok(&mut app, "sample");
    ok(&mut app, "select n3");
    ok(&mut app, r"equation n3 $x$");
    frame(&ctx, &mut app);
    let layout = app.layout().unwrap().clone();
    let at = layout.rect(rootpoint_model::NodeId(3)).unwrap().center();
    app.edit_at(rootpoint_model::NodeId(3), at, &layout);
    app.move_caret(rootpoint_app::editing::CaretMove::TextEnd, false);
    frame(&ctx, &mut app);
    frame(&ctx, &mut app);
    assert!(!app.caret_in_matrix());
    click(&ctx, &mut app, "a/b");
    assert_eq!(app.math_caret().map(|p| p.steps), Some(vec![(1, 0)]));
    click(&ctx, &mut app, "Relations");
    // The category's first command, which stands inside the panel with no scroll.
    let first = rootpoint_app::palette::of_category(rootpoint_app::palette::Category::Relations)
        .next()
        .unwrap();
    let label = format!("\\{}", first.name);
    let panel = app.side_panel_rect().unwrap();
    assert!(panel.contains(app.button_rect(&label).unwrap().center()));
    click(&ctx, &mut app, &label);
    let latex = |app: &App| app.edited_formula().map(|f| f.to_latex());
    let after = latex(&app).unwrap();
    assert!(after.contains(&label), "{after}");
    click(&ctx, &mut app, "pmatrix");
    assert!(app.caret_in_matrix());
    click(&ctx, &mut app, "row: append");
    assert_eq!(
        latex(&app),
        Some(after.replace("}{}", r" \begin{pmatrix} & \\ & \\ & \end{pmatrix}}{}"))
    );
}

/// Removing a style that texts name opens the dialog and sends nothing; its "Remove style" with the
/// custom copy, chosen first, gives each user Body's values, and one undo restores Body.
#[test]
fn removing_a_style_in_use_asks_for_a_replacement() {
    let mut app = App::new(Box::new(MemoryFiles::default()));
    let ctx = Context::default();
    ok(&mut app, "sample");
    frame(&ctx, &mut app);
    frame(&ctx, &mut app);
    let before = app.session().document().clone();
    click(&ctx, &mut app, "Styles");
    click(&ctx, &mut app, "style Body");
    assert_eq!(app.selected_style(), Some("Body"));
    click(&ctx, &mut app, "Remove");
    let removal = app.removal().expect("the dialog is open");
    assert_eq!(
        (removal.name.as_str(), removal.into.as_deref()),
        ("Body", None)
    );
    assert_eq!(
        app.session().document(),
        &before,
        "the dialog sent a command"
    );
    click(&ctx, &mut app, "Remove style");
    assert!(app.removal().is_none());
    let doc = app.session().document();
    assert!(!doc.styles.contains_key("Body"));
    for id in [3, 5, 9] {
        match doc.node(rootpoint_model::NodeId(id)) {
            Some(rootpoint_model::Node::Area(a)) => match &a.content {
                rootpoint_model::Content::Text(t) => assert_eq!(
                    t.style,
                    rootpoint_model::StyleChoice::Custom(before.styles["Body"].clone()),
                    "n{id}"
                ),
                other => panic!("n{id} holds {other:?}"),
            },
            other => panic!("n{id} is {other:?}"),
        }
    }
    ok(&mut app, "undo");
    assert_eq!(app.session().document().slides, before.slides);
    assert_eq!(app.session().document().styles, before.styles);
}

/// The style of the text of area `id`.
fn style_of(app: &App, id: u64) -> rootpoint_model::StyleChoice {
    match app.session().document().node(rootpoint_model::NodeId(id)) {
        Some(rootpoint_model::Node::Area(a)) => match &a.content {
            rootpoint_model::Content::Text(t) => t.style.clone(),
            other => panic!("n{id} holds {other:?}"),
        },
        other => panic!("n{id} is {other:?}"),
    }
}

/// A click on "bold" in the Selection tab makes the text custom and leaves its named style as it
/// was; the same click in the Styles tab changes the named style, and its texts keep naming it.
#[test]
fn only_the_styles_tab_changes_a_named_style() {
    let mut app = App::new(Box::new(MemoryFiles::default()));
    let ctx = Context::default();
    ok(&mut app, "sample");
    ok(&mut app, "select n3");
    frame(&ctx, &mut app);
    frame(&ctx, &mut app);
    let body = app.session().document().styles["Body"].clone();
    click(&ctx, &mut app, "bold");
    assert_eq!(app.session().document().styles["Body"], body);
    assert_eq!(
        style_of(&app, 3),
        rootpoint_model::StyleChoice::Custom(rootpoint_model::TextStyle {
            bold: true,
            ..body.clone()
        })
    );
    click(&ctx, &mut app, "Styles");
    click(&ctx, &mut app, "style Body");
    click(&ctx, &mut app, "bold");
    assert!(app.session().document().styles["Body"].bold);
    assert_eq!(
        style_of(&app, 5),
        rootpoint_model::StyleChoice::Named("Body".to_owned())
    );
}

/// The dialog offers no style that would draw with a stand-in: here Legend, whose built-in family
/// a file edited by hand names, and which the binary does not carry.
#[test]
fn the_dialog_offers_only_styles_a_text_may_name() {
    /// One file, the presentation, which the test writes and the app reads.
    struct Deck(String);
    impl rootpoint_protocol::Files for Deck {
        fn read(&mut self, _: &str) -> Result<Vec<u8>, String> {
            Ok(self.0.clone().into_bytes())
        }
        fn write(&mut self, path: &str, _: &[u8]) -> Result<(), String> {
            Err(format!("{path}: read only"))
        }
        fn exists(&mut self, _: &str) -> Result<bool, String> {
            Ok(true)
        }
        fn canonical(&mut self, path: &str) -> Result<String, String> {
            Ok(path.to_owned())
        }
    }
    let mut sample = App::new(Box::new(MemoryFiles::default()));
    ok(&mut sample, "sample");
    let text = rootpoint_model::save_ron(sample.session().document());
    let legend = "        \"Legend\": (\n            font: Builtin(\"sans\"),";
    let edited = text.replacen(
        legend,
        "        \"Legend\": (\n            font: Builtin(\"serif\"),",
        1,
    );
    assert_ne!(edited, text, "the saved sample changed shape");
    let mut app = App::new(Box::new(Deck(edited)));
    ok(&mut app, "open deck");
    assert_eq!(app.replacements("Body"), ["Subtitle", "Title"]);
}

/// The new area of a split, read from its reply.
fn new_area(reply: &str) -> String {
    reply.split_whitespace().next().unwrap()["new=".len()..].to_owned()
}

#[test]
fn the_side_panel_s_width_is_the_same_in_every_state_of_its_content() {
    let mut app = App::new(Box::new(MemoryFiles::default()));
    let ctx = Context::default();
    ok(&mut app, "sample");
    let mut widths = Vec::new();
    ok(&mut app, "select s1");
    widths.push(("the slide alone", width(&ctx, &mut app)));
    ok(&mut app, "select n3");
    widths.push(("a text area", width(&ctx, &mut app)));
    ok(&mut app, "select n14");
    widths.push(("an image area", width(&ctx, &mut app)));
    let long = "x".repeat(200);
    ok(&mut app, &format!("text n3 \"{long}\""));
    ok(&mut app, "select n3");
    widths.push(("a text of 200 characters", width(&ctx, &mut app)));
    // A tree 9 levels deep: n5, at depth 3, split six times, each split of the deepest area
    // wrapping it in a split of the other axis.
    let mut node = "n5".to_owned();
    for depth in 0..6 {
        let axis = if depth % 2 == 0 { "stacked" } else { "beside" };
        node = new_area(&ok(&mut app, &format!("split {node} {axis} 0.5")));
    }
    ok(&mut app, &format!("select {node}"));
    widths.push(("a tree 9 levels deep", width(&ctx, &mut app)));
    // A text edited with the caret in a nested span: the ancestor row and the controls at the
    // caret.
    ok(&mut app, "select n26");
    frame(&ctx, &mut app);
    let layout = app.layout().unwrap().clone();
    let at = layout.rect(rootpoint_model::NodeId(26)).unwrap().center();
    app.edit_at(rootpoint_model::NodeId(26), at, &layout);
    app.set_caret_pos(rootpoint_model::CaretPos {
        node: rootpoint_model::TextNodeRef::Span(rootpoint_model::SpanId(32)),
        offset: 1,
    });
    widths.push(("a text edited in a nested span", width(&ctx, &mut app)));
    assert!(
        app.button_rect("t32").is_some(),
        "the ancestor row lists t32"
    );
    // A formula edited, with the palette's structures and a category opened.
    app.stop_editing();
    ok(&mut app, "select n3");
    ok(&mut app, r"equation n3 $x$");
    frame(&ctx, &mut app);
    let layout = app.layout().unwrap().clone();
    let at = layout.rect(rootpoint_model::NodeId(3)).unwrap().center();
    app.edit_at(rootpoint_model::NodeId(3), at, &layout);
    frame(&ctx, &mut app);
    frame(&ctx, &mut app);
    click(&ctx, &mut app, "Arrows");
    widths.push(("a formula edited, with its palette", width(&ctx, &mut app)));
    app.stop_editing();
    frame(&ctx, &mut app);
    click(&ctx, &mut app, "Styles");
    click(&ctx, &mut app, "style Subtitle");
    assert_eq!(app.selected_style(), Some("Subtitle"));
    widths.push(("the Styles tab, a style selected", width(&ctx, &mut app)));
    click(&ctx, &mut app, "span style Key");
    assert_eq!(app.selected_span_style(), Some("Key"));
    widths.push((
        "the Styles tab, a span style selected",
        width(&ctx, &mut app),
    ));
    // The Assets tab, an asset of a long path selected, its thumbnail drawn.
    ok(
        &mut app,
        &format!("asset-move a19 {}/splits.png", "folder".repeat(20)),
    );
    click(&ctx, &mut app, "Assets");
    click(&ctx, &mut app, "asset a19");
    assert_eq!(app.selected_asset(), Some(rootpoint_model::AssetId(19)));
    widths.push(("the Assets tab, an asset selected", width(&ctx, &mut app)));
    let first = widths[0].1;
    for (state, w) in &widths {
        assert_eq!(*w, first, "{state}: {widths:?}");
    }
}

/// A use's button of the Assets tab selects its node and shows its slide.
#[test]
fn a_use_s_button_selects_its_node_and_shows_its_slide() {
    let mut app = App::new(Box::new(MemoryFiles::default()));
    let ctx = Context::default();
    ok(&mut app, "sample");
    frame(&ctx, &mut app);
    click(&ctx, &mut app, "Assets");
    click(&ctx, &mut app, "asset a19");
    click(&ctx, &mut app, "use n17");
    assert_eq!(
        app.session().view().selected,
        Some(rootpoint_model::NodeId(17))
    );
    let slide = app
        .session()
        .document()
        .slides
        .iter()
        .find(|s| s.root.find(rootpoint_model::NodeId(17)).is_some())
        .unwrap()
        .id;
    assert_eq!(app.shown_slide(), Some(slide));
}

/// A span style's use of an asset is a button of the Assets tab that shows the Styles tab with
/// the span style selected, as a style's use does.
#[test]
fn a_span_style_s_use_selects_it_in_the_styles_tab() {
    let mut app = App::new(Box::new(MemoryFiles::default()));
    let ctx = Context::default();
    ok(&mut app, "sample");
    ok(&mut app, "span-style set \"Key\" font=a19");
    frame(&ctx, &mut app);
    click(&ctx, &mut app, "Assets");
    click(&ctx, &mut app, "asset a19");
    click(&ctx, &mut app, "use span style Key");
    assert_eq!(app.selected_span_style(), Some("Key"));
    assert!(
        app.button_rect("span style Key").is_some(),
        "the Styles tab lists the span style"
    );
}

/// A drag of an asset's row onto a folder's row moves the asset into that folder, by
/// `asset-move`.
#[test]
fn an_asset_dragged_onto_a_folder_moves_there() {
    let mut app = App::new(Box::new(MemoryFiles::default()));
    let ctx = Context::default();
    ok(&mut app, "sample");
    ok(&mut app, "asset-move a19 img/splits.png");
    frame(&ctx, &mut app);
    click(&ctx, &mut app, "Assets");
    let from = app
        .button_rect("asset a19")
        .expect("the asset's row")
        .center();
    let to = app.button_rect("folder ").expect("the root's row").center();
    let button = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame_with(
        &ctx,
        &mut app,
        vec![Event::PointerMoved(from), button(from, true)],
    );
    for i in 1..=10 {
        let at = from + (to - from) * (i as f32 / 10.0);
        frame_with(&ctx, &mut app, vec![Event::PointerMoved(at)]);
    }
    frame_with(&ctx, &mut app, vec![button(to, false)]);
    frame(&ctx, &mut app);
    assert_eq!(
        app.session().document().assets[&rootpoint_model::AssetId(19)].path,
        "splits.png"
    );
}
