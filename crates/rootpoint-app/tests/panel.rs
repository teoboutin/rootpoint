//! The side panel: its width is the same in every state of its content, the instrument of AC2 of
//! the milestone on the conveniences of the editor.

use eframe::egui::{Context, Pos2, RawInput, Rect, Vec2};
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
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
        ..Default::default()
    };
    let mut out = ctx.run_ui(input, |ui| app.frame(ui));
    // No painter applies the texture deltas, and egui refuses to drop them unapplied.
    out.textures_delta.clear();
}

/// The side panel's width, two frames after a change, as a test that reads a place waits; and that
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
    // A tree deeper than the 6 levels AC2 names, 9: n5, at depth 3, split six times, each split of
    // the deepest area wrapping it in a split of the other axis.
    let mut node = "n5".to_owned();
    for depth in 0..6 {
        let axis = if depth % 2 == 0 { "stacked" } else { "beside" };
        node = new_area(&ok(&mut app, &format!("split {node} {axis} 0.5")));
    }
    ok(&mut app, &format!("select {node}"));
    widths.push(("a tree 9 levels deep", width(&ctx, &mut app)));
    let first = widths[0].1;
    for (state, w) in &widths {
        assert_eq!(*w, first, "{state}: {widths:?}");
    }
}
