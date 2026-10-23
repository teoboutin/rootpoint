//! The operations of the model: each inverse restores the document, and the tree keeps its
//! invariants.

use std::collections::BTreeSet;

use proptest::prelude::*;
use rootpoint_model::*;

/// Every invariant a document's trees keep: every split has two children or more, as many
/// dividers as children minus one, strictly increasing in (0, 1), and no child split of its own
/// axis; every id is unique and below the counter's next value.
fn check_invariants(doc: &Document) {
    let mut ids = BTreeSet::new();
    for slide in &doc.slides {
        assert!(ids.insert(slide.id.0), "slide id {} reused", slide.id.0);
        check_node(&slide.root, &mut ids);
    }
    for id in doc.assets.keys() {
        assert!(ids.insert(id.0), "asset id {} reused", id.0);
    }
    let after = {
        let mut d = doc.clone();
        apply(&mut d, Op::AddSlide { at: 0 }).unwrap();
        d.slides[0].id.0
    };
    assert!(
        ids.iter().all(|&id| id < after),
        "an id is not below the counter"
    );
}

fn check_node(node: &Node, ids: &mut BTreeSet<u64>) {
    assert!(ids.insert(node.id().0), "node id {} reused", node.id().0);
    if let Node::Split(s) = node {
        assert!(
            s.children.len() >= 2,
            "split {:?} has fewer than two children",
            s.id
        );
        assert_eq!(s.dividers.len(), s.children.len() - 1, "split {:?}", s.id);
        let mut last = 0.0;
        for &d in &s.dividers {
            assert!(
                d > last && d < 1.0,
                "split {:?} dividers {:?}",
                s.id,
                s.dividers
            );
            last = d;
        }
        for child in &s.children {
            if let Node::Split(c) = child {
                assert_ne!(
                    c.axis, s.axis,
                    "split {:?} has a child split of its own axis",
                    s.id
                );
            }
            check_node(child, ids);
        }
    }
}

/// Apply `op`, check the invariants, apply the inverse, and check the document is back.
fn round_trip(doc: &Document, op: Op) -> Document {
    let mut work = doc.clone();
    let inverse = apply(&mut work, op.clone()).unwrap_or_else(|e| panic!("{op:?}: {e:?}"));
    check_invariants(&work);
    let after = work.clone();
    apply(&mut work, inverse).unwrap();
    check_invariants(&work);
    assert!(
        work.same_content(doc),
        "the inverse of {op:?} did not restore the document"
    );
    after
}

fn new_doc() -> (Document, SlideId, NodeId) {
    let mut doc = Document::default();
    apply(&mut doc, Op::AddSlide { at: 0 }).unwrap();
    let slide = doc.slides[0].id;
    let root = doc.slides[0].root.id();
    (doc, slide, root)
}

fn split(doc: &mut Document, node: NodeId, axis: Axis, at: f64, side: Side) {
    apply(
        doc,
        Op::Split {
            node,
            axis,
            at,
            side,
        },
    )
    .unwrap();
}

fn root_split(doc: &Document) -> &rootpoint_model::Split {
    match &doc.slides[0].root {
        Node::Split(s) => s,
        Node::Area(_) => panic!("the root is an area"),
    }
}

fn child_ids(s: &rootpoint_model::Split) -> Vec<NodeId> {
    s.children.iter().map(Node::id).collect()
}

/// One slide: a side-by-side split of an area on the left and, on the right, a stacked split of two
/// areas. Returns the document and the ids (left, top right, bottom right).
fn fixture() -> (Document, [NodeId; 3]) {
    let (mut doc, _, left) = new_doc();
    split(&mut doc, left, Axis::LeftToRight, 0.5, Side::After);
    let right = root_split(&doc).children[1].id();
    split(&mut doc, right, Axis::TopToBottom, 0.5, Side::After);
    let Node::Split(r) = &root_split(&doc).children[1] else {
        panic!("the right child is not a split");
    };
    let ids = [left, r.children[0].id(), r.children[1].id()];
    (doc, ids)
}

