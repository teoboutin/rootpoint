//! A text's formulas and references: each formula is an inline box of the text's layout, drawn in
//! the math font at its text node's size and colour, after which the map's next caret position
//! stands; each reference draws its equation's number, or `(??)`, which the report names when the
//! area it names holds no equation.

mod common;

use common::*;
use rootpoint_math::Formula;
use rootpoint_model::*;
use rootpoint_render::{FontSet, ImageSet, Precision, area_text, render_slide};

fn formula(text: &str) -> Formula {
    Formula::parse(text).unwrap()
}

fn insert(doc: &mut Document, area: NodeId, node: TextNodeRef, offset: usize, item: TextItem) {
    let op = match item {
        TextItem::Math(formula) => Op::InsertMath {
            area,
            node,
            offset,
            formula,
        },
        TextItem::Ref(target) => Op::InsertRef {
            area,
            node,
            offset,
            target,
        },
        TextItem::Chars(text) => Op::InsertText {
            area,
            node,
            offset,
            text,
        },
        TextItem::Span(_) => unreachable!(),
    };
    apply(doc, op).unwrap();
}

/// One slide: a text above two equations side by side, (1) and (2). The text holds a formula, a
/// red span at 60 slide units holding another, a reference to each equation, and a reference to the
/// text's own area, which holds no equation.
fn slide() -> (Document, NodeId, [NodeId; 2]) {
    let (mut doc, text) = one_slide();
    split(&mut doc, text, Axis::TopToBottom, 0.55);
    let below = doc.slides[0].root.areas()[1].id;
    split(&mut doc, below, Axis::LeftToRight, 0.5);
    let areas: Vec<NodeId> = doc.slides[0].root.areas().iter().map(|a| a.id).collect();
    let equations = [areas[1], areas[2]];
    for (area, latex) in equations
        .iter()
        .zip([r"e^{i\pi} + 1 = 0", r"\sum_{k=1}^{n} k = \frac{n(n+1)}{2}"])
    {
        let content = Content::Equation(doc.new_equation(formula(latex)));
        set(&mut doc, *area, content);
    }
    set(
        &mut doc,
        text,
        Content::Text(TextContent::plain(
            "Let  and take  as in , then  and the unknown .",
            StyleChoice::Custom(TextStyle::default()),
        )),
    );
    // Offsets of the gaps, from the end, so that each insertion leaves the earlier ones in place.
    insert(&mut doc, text, TextNodeRef::Root, 45, TextItem::Ref(text));
    insert(
        &mut doc,
        text,
        TextNodeRef::Root,
        28,
        TextItem::Ref(equations[1]),
    );
    insert(
        &mut doc,
        text,
        TextNodeRef::Root,
        21,
        TextItem::Ref(equations[0]),
    );
    apply(
        &mut doc,
        Op::WrapSpan {
            area: text,
            node: TextNodeRef::Root,
            range: 14..14,
            style: SpanChoice::Custom(SpanStyle {
                size: Some(60.0),
                colour: Some(Rgba {
                    r: 200,
                    g: 30,
                    b: 30,
                    a: 255,
                }),
                ..SpanStyle::default()
            }),
        },
    )
    .unwrap();
    let Some(Node::Area(Area {
        content: Content::Text(t),
        ..
    })) = doc.node(text)
    else {
        panic!()
    };
    let span = t.root.spans()[0].id;
    insert(
        &mut doc,
        text,
        TextNodeRef::Span(span),
        0,
        TextItem::Math(formula(r"\sqrt{x^2 + y^2}")),
    );
    insert(
        &mut doc,
        text,
        TextNodeRef::Root,
        4,
        TextItem::Math(formula(r"x \in \mathbb{R}")),
    );
    (doc, text, equations)
}

fn render(doc: &Document, width: u16, height: u16) -> rootpoint_render::Rendered {
    render_slide(
        doc,
        doc.slides[0].id,
        width,
        height,
        &mut FontSet::new(),
        &ImageSet::new(),
        Precision::Portable,
    )
    .unwrap()
}

#[test]
fn a_text_of_formulas_and_references_matches_its_golden_image() {
    let (doc, text, _) = slide();
    let out = render(&doc, 960, 540);
    assert_eq!(out.report.missing_glyphs, vec![]);
    assert_eq!(out.report.unknown_commands, vec![]);
    assert_eq!(out.report.dangling_references, vec![(text, text)]);
    assert_golden(out.pixmap, "inline-math.png");
}

/// The positions before and after a formula stand at the byte indices either side of its mark, the
/// box at the first, and the text after it starts past the box.
#[test]
fn the_map_counts_a_formula_as_one_offset_either_side_of_its_box() {
    let (mut doc, area) = one_slide();
    set(
        &mut doc,
        area,
        Content::Text(TextContent::plain(
            "ab",
            StyleChoice::Custom(TextStyle::default()),
        )),
    );
    insert(
        &mut doc,
        area,
        TextNodeRef::Root,
        1,
        TextItem::Math(formula(r"\frac{1}{2}")),
    );
    let t = area_text(&doc, doc.slides[0].id, area, &mut FontSet::new()).unwrap();
    let at = |offset| {
        t.map
            .byte_index(CaretPos {
                node: TextNodeRef::Root,
                offset,
            })
            .unwrap()
    };
    assert_eq!(t.formulas.len(), 1);
    let f = &t.formulas[0];
    assert_eq!((at(1), at(2)), (f.range.start, f.range.end));
    assert!(at(2) > at(1));
    assert_eq!(t.map.positions(TextNodeRef::Root), Some(4));
    // The caret after the formula stands past its box, and before it at the box's left edge.
    let x = |byte| {
        parley::Cursor::from_byte_index(&t.layout, byte, parley::Affinity::Downstream)
            .geometry(&t.layout, 1.0)
            .x0
    };
    assert!(
        (x(at(1)) - f.origin.0).abs() < 1e-3,
        "{} {}",
        x(at(1)),
        f.origin.0
    );
    assert!(x(at(2)) >= f.origin.0 + f.layout.width - 1e-3);
}
