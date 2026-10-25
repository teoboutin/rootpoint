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
    if let Node::Area(Area {
        content: Content::Text(t),
        ..
    }) = node
    {
        for span in t.root.spans() {
            assert!(ids.insert(span.id.0), "span id {} reused", span.id.0);
        }
    }
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
        path: "cat.png".to_owned(),
    };
    let RemoveAssetId(asset) = add_asset(&mut doc, image.clone());
    apply(
        &mut doc,
        Op::SetContent {
            area: top,
            content: Content::Text(TextContent {
                root: rootpoint_model::TextNode::plain("Hello"),
                style: StyleChoice::Custom(TextStyle::default()),
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
        Op::SetAssetPath {
            id: asset,
            path: "img/dog.png".to_owned(),
        },
        // A reload of a file changed outside the editor, here an empty document.
        Op::ReplaceDocument {
            document: Box::new(Document::default()),
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
                if t.plain_text().is_empty() && t.style == StyleChoice::Named(BODY.to_owned())
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
        Op::SetAssetPath {
            id: AssetId(999),
            path: "dog.png".to_owned(),
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
            path: "cat.png".to_owned(),
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

// Every variant of `Op` and of `Content` but those of text trees and span styles is reached by a
// kind of step in `op_for`, or as an inverse; a variant left out is silently untested. The text
// operations have their own property test below, and the span style operations their unit tests.
fn step() -> impl Strategy<Value = Step> {
    // Positions near 0 and 1 as well as ordinary ones, where rounding meets the dividers.
    let at = prop_oneof![0.05f64..0.95, 1e-12f64..1e-3, (1.0 - 1e-3)..(1.0 - 1e-12)];
    (0u8..19, any::<usize>(), any::<usize>(), at, any::<bool>()).prop_map(
        |(kind, pick, other, at, flag)| Step {
            kind,
            pick,
            other,
            at,
            flag,
        },
    )
}

/// The names the style steps pick from: two of the predefined styles, two new ones, and the empty
/// name, which is refused.
const STYLE_NAMES: [&str; 5] = ["Title", "Body", "Quote", "Note", ""];

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
        // An equation, numbered or not, fitted or held at a size, as often as a text.
        7 if !areas.is_empty() && s.flag => Op::SetContent {
            area: areas[pick(areas.len())],
            content: Content::Equation(EquationContent {
                formula: rootpoint_math::Formula::parse(&format!(r"x^{{{}}}", s.other % 10))
                    .unwrap(),
                numbered: s.other.is_multiple_of(2),
                size: if s.other.is_multiple_of(3) {
                    EquationSize::Fit
                } else {
                    EquationSize::AtMost(10.0 + (s.other % 90) as f64)
                },
                colour: TextStyle::default().colour,
            }),
        },
        7 if !areas.is_empty() => Op::SetContent {
            area: areas[pick(areas.len())],
            content: Content::Text(TextContent {
                root: rootpoint_model::TextNode::plain(format!("text {}", s.other % 100)),
                style: StyleChoice::Custom(TextStyle::default()),
            }),
        },
        8 if !doc.slides.is_empty() => Op::MoveSlide {
            slide: doc.slides[pick(doc.slides.len())].id,
            to: s.other % doc.slides.len(),
        },
        9 => Op::AddAsset {
            asset: Asset {
                path: if s.flag {
                    format!("image{}.png", s.other % 10)
                } else {
                    format!("../image{}.png", s.other % 10)
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
                root: rootpoint_model::TextNode::plain("in an asset font"),
                style: StyleChoice::Custom(TextStyle {
                    font: FontRef::Asset(assets[s.other % assets.len()]),
                    ..TextStyle::default()
                }),
            }),
        },
        13 if !doc.slides.is_empty() => Op::RemoveSlide {
            slide: doc.slides[pick(doc.slides.len())].id,
        },
        // Style names include ones the table holds, ones it lacks and the empty name, and the
        // texts name them, so renames and removals meet texts that name a style.
        14 => Op::AddStyle {
            name: STYLE_NAMES[s.pick % STYLE_NAMES.len()].to_owned(),
            style: TextStyle {
                size: 10.0 + (s.other % 90) as f64,
                ..TextStyle::default()
            },
        },
        15 => Op::SetStyle {
            name: STYLE_NAMES[s.pick % STYLE_NAMES.len()].to_owned(),
            style: TextStyle {
                bold: s.flag,
                ..TextStyle::default()
            },
        },
        16 => Op::RenameStyle {
            from: STYLE_NAMES[s.pick % STYLE_NAMES.len()].to_owned(),
            to: STYLE_NAMES[s.other % STYLE_NAMES.len()].to_owned(),
        },
        17 if s.flag && !areas.is_empty() => Op::SetContent {
            area: areas[pick(areas.len())],
            content: Content::Text(TextContent {
                root: rootpoint_model::TextNode::plain("named"),
                style: StyleChoice::Named(STYLE_NAMES[s.other % STYLE_NAMES.len()].to_owned()),
            }),
        },
        17 => Op::RemoveStyle {
            name: STYLE_NAMES[s.pick % STYLE_NAMES.len()].to_owned(),
            into: match s.other % 3 {
                0 => None,
                1 => Some(Replacement::Custom),
                _ => Some(Replacement::Named(
                    STYLE_NAMES[(s.other / 3) % STYLE_NAMES.len()].to_owned(),
                )),
            },
        },
        // A reload of a file changed outside the editor: a document of a few slides of its own,
        // whose counter is below the editor's.
        18 => {
            let mut file = Document::default();
            for at in 0..s.other % 3 {
                apply(&mut file, Op::AddSlide { at }).unwrap();
            }
            Op::ReplaceDocument {
                document: Box::new(file),
            }
        }
        _ => return None,
    })
}

/// A document's next id, as the next slide added takes it.
fn next_id(doc: &Document) -> u64 {
    let mut d = doc.clone();
    apply(&mut d, Op::AddSlide { at: 0 }).unwrap();
    d.slides[0].id.0
}

/// A reload never gives again an id the editor handed out: the counter keeps the larger of the
/// two documents', at the replace and at its undo.
#[test]
fn a_replace_keeps_the_larger_counter() {
    let (mut doc, _) = fixture();
    let before = next_id(&doc);
    let inverse = apply(
        &mut doc,
        Op::ReplaceDocument {
            document: Box::new(Document::default()),
        },
    )
    .unwrap();
    assert_eq!(next_id(&doc), before);
    // The ids the reloaded document handed out stay taken after the undo.
    apply(&mut doc, Op::AddSlide { at: 0 }).unwrap();
    let after_add = next_id(&doc);
    apply(&mut doc, inverse).unwrap();
    assert_eq!(next_id(&doc), after_add);
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

#[test]
fn an_asset_a_style_uses_cannot_be_removed() {
    let (mut doc, _) = fixture();
    let RemoveAssetId(asset) = add_asset(
        &mut doc,
        Asset {
            path: "font.ttf".to_owned(),
        },
    );
    apply(
        &mut doc,
        Op::AddStyle {
            name: "Quote".to_owned(),
            style: TextStyle {
                font: FontRef::Asset(asset),
                ..TextStyle::default()
            },
        },
    )
    .unwrap();
    assert_eq!(
        apply(&mut doc, Op::RemoveAsset { id: asset }),
        Err(OpError::AssetInUse(asset))
    );
}

/// A document of one slide whose root is one text area, and the area's id.
fn one_text(text: &str) -> (Document, NodeId) {
    let mut doc = Document::default();
    apply(&mut doc, Op::AddSlide { at: 0 }).unwrap();
    let area = doc.slides[0].root.id();
    apply(
        &mut doc,
        Op::InsertText {
            area,
            node: TextNodeRef::Root,
            offset: 0,
            text: text.to_owned(),
        },
    )
    .unwrap();
    (doc, area)
}

fn text_of(doc: &Document, area: NodeId) -> &TextContent {
    match doc.node(area) {
        Some(Node::Area(Area {
            content: Content::Text(t),
            ..
        })) => t,
        other => panic!("area {area:?} holds no text: {other:?}"),
    }
}

fn bold() -> SpanChoice {
    SpanChoice::Custom(SpanStyle {
        bold: Some(true),
        ..SpanStyle::default()
    })
}

/// The id of the first span of the area's text.
fn first_span(doc: &Document, area: NodeId) -> SpanId {
    text_of(doc, area).root.spans()[0].id
}

/// A text node, and every span below it, is in its canonical form: no run of characters empty, no
/// two adjacent, and no custom span that sets no field.
fn check_canonical(node: &TextNode) {
    let mut last_chars = false;
    for item in &node.items {
        match item {
            TextItem::Chars(s) => {
                assert!(!s.is_empty(), "an empty run in {node:?}");
                assert!(!last_chars, "two adjacent runs in {node:?}");
                last_chars = true;
            }
            TextItem::Span(span) => {
                last_chars = false;
                assert_ne!(
                    span.style,
                    SpanChoice::Custom(SpanStyle::default()),
                    "a custom span that sets nothing"
                );
                check_canonical(&span.text);
            }
        }
    }
}

#[test]
fn wrapping_a_range_makes_a_span_with_a_new_id() {
    let (mut doc, area) = one_text("a prime number");
    apply(
        &mut doc,
        Op::WrapSpan {
            area,
            node: TextNodeRef::Root,
            range: 2..7,
            style: bold(),
        },
    )
    .unwrap();
    let t = text_of(&doc, area);
    assert_eq!(t.plain_text(), "a prime number");
    let span = t.root.spans()[0];
    assert_eq!(span.text, TextNode::plain("prime"));
    assert!(span.id.0 < next_id(&doc));
}

#[test]
fn a_custom_span_that_sets_nothing_is_refused() {
    let (mut doc, area) = one_text("abc");
    let before = doc.clone();
    let empty = SpanChoice::Custom(SpanStyle::default());
    assert_eq!(
        apply(
            &mut doc,
            Op::WrapSpan {
                area,
                node: TextNodeRef::Root,
                range: 0..1,
                style: empty.clone(),
            },
        ),
        Err(OpError::EmptySpanStyle)
    );
    assert_eq!(doc, before);
}

/// A named span is kept whatever its values, and so is a span that sets a value equal to its
/// parent's: a bold word in a bold span stays a span of its own.
#[test]
fn a_named_span_and_a_span_equal_to_its_parent_are_kept() {
    let (mut doc, area) = one_text("bold word");
    let wrap = |doc: &mut Document, node, range, style| {
        apply(
            doc,
            Op::WrapSpan {
                area,
                node,
                range,
                style,
            },
        )
        .unwrap()
    };
    wrap(&mut doc, TextNodeRef::Root, 0..9, bold());
    let outer = first_span(&doc, area);
    wrap(&mut doc, TextNodeRef::Span(outer), 5..9, bold());
    wrap(
        &mut doc,
        TextNodeRef::Span(outer),
        0..1,
        SpanChoice::Named("Missing".to_owned()),
    );
    let spans = text_of(&doc, area).root.spans().len();
    assert_eq!(spans, 3);
}

#[test]
fn a_span_cut_at_an_edge_is_refused_and_inside_gives_two_spans() {
    let (mut doc, area) = one_text("primenumber");
    apply(
        &mut doc,
        Op::WrapSpan {
            area,
            node: TextNodeRef::Root,
            range: 0..11,
            style: bold(),
        },
    )
    .unwrap();
    let span = first_span(&doc, area);
    for offset in [0, 11] {
        assert_eq!(
            apply(&mut doc, Op::SplitSpan { area, span, offset }),
            Err(OpError::SpanEdge(span))
        );
    }
    apply(
        &mut doc,
        Op::SplitSpan {
            area,
            span,
            offset: 5,
        },
    )
    .unwrap();
    let t = text_of(&doc, area);
    let spans = t.root.spans();
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].id, span);
    assert_eq!(spans[0].text, TextNode::plain("prime"));
    assert_eq!(spans[1].text, TextNode::plain("number"));
    assert_eq!(spans[0].style, spans[1].style);
}

/// A deleted range takes a span inside it whole, and the undo brings the span back with its id.
#[test]
fn a_deletion_takes_a_span_whole_and_its_undo_brings_it_back() {
    let (mut doc, area) = one_text("a prime number");
    apply(
        &mut doc,
        Op::WrapSpan {
            area,
            node: TextNodeRef::Root,
            range: 2..7,
            style: bold(),
        },
    )
    .unwrap();
    // "a " then the span, one offset, then " number".
    let before = doc.clone();
    let inverse = apply(
        &mut doc,
        Op::DeleteText {
            area,
            node: TextNodeRef::Root,
            range: 1..4,
        },
    )
    .unwrap();
    assert_eq!(text_of(&doc, area).plain_text(), "anumber");
    apply(&mut doc, inverse).unwrap();
    assert!(doc.same_content(&before));
}

#[test]
fn a_tree_set_whole_gives_every_span_a_new_id() {
    let (mut doc, area) = one_text("");
    let given = TextNode {
        items: vec![
            TextItem::Chars("a ".to_owned()),
            TextItem::Span(Span {
                id: SpanId(0),
                style: bold(),
                text: TextNode::plain("b"),
            }),
        ],
    };
    apply(&mut doc, Op::SetText { area, root: given }).unwrap();
    let span = first_span(&doc, area);
    assert_ne!(span, SpanId(0));
    check_invariants(&doc);
}

/// A step of a random edit of one text: which operation, two offsets, which text node, and a
/// string.
#[derive(Clone, Debug)]
struct TextStep {
    kind: u8,
    a: usize,
    b: usize,
    node: usize,
    text: String,
}

fn text_step() -> impl Strategy<Value = TextStep> {
    (0u8..7, 0usize..20, 0usize..20, 0usize..6, "[a-cé ]{0,4}").prop_map(
        |(kind, a, b, node, text)| TextStep {
            kind,
            a,
            b,
            node,
            text,
        },
    )
}

/// The operation a step makes on the text of `area`, with its offsets taken within the chosen
/// text node, so that most steps are accepted.
fn text_op(doc: &Document, area: NodeId, s: &TextStep) -> Op {
    let t = text_of(doc, area);
    let spans = t.root.spans();
    let node = match s.node % (spans.len() + 1) {
        0 => TextNodeRef::Root,
        i => TextNodeRef::Span(spans[i - 1].id),
    };
    let len = match node {
        TextNodeRef::Root => offsets(&t.root),
        TextNodeRef::Span(id) => offsets(&spans.iter().find(|x| x.id == id).unwrap().text),
    };
    let (lo, hi) = {
        let (a, b) = (s.a % (len + 1), s.b % (len + 1));
        (a.min(b), a.max(b))
    };
    let span = match node {
        TextNodeRef::Span(id) => id,
        TextNodeRef::Root => spans.first().map(|x| x.id).unwrap_or(SpanId(u64::MAX)),
    };
    let style = if s.b.is_multiple_of(3) {
        SpanChoice::Named("Term".to_owned())
    } else {
        bold()
    };
    match s.kind {
        0 => Op::InsertText {
            area,
            node,
            offset: lo,
            text: s.text.clone(),
        },
        1 => Op::DeleteText {
            area,
            node,
            range: lo..hi,
        },
        2 => Op::WrapSpan {
            area,
            node,
            range: lo..hi,
            style,
        },
        3 => Op::UnwrapSpan { area, span },
        4 => Op::SetSpanStyle { area, span, style },
        5 => Op::SplitSpan {
            area,
            span,
            offset: lo,
        },
        _ => Op::SetText {
            area,
            root: t.root.clone(),
        },
    }
}

/// The number of offsets of a text node: its characters, and one per span.
fn offsets(node: &TextNode) -> usize {
    node.items
        .iter()
        .map(|i| match i {
            TextItem::Chars(s) => s.chars().count(),
            TextItem::Span(_) => 1,
        })
        .sum()
}

proptest! {
    /// Over random edits of one text, every accepted operation keeps the invariants and the
    /// canonical form and is undone by its inverse, and every refused one changes nothing.
    #[test]
    fn random_text_edits_keep_the_canonical_form_and_invert(
        steps in prop::collection::vec(text_step(), 1..40)
    ) {
        let (mut doc, area) = one_text("ab cd");
        for s in &steps {
            let op = text_op(&doc, area, s);
            let before = doc.clone();
            match apply(&mut doc, op.clone()) {
                Ok(inverse) => {
                    check_invariants(&doc);
                    check_canonical(&text_of(&doc, area).root);
                    let mut undone = doc.clone();
                    apply(&mut undone, inverse).unwrap();
                    prop_assert!(undone.same_content(&before), "{:?} not undone", op);
                }
                Err(_) => prop_assert_eq!(&doc, &before),
            }
        }
    }
}

/// A document with the span style "Term", italic, and one text whose span `t` names it.
fn with_term() -> (Document, NodeId, SpanId) {
    let (mut doc, area) = one_text("a term here");
    apply(
        &mut doc,
        Op::AddSpanStyle {
            name: "Term".to_owned(),
            style: SpanStyle {
                italic: Some(true),
                ..SpanStyle::default()
            },
        },
    )
    .unwrap();
    apply(
        &mut doc,
        Op::WrapSpan {
            area,
            node: TextNodeRef::Root,
            range: 2..6,
            style: SpanChoice::Named("Term".to_owned()),
        },
    )
    .unwrap();
    let span = first_span(&doc, area);
    (doc, area, span)
}

#[test]
fn a_span_style_removed_with_a_copy_gives_its_users_its_values_and_its_undo_names_it_again() {
    let (mut doc, area, span) = with_term();
    let before = doc.clone();
    let inverse = apply(
        &mut doc,
        Op::RemoveSpanStyle {
            name: "Term".to_owned(),
            into: Some(Replacement::Custom),
        },
    )
    .unwrap();
    assert!(!doc.span_styles.contains_key("Term"));
    let style = &text_of(&doc, area).root.spans()[0].style;
    assert_eq!(
        style,
        &SpanChoice::Custom(SpanStyle {
            italic: Some(true),
            ..SpanStyle::default()
        })
    );
    apply(&mut doc, inverse).unwrap();
    assert!(doc.same_content(&before));
    assert_eq!(span_style_users(&doc, "Term"), vec![(area, span)]);
}

#[test]
fn a_span_style_removed_with_no_replacement_leaves_a_breach() {
    let (mut doc, area, span) = with_term();
    apply(
        &mut doc,
        Op::RemoveSpanStyle {
            name: "Term".to_owned(),
            into: None,
        },
    )
    .unwrap();
    assert_eq!(
        doc.breaches(),
        vec![Breach::MissingSpanStyle {
            area,
            span,
            name: "Term".to_owned()
        }]
    );
}

#[test]
fn a_span_style_rename_renames_its_spans_and_is_refused_onto_a_named_one() {
    let (mut doc, area, span) = with_term();
    apply(
        &mut doc,
        Op::RenameSpanStyle {
            from: "Term".to_owned(),
            to: "Word".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(span_style_users(&doc, "Word"), vec![(area, span)]);
    // A span naming a style the table lacks blocks a rename onto that name.
    apply(
        &mut doc,
        Op::SetSpanStyle {
            area,
            span,
            style: SpanChoice::Named("Ghost".to_owned()),
        },
    )
    .unwrap();
    assert_eq!(
        apply(
            &mut doc,
            Op::RenameSpanStyle {
                from: "Word".to_owned(),
                to: "Ghost".to_owned(),
            },
        ),
        Err(OpError::SpanStyleNamed {
            name: "Ghost".to_owned(),
            span
        })
    );
}

#[test]
fn a_span_font_asset_cannot_be_removed_and_one_above_the_counter_is_refused() {
    let (mut doc, area) = one_text("abc");
    let RemoveAssetId(asset) = add_asset(
        &mut doc,
        Asset {
            path: "font.ttf".to_owned(),
        },
    );
    let font = |id| {
        SpanChoice::Custom(SpanStyle {
            font: Some(FontRef::Asset(id)),
            ..SpanStyle::default()
        })
    };
    apply(
        &mut doc,
        Op::WrapSpan {
            area,
            node: TextNodeRef::Root,
            range: 0..1,
            style: font(asset),
        },
    )
    .unwrap();
    assert_eq!(
        apply(&mut doc, Op::RemoveAsset { id: asset }),
        Err(OpError::AssetInUse(asset))
    );
    let far = AssetId(next_id(&doc) + 10);
    assert!(matches!(
        apply(
            &mut doc,
            Op::WrapSpan {
                area,
                node: TextNodeRef::Root,
                range: 1..2,
                style: font(far),
            },
        ),
        Err(OpError::Invalid(_))
    ));
}

/// A text whose named style the table lacks still has its spans' breaches named, as the renderer
/// reports them.
#[test]
fn the_spans_of_a_text_whose_style_is_missing_are_named_too() {
    let (mut doc, area) = one_text("abc");
    apply(
        &mut doc,
        Op::WrapSpan {
            area,
            node: TextNodeRef::Root,
            range: 0..1,
            style: SpanChoice::Named("Ghost".to_owned()),
        },
    )
    .unwrap();
    let span = first_span(&doc, area);
    let Some(Node::Area(a)) = doc.node(area).cloned() else {
        panic!("no area")
    };
    let Content::Text(mut text) = a.content else {
        panic!("no text")
    };
    text.style = StyleChoice::Named("Gone".to_owned());
    apply(
        &mut doc,
        Op::SetContent {
            area,
            content: Content::Text(text),
        },
    )
    .unwrap();
    assert_eq!(
        doc.breaches(),
        vec![
            Breach::MissingStyle {
                area,
                name: "Gone".to_owned()
            },
            Breach::MissingSpanStyle {
                area,
                span,
                name: "Ghost".to_owned()
            },
        ]
    );
}

/// Removing a span style that sets nothing into a copy is refused while a span names it, since it
/// would leave custom spans that set nothing.
#[test]
fn a_span_style_that_sets_nothing_cannot_be_removed_into_a_copy() {
    let (mut doc, area) = one_text("abc");
    apply(
        &mut doc,
        Op::AddSpanStyle {
            name: "Empty".to_owned(),
            style: SpanStyle::default(),
        },
    )
    .unwrap();
    apply(
        &mut doc,
        Op::WrapSpan {
            area,
            node: TextNodeRef::Root,
            range: 0..1,
            style: SpanChoice::Named("Empty".to_owned()),
        },
    )
    .unwrap();
    let before = doc.clone();
    assert_eq!(
        apply(
            &mut doc,
            Op::RemoveSpanStyle {
                name: "Empty".to_owned(),
                into: Some(Replacement::Custom),
            },
        ),
        Err(OpError::EmptySpanStyle)
    );
    assert_eq!(doc, before);
}

/// Two adjacent spans of one style stay two spans through an edit of the text around them.
#[test]
fn equal_sibling_spans_stay_apart() {
    let (mut doc, area) = one_text("prime number");
    for range in [0..5, 1..8] {
        apply(
            &mut doc,
            Op::WrapSpan {
                area,
                node: TextNodeRef::Root,
                range,
                style: bold(),
            },
        )
        .unwrap();
    }
    // The root is now: span "prime", span " number".
    apply(
        &mut doc,
        Op::InsertText {
            area,
            node: TextNodeRef::Root,
            offset: 2,
            text: "!".to_owned(),
        },
    )
    .unwrap();
    let t = text_of(&doc, area);
    assert_eq!(t.plain_text(), "prime number!");
    let spans = t.root.spans();
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].style, spans[1].style);
}

/// A span naming a style of the table whose values equal its parent's is kept.
#[test]
fn a_named_span_equal_to_its_parent_is_kept() {
    let (mut doc, area) = one_text("bold");
    apply(
        &mut doc,
        Op::AddSpanStyle {
            name: "Strong".to_owned(),
            style: SpanStyle {
                bold: Some(true),
                ..SpanStyle::default()
            },
        },
    )
    .unwrap();
    apply(
        &mut doc,
        Op::WrapSpan {
            area,
            node: TextNodeRef::Root,
            range: 0..4,
            style: bold(),
        },
    )
    .unwrap();
    let outer = first_span(&doc, area);
    apply(
        &mut doc,
        Op::WrapSpan {
            area,
            node: TextNodeRef::Span(outer),
            range: 0..4,
            style: SpanChoice::Named("Strong".to_owned()),
        },
    )
    .unwrap();
    assert_eq!(text_of(&doc, area).root.spans().len(), 2);
}

/// A span's font asset the table lacks, and a built-in family the binary lacks, are named by
/// `Document::breaches`.
#[test]
fn a_span_font_the_document_lacks_is_a_breach() {
    let (mut doc, area) = one_text("abc");
    let missing = AssetId(next_id(&doc) - 1);
    let font = |font| {
        SpanChoice::Custom(SpanStyle {
            font: Some(font),
            ..SpanStyle::default()
        })
    };
    for (range, f) in [
        (0..1, FontRef::Asset(missing)),
        (1..2, FontRef::Builtin("serif".to_owned())),
    ] {
        apply(
            &mut doc,
            Op::WrapSpan {
                area,
                node: TextNodeRef::Root,
                range,
                style: font(f),
            },
        )
        .unwrap();
    }
    assert_eq!(
        doc.breaches(),
        vec![
            Breach::MissingAsset {
                area,
                asset: missing
            },
            Breach::UnknownFont {
                area,
                name: "serif".to_owned()
            },
        ]
    );
}

/// A duplicated slide's spans take new ids, as its nodes do.
#[test]
fn a_duplicated_slide_s_spans_take_new_ids() {
    let (mut doc, area) = one_text("abc");
    apply(
        &mut doc,
        Op::WrapSpan {
            area,
            node: TextNodeRef::Root,
            range: 0..1,
            style: bold(),
        },
    )
    .unwrap();
    let slide = doc.slides[0].id;
    apply(&mut doc, Op::DuplicateSlide { slide }).unwrap();
    check_invariants(&doc);
    let span = |i: usize| {
        let Node::Area(Area {
            content: Content::Text(t),
            ..
        }) = &doc.slides[i].root
        else {
            panic!("slide {i} is not one text area")
        };
        t.root.spans()[0].id
    };
    assert_ne!(span(0), span(1));
}

/// A span style's font is a use of its asset, and one at or above the counter is refused.
#[test]
fn a_span_style_s_font_is_a_use_and_is_below_the_counter() {
    let (mut doc, _) = one_text("abc");
    let RemoveAssetId(asset) = add_asset(
        &mut doc,
        Asset {
            path: "font.ttf".to_owned(),
        },
    );
    let style = |id| SpanStyle {
        font: Some(FontRef::Asset(id)),
        ..SpanStyle::default()
    };
    apply(
        &mut doc,
        Op::AddSpanStyle {
            name: "Code".to_owned(),
            style: style(asset),
        },
    )
    .unwrap();
    assert_eq!(
        apply(&mut doc, Op::RemoveAsset { id: asset }),
        Err(OpError::AssetInUse(asset))
    );
    let far = AssetId(next_id(&doc) + 10);
    assert!(matches!(
        apply(
            &mut doc,
            Op::SetSpanStyleValues {
                name: "Code".to_owned(),
                style: style(far),
            },
        ),
        Err(OpError::Invalid(_))
    ));
}

/// A span whose custom style sets nothing, as a file can hold, cannot be cut: the second half
/// would be another span that sets nothing.
#[test]
fn a_span_that_sets_nothing_cannot_be_cut() {
    let (mut doc, area) = one_text("abcd");
    apply(
        &mut doc,
        Op::WrapSpan {
            area,
            node: TextNodeRef::Root,
            range: 0..4,
            style: bold(),
        },
    )
    .unwrap();
    // The span's style emptied as a file can hold it, by a content set whole.
    let Some(Node::Area(a)) = doc.node(area).cloned() else {
        panic!("no area")
    };
    let Content::Text(mut text) = a.content else {
        panic!("no text")
    };
    if let TextItem::Span(span) = &mut text.root.items[0] {
        span.style = SpanChoice::Custom(SpanStyle::default());
    }
    apply(
        &mut doc,
        Op::SetContent {
            area,
            content: Content::Text(text),
        },
    )
    .unwrap();
    let span = first_span(&doc, area);
    assert_eq!(
        apply(
            &mut doc,
            Op::SplitSpan {
                area,
                span,
                offset: 2
            }
        ),
        Err(OpError::EmptySpanStyle)
    );
}