#[test]
fn every_operation_is_undone_by_its_inverse() {
    let (mut doc, [left, top, bottom]) = fixture();
    let slide = doc.slides[0].id;
    let image = Asset {
        source: AssetSource::Path("cat.png".to_owned()),
    };
    let RemoveAssetId(asset) = add_asset(&mut doc, image.clone());
    apply(
        &mut doc,
        Op::SetContent {
            area: top,
            content: Content::Text(TextContent {
                text: "Hello".to_owned(),
                style: TextStyle::default(),
            }),
        },
    )
    .unwrap();
    let beside = doc.slides[0].root.id();
    let removed = doc.slides[0].clone();
    let ops = vec![
        Op::AddSlide { at: 1 },
        Op::RemoveSlide { slide },
        Op::DuplicateSlide { slide },
        Op::Split {
            node: left,
            axis: Axis::TopToBottom,
            at: 0.3,
            side: Side::Before,
        },
        Op::Split {
            node: beside,
            axis: Axis::LeftToRight,
            at: 0.25,
            side: Side::After,
        },
        Op::Remove { node: bottom },
        Op::Remove { node: left },
        Op::SetTree {
            slide,
            // A tree of one area, reusing an id the document allocated.
            root: Node::Area(Area {
                id: top,
                content: Content::default(),
            }),
        },
        Op::MoveDivider {
            split: beside,
            index: 0,
            pos: 0.4,
        },
        Op::Swap { a: top, b: bottom },
        Op::SetContent {
            area: bottom,
            content: Content::Image(ImageContent {
                asset,
                fit: ImageFit::Fill,
            }),
        },
        Op::AddAsset {
            asset: image.clone(),
        },
        Op::RemoveAsset { id: asset },
        Op::SetAssetSource {
            id: asset,
            source: AssetSource::Embedded(vec![1, 2, 3]),
        },
    ];
    for op in ops {
        round_trip(&doc, op);
    }
    // MoveSlide and RestoreSlide need a document of two slides, and a removed slide.
    apply(&mut doc, Op::AddSlide { at: 1 }).unwrap();
    round_trip(&doc, Op::MoveSlide { slide, to: 1 });
    let mut without = doc.clone();
    apply(&mut without, Op::RemoveSlide { slide }).unwrap();
    round_trip(
        &without,
        Op::RestoreSlide {
            at: 0,
            slide: removed,
        },
    );
    let mut no_asset = doc.clone();
    apply(&mut no_asset, Op::RemoveAsset { id: asset }).unwrap();
    round_trip(
        &no_asset,
        Op::RestoreAsset {
            id: asset,
            asset: image,
        },
    );
}

struct RemoveAssetId(AssetId);

fn add_asset(doc: &mut Document, asset: Asset) -> RemoveAssetId {
    match apply(doc, Op::AddAsset { asset }).unwrap() {
        Op::RemoveAsset { id } => RemoveAssetId(id),
        other => panic!("AddAsset returned {other:?}"),
    }
}

#[test]
fn splitting_a_child_along_its_parents_axis_merges_into_the_parent() {
    for side in [Side::Before, Side::After] {
        let (mut doc, _, root) = new_doc();
        split(&mut doc, root, Axis::LeftToRight, 0.5, Side::After);
        let right = root_split(&doc).children[1].id();
        split(&mut doc, right, Axis::LeftToRight, 0.5, side);
        let s = root_split(&doc);
        assert_eq!(s.axis, Axis::LeftToRight);
        assert_eq!(s.children.len(), 3, "{side:?}");
        assert_eq!(s.dividers, vec![0.5, 0.75], "{side:?}");
        let new = match side {
            Side::Before => 1,
            Side::After => 2,
        };
        assert!(matches!(
            &s.children[new],
            Node::Area(Area { content: Content::Text(t), .. })
                if t.text.is_empty() && t.style == TextStyle::default()
        ));
        assert_ne!(s.children[new].id(), right);
        check_invariants(&doc);
    }
}

