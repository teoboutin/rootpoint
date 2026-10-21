//! Hit testing matches the drawing: through `slide_to_pixels`, at two view sizes, a point selects
//! the area whose rectangle holds it, and a press within 4 points of a divider grabs it, the nearer
//! of two; a pointer over a drop target falls in the zone the hint names; and a snapped line lies
//! on a twelfth of its node or on a divider within 8 points. Claim 2 of
//! `spec@first-prototype@editor-shell`.

use eframe::egui::{Pos2, Rect, Vec2};
use rootpoint_app::hit::{Edge, Layout, MIN_PART, Press, Zone};
use rootpoint_model::{Axis, Document, Node, NodeId, Op, Side, apply, area_rects};
use rootpoint_render::slide_to_pixels;

/// A slide of three areas side by side: halves split at 0.5, then the left half split again
/// beside at 0.99, which the model merges into the one split, so its dividers lie at 950.4 and
/// 960 slide units, 9.6 apart. The split, and the areas left to right.
fn two_close_dividers() -> (Document, NodeId, [NodeId; 3]) {
    let mut doc = Document::default();
    apply(&mut doc, Op::AddSlide { at: 0 }).unwrap();
    let first = doc.slides[0].root.id();
    for at in [0.5, 0.99] {
        apply(
            &mut doc,
            Op::Split {
                node: first,
                axis: Axis::LeftToRight,
                at,
                side: Side::After,
            },
        )
        .unwrap();
    }
    let Node::Split(root) = &doc.slides[0].root else {
        panic!("the root is a split");
    };
    assert_eq!(root.dividers, [0.495, 0.5]);
    let ids = [
        root.children[0].id(),
        root.children[1].id(),
        root.children[2].id(),
    ];
    let root = root.id;
    (doc, root, ids)
}

/// The two views: 800×450 points at 1.5 pixels per point, offset in the window, and 400×400
/// points at one pixel per point, where the slide is letterboxed above and below.
fn views() -> [(Rect, [usize; 2]); 2] {
    [
        (
            Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(800.0, 450.0)),
            [1200, 675],
        ),
        (
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(400.0, 400.0)),
            [400, 400],
        ),
    ]
}

/// The point where `render_slide` draws the slide unit `(u, v)` in a view, from `slide_to_pixels`.
fn at(doc: &Document, view: Rect, pixels: [usize; 2], u: f64, v: f64) -> Pos2 {
    let t = slide_to_pixels(doc, pixels[0] as u16, pixels[1] as u16);
    let point = f64::from(view.width()) / pixels[0] as f64;
    Pos2::new(
        view.min.x + ((t.x + t.scale * u) * point) as f32,
        view.min.y + ((t.y + t.scale * v) * point) as f32,
    )
}

/// How many points one slide unit is in a view.
fn points_per_unit(doc: &Document, view: Rect, pixels: [usize; 2]) -> f32 {
    let t = slide_to_pixels(doc, pixels[0] as u16, pixels[1] as u16);
    (t.scale * f64::from(view.width()) / pixels[0] as f64) as f32
}

#[test]
fn a_point_selects_the_area_whose_rectangle_holds_it() {
    let (doc, _, _) = two_close_dividers();
    let slide = doc.slides[0].id;
    let rects = area_rects(&doc.slides[0], doc.aspect);
    for (view, pixels) in views() {
        let layout = Layout::new(&doc, slide, view, pixels).unwrap();
        // A grid of points, none on an edge, the gaps far from the dividers included.
        for i in 0..48 {
            for j in 0..27 {
                let (u, v) = (20.0 + 40.0 * f64::from(i) + 0.5, 20.0 + 40.0 * f64::from(j));
                let want = rects
                    .iter()
                    .find(|(_, r)| u >= r.x && u < r.x + r.w && v >= r.y && v < r.y + r.h)
                    .map(|a| a.0);
                let p = at(&doc, view, pixels, u, v);
                assert_eq!(layout.area_at(p), want, "({u}, {v}) at {p:?} in {view:?}");
            }
        }
    }
}

