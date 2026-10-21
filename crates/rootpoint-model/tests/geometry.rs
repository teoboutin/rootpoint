//! The area rectangles of a slide tile it: they do not overlap, and their areas sum to the
//! slide's.

use proptest::prelude::*;
use rootpoint_model::*;

/// The tolerance on sums of floating-point areas, in square slide units.
const EPSILON: f64 = 1e-6;

fn check_tiling(doc: &Document) {
    let whole = doc.aspect.slide_rect();
    for slide in &doc.slides {
        let rects = area_rects(slide, doc.aspect);
        assert_eq!(rects.len(), slide.root.areas().len());
        let sum: f64 = rects.iter().map(|(_, r)| r.w * r.h).sum();
        assert!(
            (sum - whole.w * whole.h).abs() < EPSILON,
            "areas sum to {sum}"
        );
        for (i, (_, a)) in rects.iter().enumerate() {
            assert!(
                a.x >= -EPSILON && a.y >= -EPSILON,
                "{a:?} starts outside the slide"
            );
            assert!(a.x + a.w <= whole.w + EPSILON && a.y + a.h <= whole.h + EPSILON);
            for (_, b) in &rects[i + 1..] {
                let overlap_w = (a.x + a.w).min(b.x + b.w) - a.x.max(b.x);
                let overlap_h = (a.y + a.h).min(b.y + b.h) - a.y.max(b.y);
                assert!(
                    overlap_w <= EPSILON || overlap_h <= EPSILON,
                    "{a:?} overlaps {b:?}"
                );
            }
        }
    }
}

#[test]
fn a_16_9_slide_is_1920_by_1080() {
    let r = AspectRatio::default().slide_rect();
    assert_eq!((r.w, r.h), (1920.0, 1080.0));
}

#[test]
fn a_side_by_side_split_lays_its_children_left_to_right() {
    let mut doc = Document::default();
    apply(&mut doc, Op::AddSlide { at: 0 }).unwrap();
    let root = doc.slides[0].root.id();
    apply(
        &mut doc,
        Op::Split {
            node: root,
            axis: Axis::LeftToRight,
            at: 0.25,
            side: Side::After,
        },
    )
    .unwrap();
    let rects = area_rects(&doc.slides[0], doc.aspect);
    assert_eq!(rects[0].0, root);
    assert_eq!(
        rects[0].1,
        Rect {
            x: 0.0,
            y: 0.0,
            w: 480.0,
            h: 1080.0
        }
    );
    assert_eq!(
        rects[1].1,
        Rect {
            x: 480.0,
            y: 0.0,
            w: 1440.0,
            h: 1080.0
        }
    );
}

proptest! {
    /// Over random aspect ratios and random trees built by random splits and removals, the
    /// rectangles tile every slide.
    #[test]
    fn rectangles_tile_the_slide(
        width in 1u32..40,
        height in 1u32..40,
        steps in prop::collection::vec((any::<usize>(), any::<bool>(), 0.05f64..0.95, any::<bool>(), any::<bool>()), 1..40)
    ) {
        let mut doc = Document::default();
        doc.aspect = AspectRatio { width, height };
        apply(&mut doc, Op::AddSlide { at: 0 }).unwrap();
        for (pick, beside, at, after, remove) in steps {
            let ids: Vec<NodeId> = {
                fn walk(n: &Node, out: &mut Vec<NodeId>) {
                    out.push(n.id());
                    if let Node::Split(s) = n {
                        s.children.iter().for_each(|c| walk(c, out));
                    }
                }
                let mut out = Vec::new();
                walk(&doc.slides[0].root, &mut out);
                out
            };
            let node = ids[pick % ids.len()];
            let op = if remove {
                Op::Remove { node }
            } else {
                Op::Split {
                    node,
                    axis: if beside { Axis::LeftToRight } else { Axis::TopToBottom },
                    at,
                    side: if after { Side::After } else { Side::Before },
                }
            };
            let _ = apply(&mut doc, op);
            check_tiling(&doc);
        }
    }
}