#[test]
fn splitting_a_split_along_its_own_axis_adds_a_first_or_last_child() {
    let (mut doc, _, root) = new_doc();
    split(&mut doc, root, Axis::LeftToRight, 0.5, Side::After);
    let beside = doc.slides[0].root.id();
    let before = child_ids(root_split(&doc));
    split(&mut doc, beside, Axis::LeftToRight, 0.75, Side::After);
    let s = root_split(&doc);
    assert_eq!(s.dividers, vec![0.375, 0.75]);
    assert_eq!(&child_ids(s)[..2], &before[..]);
    check_invariants(&doc);

    let (mut doc, _, root) = new_doc();
    split(&mut doc, root, Axis::LeftToRight, 0.5, Side::After);
    let beside = doc.slides[0].root.id();
    split(&mut doc, beside, Axis::LeftToRight, 0.25, Side::Before);
    assert_eq!(root_split(&doc).dividers, vec![0.25, 0.625]);
    check_invariants(&doc);
}

#[test]
fn splitting_across_the_parents_axis_wraps_the_node() {
    let (mut doc, [left, ..]) = fixture();
    split(&mut doc, left, Axis::TopToBottom, 0.4, Side::Before);
    let s = root_split(&doc);
    let Node::Split(wrapped) = &s.children[0] else {
        panic!("the left child is not a split");
    };
    assert_eq!(wrapped.axis, Axis::TopToBottom);
    assert_eq!(wrapped.dividers, vec![0.4]);
    assert_eq!(wrapped.children[1].id(), left);
    check_invariants(&doc);
}

#[test]
fn removing_gives_the_extent_to_the_sibling_before_or_after() {
    let (mut doc, _, root) = new_doc();
    split(&mut doc, root, Axis::LeftToRight, 0.25, Side::After);
    let second = root_split(&doc).children[1].id();
    split(&mut doc, second, Axis::LeftToRight, 0.5, Side::After);
    // Dividers at 0.25 and 0.625, children [root, second, third].
    let ids = child_ids(root_split(&doc));
    let mut middle = doc.clone();
    apply(&mut middle, Op::Remove { node: ids[1] }).unwrap();
    assert_eq!(root_split(&middle).dividers, vec![0.625]);
    assert_eq!(child_ids(root_split(&middle)), vec![ids[0], ids[2]]);
    let mut first = doc.clone();
    apply(&mut first, Op::Remove { node: ids[0] }).unwrap();
    assert_eq!(root_split(&first).dividers, vec![0.625]);
    assert_eq!(child_ids(root_split(&first)), vec![ids[1], ids[2]]);
}

#[test]
fn a_split_left_with_one_child_collapses() {
    let (mut doc, [left, top, bottom]) = fixture();
    apply(&mut doc, Op::Remove { node: left }).unwrap();
    // The side-by-side root lost a child, and its one child, the stacked split, became the root.
    let s = root_split(&doc);
    assert_eq!(s.axis, Axis::TopToBottom);
    assert_eq!(child_ids(s), vec![top, bottom]);
    check_invariants(&doc);
}

#[test]
fn a_collapse_into_a_split_of_the_parents_axis_merges() {
    // Side-by-side root [a | stacked[b / beside[c | d]]]: removing b leaves the stacked split with
    // one child, beside[c | d], which merges into the side-by-side root.
    let (mut doc, [a, b, below]) = fixture();
    split(&mut doc, below, Axis::LeftToRight, 0.5, Side::After);
    apply(&mut doc, Op::Remove { node: b }).unwrap();
    let s = root_split(&doc);
    assert_eq!(s.axis, Axis::LeftToRight);
    assert_eq!(s.children.len(), 3);
    assert_eq!(s.children[0].id(), a);
    assert_eq!(s.dividers, vec![0.5, 0.75]);
    check_invariants(&doc);
}

