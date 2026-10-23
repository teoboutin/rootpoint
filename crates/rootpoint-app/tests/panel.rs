//! The side panel: its width is the same in every state of its content,
//! `design@rootpoint-app@sidebar-width`, the instrument of AC2 of
//! `milestone@plans@editor-conveniences`.

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
    // A tree deeper than the 6 levels AC2 of `milestone@plans@editor-conveniences` names, 9: n5,
    // at depth 3, split six times, each split of the deepest area wrapping it in a split of the
    // other axis.
    let mut node = "n5".to_owned();
    for depth in 0..6 {
        let axis = if depth % 2 == 0 { "stacked" } else { "beside" };
        node = new_area(&ok(&mut app, &format!("split {node} {axis} 0.5")));
    }
    ok(&mut app, &format!("select {node}"));
    widths.push(("a tree 9 levels deep", width(&ctx, &mut app)));
    frame(&ctx, &mut app);
    click(&ctx, &mut app, "Styles");
    click(&ctx, &mut app, "style Subtitle");
    assert_eq!(app.selected_style(), Some("Subtitle"));
    widths.push(("the Styles tab, a style selected", width(&ctx, &mut app)));
    let first = widths[0].1;
    for (state, w) in &widths {
        assert_eq!(*w, first, "{state}: {widths:?}");
    }
}