#[test]
fn a_press_within_4_points_of_a_divider_grabs_it_at_every_view_size() {
    let (doc, root, [_, _, right]) = two_close_dividers();
    let slide = doc.slides[0].id;
    for (view, pixels) in views() {
        let layout = Layout::new(&doc, slide, view, pixels).unwrap();
        let root_divider = layout
            .dividers
            .iter()
            .position(|d| d.split == root && d.index == 1)
            .unwrap();
        // The divider at 960 slide units, from its right side, where the other one is farther.
        let line = at(&doc, view, pixels, 960.0, 540.0);
        let near = line + Vec2::new(3.9, 0.0);
        let far = line + Vec2::new(4.1, 0.0);
        assert_eq!(
            layout.press(near, None, None),
            Press::Divider(root_divider),
            "{view:?}"
        );
        assert_eq!(
            layout.press(far, None, None),
            Press::Area(right),
            "{view:?}"
        );
    }
}

#[test]
fn the_nearer_of_two_dividers_is_grabbed() {
    let (doc, root, _) = two_close_dividers();
    let (view, pixels) = views()[0];
    let layout = Layout::new(&doc, doc.slides[0].id, view, pixels).unwrap();
    let index = |i| {
        layout
            .dividers
            .iter()
            .position(|d| d.split == root && d.index == i)
            .unwrap()
    };
    // The two lines are 9.6 slide units apart, 4 points in this view: a point between them is
    // within 4 points of both.
    let unit = points_per_unit(&doc, view, pixels);
    assert!((9.6 * unit - 4.0).abs() < 0.01);
    let left = at(&doc, view, pixels, 950.4, 540.0);
    let right = at(&doc, view, pixels, 960.0, 540.0);
    assert_eq!(
        layout.press(left + Vec2::new(1.5, 0.0), None, None),
        Press::Divider(index(0))
    );
    assert_eq!(
        layout.press(right - Vec2::new(1.5, 0.0), None, None),
        Press::Divider(index(1))
    );
}

#[test]
fn a_press_in_the_letterbox_takes_nothing() {
    let (doc, _, _) = two_close_dividers();
    let (view, pixels) = views()[1];
    let layout = Layout::new(&doc, doc.slides[0].id, view, pixels).unwrap();
    // The 16:9 slide in a 400×400 view starts 87.5 points down.
    assert_eq!(
        layout.press(Pos2::new(200.0, 80.0), None, None),
        Press::Nothing
    );
    assert_eq!(
        layout.press(Pos2::new(200.0, 320.0), None, None),
        Press::Nothing
    );
    assert!(matches!(
        layout.press(Pos2::new(300.0, 200.0), None, None),
        Press::Area(_)
    ));
}

#[test]
fn a_divider_drag_stops_where_a_child_would_be_less_than_40_slide_units() {
    let (doc, root, _) = two_close_dividers();
    for (view, pixels) in views() {
        let layout = Layout::new(&doc, doc.slides[0].id, view, pixels).unwrap();
        let index = |i| {
            layout
                .dividers
                .iter()
                .position(|d| d.split == root && d.index == i)
                .unwrap()
        };
        let far_left = at(&doc, view, pixels, -500.0, 540.0);
        let far_right = at(&doc, view, pixels, 2500.0, 540.0);
        let middle = at(&doc, view, pixels, 1440.0, 540.0);
        let close = |a: f64, b: f64| (a - b).abs() < 1e-3;
        let margin = MIN_PART / 1920.0;
        // The divider at 0.5 lies between the one at 0.495 and the split's right edge.
        let d = index(1);
        assert!(close(
            layout.drag_position(d, far_left, false).unwrap(),
            0.495 + margin
        ));
        assert!(close(
            layout.drag_position(d, far_right, false).unwrap(),
            1.0 - margin
        ));
        assert!(close(layout.drag_position(d, middle, false).unwrap(), 0.75));
        // The divider at 0.495 lies between the split's left edge and the one at 0.5.
        let d = index(0);
        assert!(close(
            layout.drag_position(d, far_left, false).unwrap(),
            margin
        ));
        assert!(close(
            layout.drag_position(d, far_right, false).unwrap(),
            0.5 - margin
        ));
    }
}