#[test]
fn a_refused_operation_changes_nothing() {
    let (doc, [left, ..]) = fixture();
    let beside = doc.slides[0].root.id();
    let refused = [
        Op::Remove { node: beside },
        Op::Remove { node: NodeId(999) },
        Op::Split {
            node: left,
            axis: Axis::LeftToRight,
            at: 1.0,
            side: Side::After,
        },
        Op::MoveDivider {
            split: beside,
            index: 0,
            pos: 1.5,
        },
        Op::MoveDivider {
            split: left,
            index: 0,
            pos: 0.5,
        },
        Op::Swap { a: left, b: beside },
        Op::SetContent {
            area: left,
            content: Content::Image(ImageContent {
                asset: AssetId(999),
                fit: ImageFit::Fit,
            }),
        },
        Op::RemoveAsset { id: AssetId(999) },
        Op::SetAssetSource {
            id: AssetId(999),
            source: AssetSource::Path("dog.png".to_owned()),
        },
        Op::AddSlide { at: 5 },
    ];
    for op in refused {
        let mut work = doc.clone();
        assert!(apply(&mut work, op.clone()).is_err(), "{op:?} was accepted");
        assert_eq!(work, doc, "{op:?} changed the document");
    }
}

#[test]
fn an_asset_in_use_cannot_be_removed() {
    let (mut doc, [left, ..]) = fixture();
    let RemoveAssetId(asset) = add_asset(
        &mut doc,
        Asset {
            source: AssetSource::Path("cat.png".to_owned()),
        },
    );
    apply(
        &mut doc,
        Op::SetContent {
            area: left,
            content: Content::Image(ImageContent {
                asset,
                fit: ImageFit::Fit,
            }),
        },
    )
    .unwrap();
    assert_eq!(
        apply(&mut doc, Op::RemoveAsset { id: asset }),
        Err(OpError::AssetInUse(asset))
    );
}

#[test]
fn a_duplicated_slide_has_new_ids() {
    let (mut doc, [left, top, bottom]) = fixture();
    let slide = doc.slides[0].id;
    apply(&mut doc, Op::DuplicateSlide { slide }).unwrap();
    let copy = doc.slides[1].root.areas();
    assert_eq!(copy.len(), 3);
    for area in copy {
        assert!(![left, top, bottom].contains(&area.id));
    }
    check_invariants(&doc);
}

/// One step of a random edit sequence: a kind of operation, and numbers that pick its targets
/// among the document's ids at the time it is applied.
#[derive(Clone, Debug)]
struct Step {
    kind: u8,
    pick: usize,
    other: usize,
    at: f64,
    flag: bool,
}

// Every variant of `Op` and of `Content` is reached by a kind of step in `op_for`, or as an
// inverse; a variant left out is silently untested.
fn step() -> impl Strategy<Value = Step> {
    // Positions near 0 and 1 as well as ordinary ones, where rounding meets the dividers.
    let at = prop_oneof![0.05f64..0.95, 1e-12f64..1e-3, (1.0 - 1e-3)..(1.0 - 1e-12)];
    (0u8..14, any::<usize>(), any::<usize>(), at, any::<bool>()).prop_map(
        |(kind, pick, other, at, flag)| Step {
            kind,
            pick,
            other,
            at,
            flag,
        },
    )
}

fn all_nodes(doc: &Document) -> Vec<&Node> {
    fn walk<'a>(n: &'a Node, out: &mut Vec<&'a Node>) {
        out.push(n);
        if let Node::Split(s) = n {
            s.children.iter().for_each(|c| walk(c, out));
        }
    }
    let mut out = Vec::new();
    doc.slides.iter().for_each(|s| walk(&s.root, &mut out));
    out
}

