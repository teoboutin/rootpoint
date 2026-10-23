//! A file or an operation that would break an invariant of the document is refused. The cases are
//! the counterexamples the review of the model found before validation existed.

use rootpoint_model::*;

fn one_slide() -> (Document, NodeId) {
    let mut doc = Document::default();
    apply(&mut doc, Op::AddSlide { at: 0 }).unwrap();
    let root = doc.slides[0].root.id();
    (doc, root)
}

fn split_op(node: NodeId, axis: Axis, at: f64, side: Side) -> Op {
    Op::Split {
        node,
        axis,
        at,
        side,
    }
}

fn split(doc: &mut Document, node: NodeId, axis: Axis, at: f64, side: Side) {
    apply(doc, split_op(node, axis, at, side)).unwrap();
}

fn root_split(doc: &Document) -> &rootpoint_model::Split {
    match &doc.slides[0].root {
        Node::Split(s) => s,
        Node::Area(_) => panic!("the root is an area"),
    }
}

/// Apply `op`, which must be refused as invalid, and check the document is unchanged.
fn refused(doc: &mut Document, op: Op) {
    let before = doc.clone();
    match apply(doc, op.clone()) {
        Err(OpError::Invalid(_)) => {}
        other => panic!("{op:?} gave {other:?}, not a refusal as invalid"),
    }
    assert_eq!(*doc, before, "{op:?} changed the document");
}

fn fixture_text() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/fixture.ron"
    ))
    .unwrap()
}

fn load_refused(text: &str) {
    match load_ron(text) {
        Err(LoadError::Invalid(_)) => {}
        other => panic!("loaded as {other:?}, not refused as invalid"),
    }
}

#[test]
fn a_split_whose_dividers_underflow_to_zero_is_refused() {
    let (mut doc, root) = one_slide();
    split(&mut doc, root, Axis::LeftToRight, 1e-200, Side::After);
    let v = doc.slides[0].root.id();
    refused(
        &mut doc,
        split_op(v, Axis::LeftToRight, 1e-200, Side::After),
    );
}

#[test]
fn a_split_whose_divider_rounds_to_one_is_refused() {
    let (mut doc, root) = one_slide();
    split(
        &mut doc,
        root,
        Axis::LeftToRight,
        1.0 - f64::EPSILON / 2.0,
        Side::After,
    );
    let v = doc.slides[0].root.id();
    refused(&mut doc, split_op(v, Axis::LeftToRight, 0.5, Side::Before));
}

#[test]
fn a_split_whose_dividers_round_to_equal_is_refused() {
    let (mut doc, root) = one_slide();
    split(&mut doc, root, Axis::LeftToRight, 1e-20, Side::After);
    let v = doc.slides[0].root.id();
    refused(&mut doc, split_op(v, Axis::LeftToRight, 0.5, Side::Before));
}

#[test]
fn a_split_into_a_parent_extent_of_one_ulp_is_refused() {
    let (mut doc, root) = one_slide();
    split(&mut doc, root, Axis::LeftToRight, 0.5, Side::After);
    let second = root_split(&doc).children[1].id();
    split(&mut doc, second, Axis::LeftToRight, 0.5, Side::After);
    let v = doc.slides[0].root.id();
    let next = f64::from_bits(0.5f64.to_bits() + 1);
    apply(
        &mut doc,
        Op::MoveDivider {
            split: v,
            index: 1,
            pos: next,
        },
    )
    .unwrap();
    refused(
        &mut doc,
        split_op(second, Axis::LeftToRight, 0.5, Side::After),
    );
}

#[test]
fn repeated_ordinary_splits_stop_before_dividers_meet() {
    // Keep splitting the child right of 0.5 at 0.05, before it: its extent shrinks by 20 each
    // time, until rounding would make two dividers equal, which is refused.
    let (mut doc, root) = one_slide();
    split(&mut doc, root, Axis::LeftToRight, 0.5, Side::After);
    let mut target = root_split(&doc).children[1].id();
    let mut refusals = 0;
    for _ in 0..40 {
        match apply(
            &mut doc,
            split_op(target, Axis::LeftToRight, 0.05, Side::Before),
        ) {
            Ok(_) => {
                let s = root_split(&doc);
                let i = s.children.iter().position(|c| c.id() == target).unwrap();
                target = s.children[i - 1].id();
            }
            Err(OpError::Invalid(_)) => refusals += 1,
            Err(e) => panic!("{e:?}"),
        }
        assert_eq!(doc.validate(), Ok(()));
    }
    assert!(refusals > 0, "no split was refused in 40");
}

