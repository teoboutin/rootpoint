//! Hit testing matches the drawing: through `slide_to_pixels`, at two view sizes, a point selects
//! the area whose rectangle holds it, and a press within 4 points of a divider grabs it, the nearer
//! of two. Claim 2 of `spec@first-prototype@editor-shell`, its first part.

use eframe::egui::{Pos2, Rect, Vec2};
use rootpoint_app::hit::{Layout, MIN_PART, Press};
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
        assert_eq!(layout.press(near), Press::Divider(root_divider), "{view:?}");
        assert_eq!(layout.press(far), Press::Area(right), "{view:?}");
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
        layout.press(left + Vec2::new(1.5, 0.0)),
        Press::Divider(index(0))
    );
    assert_eq!(
        layout.press(right - Vec2::new(1.5, 0.0)),
        Press::Divider(index(1))
    );
}

#[test]
fn a_press_in_the_letterbox_takes_nothing() {
    let (doc, _, _) = two_close_dividers();
    let (view, pixels) = views()[1];
    let layout = Layout::new(&doc, doc.slides[0].id, view, pixels).unwrap();
    // The 16:9 slide in a 400×400 view starts 87.5 points down.
    assert_eq!(layout.press(Pos2::new(200.0, 80.0)), Press::Nothing);
    assert_eq!(layout.press(Pos2::new(200.0, 320.0)), Press::Nothing);
    assert!(matches!(
        layout.press(Pos2::new(300.0, 200.0)),
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
            layout.drag_position(d, far_left).unwrap(),
            0.495 + margin
        ));
        assert!(close(
            layout.drag_position(d, far_right).unwrap(),
            1.0 - margin
        ));
        assert!(close(layout.drag_position(d, middle).unwrap(), 0.75));
        // The divider at 0.495 lies between the split's left edge and the one at 0.5.
        let d = index(0);
        assert!(close(layout.drag_position(d, far_left).unwrap(), margin));
        assert!(close(
            layout.drag_position(d, far_right).unwrap(),
            0.5 - margin
        ));
    }
}
