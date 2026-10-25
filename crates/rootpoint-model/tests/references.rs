//! A text's references to equation areas: they follow a swapped equation and a duplicated slide's
//! copies, a retarget moves back only what it moved, no undo is refused, a dangling one is a
//! cosmetic breach, and one at or above the id counter is structural.

use rootpoint_math::Formula;
use rootpoint_model::*;

/// One slide split beside: n2, a text, and its new neighbour, an equation, then a third area, a
/// second equation, below n2. Returns the document, the text and the two equations.
fn doc() -> (Document, NodeId, NodeId, NodeId) {
    let mut doc = Document::default();
    apply(&mut doc, Op::AddSlide { at: 0 }).unwrap();
    let text = doc.slides[0].root.id();
    let split = |doc: &mut Document, node, axis| {
        apply(
            doc,
            Op::Split {
                node,
                axis,
                at: 0.5,
                side: Side::After,
            },
        )
        .unwrap();
    };
    split(&mut doc, text, Axis::LeftToRight);
    let first = doc.slides[0].root.areas()[1].id;
    split(&mut doc, text, Axis::TopToBottom);
    let second = doc.slides[0].root.areas()[1].id;
    for area in [first, second] {
        let content = Content::Equation(doc.new_equation(Formula::parse("x").unwrap()));
        apply(&mut doc, Op::SetContent { area, content }).unwrap();
    }
    (doc, text, first, second)
}

fn refer(doc: &mut Document, area: NodeId, target: NodeId) {
    apply(
        doc,
        Op::InsertRef {
            area,
            node: TextNodeRef::Root,
            offset: 0,
            target,
        },
    )
    .unwrap();
}

fn refs_of(doc: &Document, area: NodeId) -> Vec<NodeId> {
    match doc.node(area) {
        Some(Node::Area(Area {
            content: Content::Text(t),
            ..
        })) => t.root.refs(),
        other => panic!("{area} holds no text: {other:?}"),
    }
}

/// A swap carries each equation's references with it, so a moved equation keeps them.
#[test]
fn a_swapped_equation_keeps_its_references() {
    let (mut doc, text, first, second) = doc();
    refer(&mut doc, text, first);
    let inverse = apply(
        &mut doc,
        Op::Swap {
            a: first,
            b: second,
        },
    )
    .unwrap();
    assert_eq!(refs_of(&doc, text), [second]);
    apply(&mut doc, inverse).unwrap();
    assert_eq!(refs_of(&doc, text), [first]);
}

/// A duplicated slide's references to its own equations name the copies; the others keep their
/// target.
#[test]
fn a_duplicated_slide_s_references_follow_the_copy() {
    let (mut doc, text, first, _) = doc();
    apply(&mut doc, Op::AddSlide { at: 1 }).unwrap();
    let elsewhere = doc.slides[1].root.id();
    let content = Content::Equation(doc.new_equation(Formula::parse("y").unwrap()));
    apply(
        &mut doc,
        Op::SetContent {
            area: elsewhere,
            content,
        },
    )
    .unwrap();
    refer(&mut doc, text, first);
    refer(&mut doc, text, elsewhere);
    let slide = doc.slides[0].id;
    let inverse = apply(&mut doc, Op::DuplicateSlide { slide }).unwrap();
    let copy = &doc.slides[1];
    let areas: Vec<NodeId> = copy.root.areas().iter().map(|a| a.id).collect();
    // The copy's areas in the order of the original's: the text, the second equation, the first.
    let [copied_text, _, copied_first] = areas[..] else {
        panic!("three areas in the copy");
    };
    assert_eq!(refs_of(&doc, copied_text), [elsewhere, copied_first]);
    assert_eq!(refs_of(&doc, text), [elsewhere, first]);
    // Its undo removes the copy, which no check of the references refuses.
    apply(&mut doc, inverse).unwrap();
    assert_eq!(doc.slides.len(), 2);
}

/// An undo is never refused for a reference: the removal of the duplicate of a slide whose
/// equation another text references, and the removal of a referenced equation, both apply, the
/// latter leaving a cosmetic breach.
#[test]
fn an_undo_is_never_refused_by_a_reference() {
    let (mut doc, text, first, _) = doc();
    refer(&mut doc, text, first);
    apply(&mut doc, Op::Remove { node: first }).unwrap();
    assert_eq!(
        doc.breaches(),
        [Breach::DanglingReference {
            area: text,
            target: first
        }]
    );
}

/// A retarget moves every reference to `from`, and its undo moves back only those, so the target's
/// own references stay.
#[test]
fn retarget_is_undone_by_moving_back_only_what_it_moved() {
    let (mut doc, text, first, second) = doc();
    refer(&mut doc, text, second);
    refer(&mut doc, text, first);
    let inverse = apply(
        &mut doc,
        Op::RetargetReferences {
            from: first,
            to: second,
        },
    )
    .unwrap();
    assert_eq!(refs_of(&doc, text), [second, second]);
    apply(&mut doc, inverse).unwrap();
    assert_eq!(refs_of(&doc, text), [first, second]);
}

/// A reference at or above the id counter would name a later area: `apply` refuses it, and so does
/// the loader.
#[test]
fn a_reference_at_or_above_the_counter_is_structural() {
    let (mut doc, text, first, _) = doc();
    let before = doc.clone();
    let beyond = NodeId(10_000);
    let op = Op::InsertRef {
        area: text,
        node: TextNodeRef::Root,
        offset: 0,
        target: beyond,
    };
    assert!(matches!(apply(&mut doc, op), Err(OpError::Invalid(_))));
    assert_eq!(doc, before);
    refer(&mut doc, text, first);
    let saved = save_ron(&doc);
    assert!(load_ron(&saved).is_ok());
    let file = saved.replace(&format!("Ref({})", first.0), &format!("Ref({})", beyond.0));
    assert_ne!(file, saved);
    assert!(load_ron(&file).is_err());
}

/// `SetMath` replaces the formula at its offset, and refuses an offset that holds none.
#[test]
fn set_math_replaces_a_formula_and_refuses_anything_else() {
    let (mut doc, text, _, _) = doc();
    let x = Formula::parse("x").unwrap();
    let y = Formula::parse("y").unwrap();
    for op in [
        Op::InsertText {
            area: text,
            node: TextNodeRef::Root,
            offset: 0,
            text: "a".to_owned(),
        },
        Op::InsertMath {
            area: text,
            node: TextNodeRef::Root,
            offset: 1,
            formula: x,
        },
    ] {
        apply(&mut doc, op).unwrap();
    }
    let set = |offset| Op::SetMath {
        area: text,
        node: TextNodeRef::Root,
        offset,
        formula: y.clone(),
    };
    assert_eq!(
        apply(&mut doc, set(0)),
        Err(OpError::NotAFormula { offset: 0 })
    );
    apply(&mut doc, set(1)).unwrap();
    let Some(Node::Area(Area {
        content: Content::Text(t),
        ..
    })) = doc.node(text)
    else {
        panic!()
    };
    assert_eq!(t.plain_text(), "a$y$");
}