#[test]
fn a_merge_on_remove_into_a_tiny_extent_is_refused() {
    let (mut doc, a) = one_slide();
    split(&mut doc, a, Axis::LeftToRight, 0.5, Side::After);
    let r = root_split(&doc).children[1].id();
    split(&mut doc, r, Axis::LeftToRight, 0.5, Side::After);
    let v = doc.slides[0].root.id();
    let near = f64::from_bits(0.5f64.to_bits() + 4);
    apply(
        &mut doc,
        Op::MoveDivider {
            split: v,
            index: 1,
            pos: near,
        },
    )
    .unwrap();
    split(&mut doc, r, Axis::TopToBottom, 0.5, Side::After);
    let Node::Split(h) = &root_split(&doc).children[1] else {
        panic!("the middle child is not a split");
    };
    let x = h.children[1].id();
    split(&mut doc, x, Axis::LeftToRight, 0.1, Side::After);
    refused(&mut doc, Op::Remove { node: r });
}

#[test]
fn a_file_with_a_missing_divider_is_refused() {
    let text = fixture_text();
    let edited = text.replacen(
        "                dividers: [\n                    0.5,\n                ],\n            ),\n        ),",
        "                dividers: [],\n            ),\n        ),",
        1,
    );
    assert_ne!(edited, text, "the fixture changed shape");
    load_refused(&edited);
}

#[test]
fn a_file_whose_counter_is_below_its_ids_is_refused() {
    let text = fixture_text();
    load_refused(&text.replacen("next_id: 8,", "next_id: 2,", 1));
    load_refused(&text.replacen("next_id: 8,", "next_id: 7,", 1));
}

#[test]
fn a_file_with_a_zero_side_is_refused() {
    load_refused(&fixture_text().replacen("height: 9,", "height: 0,", 1));
}

#[test]
fn a_file_naming_a_missing_asset_is_refused() {
    load_refused(&fixture_text().replacen("asset: 7,", "asset: 70,", 1));
}

#[test]
fn restoring_an_asset_the_counter_has_not_reached_is_refused() {
    let mut doc = Document::default();
    refused(
        &mut doc,
        Op::RestoreAsset {
            id: AssetId(1),
            asset: Asset {
                source: AssetSource::Path("a.png".into()),
            },
        },
    );
}

#[test]
fn restoring_a_slide_whose_node_ids_collide_is_refused() {
    let (mut doc, first_root) = one_slide();
    apply(&mut doc, Op::AddSlide { at: 1 }).unwrap();
    let second = doc.slides[1].id;
    let Op::RestoreSlide { at, mut slide } =
        apply(&mut doc, Op::RemoveSlide { slide: second }).unwrap()
    else {
        panic!("RemoveSlide returned another inverse");
    };
    // Give the removed slide's root the id of the other slide's root.
    if let Node::Area(a) = &mut slide.root {
        a.id = first_root;
    }
    refused(&mut doc, Op::RestoreSlide { at, slide });
}

#[test]
fn a_tree_breaking_the_split_invariants_is_refused() {
    let (mut doc, root) = one_slide();
    let slide = doc.slides[0].id;
    let Op::RemoveSlide { slide: spare } = apply(&mut doc, Op::AddSlide { at: 1 }).unwrap() else {
        panic!("AddSlide returned another inverse");
    };
    // A split of one child, with an id the counter has reached (the spare slide's).
    let bad = Node::Split(rootpoint_model::Split {
        id: NodeId(spare.0),
        axis: Axis::LeftToRight,
        children: vec![Node::Area(Area {
            id: root,
            content: Content::default(),
        })],
        dividers: vec![],
    });
    apply(&mut doc, Op::RemoveSlide { slide: spare }).unwrap();
    refused(&mut doc, Op::SetTree { slide, root: bad });
}

#[test]
fn a_text_whose_font_is_a_missing_asset_is_refused() {
    let (mut doc, root) = one_slide();
    refused(
        &mut doc,
        Op::SetContent {
            area: root,
            content: Content::Text(TextContent {
                text: "x".into(),
                style: TextStyle {
                    font: FontRef::Asset(AssetId(999)),
                    ..TextStyle::default()
                },
            }),
        },
    );
}

#[test]
fn divider_positions_of_every_magnitude_survive_the_file() {
    for pos in [
        0.1 + 0.2,
        1e-300,
        5e-324,
        1.0 - f64::EPSILON / 2.0,
        0.123_456_789_012_345_68,
        2.225_073_858_507_201_4e-308,
    ] {
        let (mut doc, root) = one_slide();
        split(&mut doc, root, Axis::LeftToRight, 0.5, Side::After);
        let v = doc.slides[0].root.id();
        apply(
            &mut doc,
            Op::MoveDivider {
                split: v,
                index: 0,
                pos,
            },
        )
        .unwrap();
        assert_eq!(load_ron(&save_ron(&doc)).as_ref(), Ok(&doc), "pos {pos:e}");
    }
}