/// The sample, its first slide's lower divider moved to 0.37, 710.4 slide units from the left,
/// which is on no twelfth of the slide's width: n4 stacked(0.3)[n2 | n6 beside(0.37)[n3 | n5]].
fn sample_off_twelfths() -> Document {
    struct NoFiles;
    impl rootpoint_protocol::Files for NoFiles {
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
    let mut session = rootpoint_protocol::Session::new();
    for line in ["sample", "divider n6 0 0.37"] {
        let command = rootpoint_protocol::parse_command(line).unwrap();
        assert!(matches!(
            session.run(command, &mut NoFiles),
            rootpoint_protocol::Reply::Ok(_)
        ));
    }
    session.document().clone()
}

/// Whether `p` lies in the convex polygon `shape`, its edges included.
fn inside(shape: &[Pos2], p: Pos2) -> bool {
    let sides: Vec<f32> = (0..shape.len())
        .map(|i| {
            let (a, b) = (shape[i], shape[(i + 1) % shape.len()]);
            (b - a).x * (p - a).y - (b - a).y * (p - a).x
        })
        .collect();
    sides.iter().all(|s| *s >= -1e-3) || sides.iter().all(|s| *s <= 1e-3)
}

#[test]
fn a_pointer_over_a_drop_target_falls_in_the_zone_the_hint_names() {
    let doc = sample_off_twelfths();
    let slide = doc.slides[0].id;
    let (dragged, target) = (NodeId(3), NodeId(2));
    for (view, pixels) in views() {
        let layout = Layout::new(&doc, slide, view, pixels).unwrap();
        let r = layout.rect(target).unwrap();
        for i in 0..40 {
            for j in 0..20 {
                let (fx, fy) = ((i as f32 + 0.5) / 40.0, (j as f32 + 0.5) / 20.0);
                let p = Pos2::new(r.min.x + fx * r.width(), r.min.y + fy * r.height());
                let drop = layout.drop(p, dragged).unwrap();
                assert_eq!(drop.target, target);
                assert!(inside(&drop.shape, p), "{p:?} outside {:?}", drop.zone);
                let third = 1.0 / 3.0;
                let middle = |f: f32| (third..=2.0 * third).contains(&f);
                let want = if middle(fx) && middle(fy) {
                    Zone::Swap
                } else if fx.min(1.0 - fx) <= fy.min(1.0 - fy) {
                    Zone::Move(if fx <= 0.5 { Edge::Left } else { Edge::Right })
                } else {
                    Zone::Move(if fy <= 0.5 { Edge::Up } else { Edge::Down })
                };
                assert_eq!(drop.zone, want, "({fx}, {fy}) in {view:?}");
            }
        }
        // A drop on the dragged area itself, or in the letterbox, has no target.
        let own = layout.rect(dragged).unwrap().center();
        assert_eq!(layout.drop(own, dragged), None);
        assert_eq!(layout.drop(view.min - Vec2::splat(5.0), dragged), None);
    }
}

#[test]
fn a_snapped_line_lies_on_a_twelfth_of_the_node_or_on_another_divider_within_8_points() {
    let doc = sample_off_twelfths();
    let slide = doc.slides[0].id;
    let title = NodeId(2);
    for (view, pixels) in views() {
        let layout = Layout::new(&doc, slide, view, pixels).unwrap();
        let r = layout.rect(title).unwrap();
        let divider = at(&doc, view, pixels, 710.4, 0.0).x;
        let y = r.center().y;
        for i in 1..200 {
            let x = r.min.x + r.width() * i as f32 / 200.0;
            let Some(p) = layout.preview(title, Axis::LeftToRight, Pos2::new(x, y), true) else {
                continue;
            };
            let twelfths = p.at * 12.0;
            let on_twelfth = (twelfths - twelfths.round()).abs() < 1e-3;
            let on_divider = (p.from.x - divider).abs() < 1e-3 && (x - divider).abs() <= 8.0 + 1e-3;
            assert!(
                on_twelfth || on_divider,
                "{x} snapped to {} in {view:?}",
                p.from.x
            );
            if (x - divider).abs() <= 3.0 {
                assert!(
                    on_divider,
                    "{x}, 3 points or less from the divider, in {view:?}"
                );
            }
        }
        // Unsnapped, the line follows the pointer.
        let x = divider + 3.0;
        let p = layout
            .preview(title, Axis::LeftToRight, Pos2::new(x, y), false)
            .unwrap();
        assert_eq!(p.from.x, x);
    }
}

#[test]
fn a_snapped_divider_drag_with_no_divider_across_lies_on_a_twelfth_of_its_split_or_at_a_stop() {
    let doc = sample_off_twelfths();
    let slide = doc.slides[0].id;
    for (view, pixels) in views() {
        let layout = Layout::new(&doc, slide, view, pixels).unwrap();
        // The slide's stacked divider, at 0.3 of the height.
        let i = layout
            .dividers
            .iter()
            .position(|d| d.split == NodeId(4))
            .unwrap();
        let r = layout.rect(NodeId(4)).unwrap();
        for j in 1..100 {
            let p = Pos2::new(r.center().x, r.min.y + r.height() * j as f32 / 100.0);
            let Some(pos) = layout.drag_position(i, p, true) else {
                continue;
            };
            // The stops at 40 slide units of 1080 lie between twelfths; every other position is
            // on one, since no other divider runs across the height.
            let stop = MIN_PART / 1080.0;
            let on_twelfth = (pos * 12.0 - (pos * 12.0).round()).abs() < 1e-3;
            assert!(
                on_twelfth || (pos - stop).abs() < 1e-6 || (pos - (1.0 - stop)).abs() < 1e-6,
                "{pos} in {view:?}"
            );
        }
    }
}

#[test]
fn a_split_at_the_pointer_gives_the_new_area_the_smaller_part_and_keeps_40_slide_units() {
    let doc = sample_off_twelfths();
    let slide = doc.slides[0].id;
    let title = NodeId(2);
    for (view, pixels) in views() {
        let layout = Layout::new(&doc, slide, view, pixels).unwrap();
        let point = |u: f64| at(&doc, view, pixels, u, 100.0);
        let preview = |u| layout.preview(title, Axis::LeftToRight, point(u), false);
        assert_eq!(preview(480.0).unwrap().side, Side::Before);
        assert_eq!(preview(1440.0).unwrap().side, Side::After);
        assert!(preview(30.0).is_none(), "a part of 30 slide units");
        assert!(preview(1900.0).is_none(), "a part of 20 slide units");
        // Ctrl or Shift held over the selection: a press there is a split; elsewhere, a select.
        let split = Some((Axis::LeftToRight, false));
        assert!(matches!(
            layout.press(point(480.0), Some(title), split),
            Press::Split(_)
        ));
        assert_eq!(
            layout.press(point(30.0), Some(title), split),
            Press::Nothing,
            "a refused split's click is ignored"
        );
        let below = at(&doc, view, pixels, 480.0, 700.0);
        assert_eq!(
            layout.press(below, Some(title), split),
            Press::Area(NodeId(3))
        );
    }
}

#[test]
fn a_press_on_the_grip_of_the_selected_area_takes_it() {
    let doc = sample_off_twelfths();
    let slide = doc.slides[0].id;
    for (view, pixels) in views() {
        let layout = Layout::new(&doc, slide, view, pixels).unwrap();
        let grip = layout.grip(NodeId(3)).unwrap();
        let r = layout.rect(NodeId(3)).unwrap();
        assert_eq!(grip.min, r.min + Vec2::splat(8.0));
        assert_eq!(grip.size(), Vec2::splat(12.0));
        let p = grip.center();
        assert_eq!(
            layout.press(p, Some(NodeId(3)), None),
            Press::Grip(NodeId(3))
        );
        // Not selected, the grip is not there: the press selects.
        assert_eq!(layout.press(p, None, None), Press::Area(NodeId(3)));
    }
}

#[test]
fn a_move_zone_that_would_split_its_target_under_40_slide_units_takes_no_drop() {
    let mut doc = sample_off_twelfths();
    // n6 beside[n3 | n5] with its divider at 0.97: n5 is 28.8 slide units wide.
    let Some(Node::Split(s)) = doc.slides[0].root.find(NodeId(6)).cloned() else {
        panic!("n6 is a split");
    };
    apply(
        &mut doc,
        Op::MoveDivider {
            split: s.id,
            index: 0,
            pos: 0.97,
        },
    )
    .unwrap();
    for (view, pixels) in views() {
        let layout = Layout::new(&doc, doc.slides[0].id, view, pixels).unwrap();
        let r = layout.rect(NodeId(5)).unwrap();
        // Its left edge's move zone would split it beside; its top edge's, stacked, has room.
        let left = Pos2::new(r.min.x + 0.5, r.center().y);
        let top = Pos2::new(r.center().x, r.min.y + 2.0);
        assert_eq!(layout.drop(left, NodeId(3)), None, "{view:?}");
        assert_eq!(
            layout.drop(top, NodeId(3)).map(|d| d.zone),
            Some(Zone::Move(Edge::Up)),
            "{view:?}"
        );
    }
}