/// The operation a step picks in this document, or none when the document has nothing to pick.
fn op_for(doc: &Document, s: &Step) -> Option<Op> {
    let nodes = all_nodes(doc);
    let areas: Vec<NodeId> = doc
        .slides
        .iter()
        .flat_map(|sl| sl.root.areas())
        .map(|a| a.id)
        .collect();
    let splits: Vec<&rootpoint_model::Split> = nodes
        .iter()
        .filter_map(|n| match n {
            Node::Split(sp) => Some(sp),
            Node::Area(_) => None,
        })
        .collect();
    let assets: Vec<AssetId> = doc.assets.keys().copied().collect();
    let pick = |len: usize| s.pick % len.max(1);
    let axis = if s.flag {
        Axis::LeftToRight
    } else {
        Axis::TopToBottom
    };
    Some(match s.kind {
        0 => Op::AddSlide {
            at: pick(doc.slides.len() + 1),
        },
        1 if !doc.slides.is_empty() => Op::DuplicateSlide {
            slide: doc.slides[pick(doc.slides.len())].id,
        },
        2 | 3 if !nodes.is_empty() => Op::Split {
            node: nodes[pick(nodes.len())].id(),
            axis,
            at: s.at,
            side: if s.other.is_multiple_of(2) {
                Side::Before
            } else {
                Side::After
            },
        },
        4 if !nodes.is_empty() => Op::Remove {
            node: nodes[pick(nodes.len())].id(),
        },
        5 if !splits.is_empty() => {
            let sp = splits[pick(splits.len())];
            let index = s.other % sp.dividers.len();
            let low = if index == 0 {
                0.0
            } else {
                sp.dividers[index - 1]
            };
            let high = sp.dividers.get(index + 1).copied().unwrap_or(1.0);
            Op::MoveDivider {
                split: sp.id,
                index,
                pos: low + s.at * (high - low),
            }
        }
        6 if !areas.is_empty() => Op::Swap {
            a: areas[pick(areas.len())],
            b: areas[s.other % areas.len()],
        },
        7 if !areas.is_empty() => Op::SetContent {
            area: areas[pick(areas.len())],
            content: Content::Text(TextContent {
                text: format!("text {}", s.other % 100),
                style: TextStyle::default(),
            }),
        },
        8 if !doc.slides.is_empty() => Op::MoveSlide {
            slide: doc.slides[pick(doc.slides.len())].id,
            to: s.other % doc.slides.len(),
        },
        9 => Op::AddAsset {
            asset: Asset {
                source: if s.flag {
                    AssetSource::Path(format!("image{}.png", s.other % 10))
                } else {
                    AssetSource::Embedded(vec![s.other as u8; 3])
                },
            },
        },
        10 if !assets.is_empty() => Op::RemoveAsset {
            id: assets[pick(assets.len())],
        },
        11 if !areas.is_empty() && !assets.is_empty() => Op::SetContent {
            area: areas[pick(areas.len())],
            content: Content::Image(ImageContent {
                asset: assets[s.other % assets.len()],
                fit: if s.flag {
                    ImageFit::Fit
                } else {
                    ImageFit::Fill
                },
            }),
        },
        12 if !areas.is_empty() && !assets.is_empty() => Op::SetContent {
            area: areas[pick(areas.len())],
            content: Content::Text(TextContent {
                text: "in an asset font".to_owned(),
                style: TextStyle {
                    font: FontRef::Asset(assets[s.other % assets.len()]),
                    ..TextStyle::default()
                },
            }),
        },
        13 if !doc.slides.is_empty() => Op::RemoveSlide {
            slide: doc.slides[pick(doc.slides.len())].id,
        },
        _ => return None,
    })
}

proptest! {
    /// Over random edit sequences, every accepted operation keeps the invariants and is undone
    /// by its inverse, and every refused one changes nothing.
    #[test]
    fn random_edits_keep_the_invariants_and_invert(steps in prop::collection::vec(step(), 1..40)) {
        let mut doc = Document::default();
        for s in &steps {
            let Some(op) = op_for(&doc, s) else { continue };
            let before = doc.clone();
            match apply(&mut doc, op.clone()) {
                Ok(inverse) => {
                    check_invariants(&doc);
                    let mut undone = doc.clone();
                    // The inverse is applied, and validated, as any operation is.
                    apply(&mut undone, inverse).unwrap();
                    check_invariants(&undone);
                    prop_assert!(undone.same_content(&before), "{:?} not undone", op);
                }
                Err(_) => prop_assert_eq!(&doc, &before),
            }
        }
    }
}
