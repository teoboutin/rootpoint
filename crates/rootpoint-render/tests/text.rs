//! Text is laid out once in slide units and drawn at any size; fonts come only from the built-in
//! set and the document's assets.

mod common;

use common::*;
use rootpoint_model::*;
use rootpoint_render::{FontSet, ImageSet, Precision, area_text, render_slide};

#[test]
fn the_alignments_match_their_golden_image() {
    let doc = alignments();
    let out = render_slide(
        &doc,
        doc.slides[0].id,
        960,
        540,
        &mut FontSet::new(),
        &ImageSet::new(),
        Precision::Portable,
    )
    .unwrap();
    assert_eq!(out.report.font_fallbacks, vec![]);
    assert_golden(out.pixmap, "alignments.png");
}

#[test]
fn the_mixed_slide_matches_its_golden_image() {
    let (doc, asset) = mixed();
    let mut images = ImageSet::new();
    images.insert_bytes(asset, &two_colour_png()).unwrap();
    let out = render_slide(
        &doc,
        doc.slides[0].id,
        960,
        540,
        &mut FontSet::new(),
        &images,
        Precision::Portable,
    )
    .unwrap();
    assert_golden(out.pixmap, "mixed.png");
}

#[test]
fn the_line_breaks_are_the_same_at_every_size() {
    let (doc, _) = mixed();
    let mut fonts = FontSet::new();
    let lines = |fonts: &mut FontSet, w, h| {
        render_slide(
            &doc,
            doc.slides[0].id,
            w,
            h,
            fonts,
            &ImageSet::new(),
            Precision::Native,
        )
        .unwrap()
        .report
        .lines
    };
    let small = lines(&mut fonts, 960, 540);
    // The paragraph wraps over more than two lines, so the comparison sees its breaks. The test
    // catches a wrap width that depends on the pixel size; it cannot tell a layout made at the pixel
    // size, font size and width both scaled, from one in slide units, since an unhinted layout
    // scales linearly.
    assert!(small.iter().any(|(_, l)| l.len() > 2), "{small:?}");
    assert_eq!(lines(&mut fonts, 1920, 1080), small);
    assert_eq!(lines(&mut fonts, 3840, 2160), small);
}

#[test]
fn a_font_no_font_of_the_set_has_falls_back_and_is_reported() {
    let (mut doc, root) = one_slide();
    // DejaVu Sans is installed on the owner's machine, and is neither built in nor an asset.
    set(
        &mut doc,
        root,
        text(
            "In a system font",
            TextStyle {
                font: FontRef::Builtin("DejaVu Sans".to_owned()),
                ..TextStyle::default()
            },
        ),
    );
    let render = |doc: &Document| {
        render_slide(
            doc,
            doc.slides[0].id,
            192,
            108,
            &mut FontSet::new(),
            &ImageSet::new(),
            Precision::Portable,
        )
        .unwrap()
        .report
        .font_fallbacks
    };
    assert_eq!(render(&doc), vec![root]);
    set(
        &mut doc,
        root,
        text("In the built-in sans", TextStyle::default()),
    );
    assert_eq!(render(&doc), vec![]);
}

/// The 4K bound of `tripwire@rootpoint-render@render-within-bounds`: 10 renders of the mixed
/// slide at 3840×2160, each within 200 ms. Run in a release build with
/// `cargo test --release -p rootpoint-render --test text -- --ignored --nocapture`.
#[test]
#[ignore]
fn time_the_mixed_slide_at_4k() {
    let (doc, asset) = mixed();
    let mut images = ImageSet::new();
    images.insert_bytes(asset, &two_colour_png()).unwrap();
    let mut fonts = FontSet::new();
    let mut times = Vec::new();
    for _ in 0..10 {
        let start = std::time::Instant::now();
        render_slide(
            &doc,
            doc.slides[0].id,
            3840,
            2160,
            &mut fonts,
            &images,
            Precision::Native,
        )
        .unwrap();
        times.push(start.elapsed());
    }
    let max = *times.iter().max().unwrap();
    println!("mixed 3840x2160 renders: {times:?}; largest {max:?}");
    assert!(
        max.as_millis() <= 200,
        "a render took {max:?}, over the 200 ms of `tripwire@rootpoint-render@render-within-bounds`"
    );
}

/// AC5 of the milestone on math, `tripwire@rootpoint-render@render-within-bounds`'s 4K bound on a
/// slide of formulas: a text holding 6 inline formulas above 4 equations, two of them of several
/// rows, 10 renders at 3840×2160, each within 200 ms. Run with the test above.
#[test]
#[ignore]
fn time_a_slide_of_formulas_at_4k() {
    use rootpoint_math::Formula;
    let (mut doc, text) = one_slide();
    split(&mut doc, text, Axis::TopToBottom, 0.3);
    let below = doc.slides[0].root.areas()[1].id;
    split(&mut doc, below, Axis::LeftToRight, 0.5);
    for i in [1, 2] {
        let area = doc.slides[0].root.areas()[i].id;
        split(&mut doc, area, Axis::TopToBottom, 0.5);
    }
    let areas: Vec<NodeId> = doc.slides[0].root.areas().iter().map(|a| a.id).collect();
    assert_eq!(areas.len(), 5);
    let equations = [
        r"\frac{\partial f_i}{\partial t} + \mathbf{c}_i \cdot \nabla_{\mathbf{x}} f_i = \Omega(f_i)",
        r"\int_{-\infty}^{\infty} P_n(x)\,\bar{\omega}(x)\,\mathrm{d}x = \sum_{i=1}^{N} w_i P_n(x_i)",
        r"f(x) &= (x+1)^2 \\ &= x^2 + 2x + 1 \\ &\leq \sqrt{\frac{a^2+b^2}{2}}",
        r"|x| = \begin{cases} x & \text{if } x \geq 0 \\ -x & \text{otherwise} \end{cases}",
    ];
    for (area, latex) in areas[1..].iter().zip(equations) {
        let content = Content::Equation(doc.new_equation(Formula::parse(latex).unwrap()));
        set(&mut doc, *area, content);
    }
    set(
        &mut doc,
        text,
        Content::Text(TextContent::plain(
            "With , ,  and , the bound  holds, and  follows.",
            StyleChoice::Custom(TextStyle::default()),
        )),
    );
    let inline = [
        r"x \in \mathbb{R}",
        r"\alpha_i^2",
        r"\frac{1}{n}",
        r"\sqrt{2}",
        r"\sum_{k} a_k \leq C",
        r"e^{i\theta}",
    ];
    // The gaps of the text, from the last, so that each insertion leaves the earlier ones in place.
    for (offset, latex) in [5, 7, 9, 14, 26, 38].into_iter().zip(inline).rev() {
        apply(
            &mut doc,
            Op::InsertMath {
                area: text,
                node: TextNodeRef::Root,
                offset,
                formula: Formula::parse(latex).unwrap(),
            },
        )
        .unwrap();
    }
    let mut fonts = FontSet::new();
    let mut times = Vec::new();
    for _ in 0..10 {
        let start = std::time::Instant::now();
        let out = render_slide(
            &doc,
            doc.slides[0].id,
            3840,
            2160,
            &mut fonts,
            &ImageSet::new(),
            Precision::Native,
        )
        .unwrap();
        times.push(start.elapsed());
        assert_eq!(out.report.unknown_commands, vec![]);
    }
    let max = *times.iter().max().unwrap();
    println!("formulas 3840x2160 renders: {times:?}; largest {max:?}");
    assert!(
        max.as_millis() <= 200,
        "a render took {max:?}, over the 200 ms of `tripwire@rootpoint-render@render-within-bounds`"
    );
}

#[test]
fn area_text_gives_the_layout_render_slide_draws() {
    let (doc, _) = mixed();
    let slide = doc.slides[0].id;
    let mut fonts = FontSet::new();
    let out = render_slide(
        &doc,
        slide,
        960,
        540,
        &mut fonts,
        &ImageSet::new(),
        Precision::Portable,
    )
    .unwrap();
    let rects = area_rects(&doc.slides[0], doc.aspect);
    for (area, lines) in &out.report.lines {
        let text = area_text(&doc, slide, *area, &mut fonts).unwrap();
        // The same lines as the render laid out. The report holds each range within the text's
        // length, where Parley gives an empty text one line of one byte.
        let Some(Node::Area(Area {
            content: Content::Text(t),
            ..
        })) = doc.slides[0].root.find(*area)
        else {
            panic!("{area} is not a text area");
        };
        let len = t.plain_text().len();
        let got: Vec<_> = text
            .layout
            .lines()
            .map(|l| l.text_range())
            .map(|r| r.start.min(len)..r.end.min(len))
            .collect();
        assert_eq!(&got, lines, "{area}");
        // Its origin is inside the area, inset by the padding on the left.
        let r = rects.iter().find(|(id, _)| id == area).unwrap().1;
        assert_eq!(text.origin.0, r.x + 24.0);
        assert!(text.origin.1 >= r.y + 24.0 && text.origin.1 < r.y + r.h);
    }
    // An area with no text has none.
    let image = doc.slides[0].root.areas()[1].id;
    assert!(area_text(&doc, slide, image, &mut fonts).is_none());
}

/// A text naming a style of the table draws as a text holding that style's values.
#[test]
fn a_named_style_draws_as_its_table_entry() {
    let render = |style: StyleChoice| {
        let (mut doc, root) = one_slide();
        set(
            &mut doc,
            root,
            Content::Text(TextContent {
                root: rootpoint_model::TextNode::plain("A title in its style"),
                style,
            }),
        );
        render_slide(
            &doc,
            doc.slides[0].id,
            384,
            216,
            &mut FontSet::new(),
            &ImageSet::new(),
            Precision::Portable,
        )
        .unwrap()
    };
    let title = Document::default().styles["Title"].clone();
    let named = render(StyleChoice::Named("Title".to_owned()));
    let custom = render(StyleChoice::Custom(title));
    assert_eq!(named.report.missing_styles, vec![]);
    assert!(named.pixmap.data_as_u8_slice() == custom.pixmap.data_as_u8_slice());
    let default = render(StyleChoice::Custom(TextStyle::default()));
    assert!(named.pixmap.data_as_u8_slice() != default.pixmap.data_as_u8_slice());
}

/// A text of a bold span holding a red span, and a red span alone, on one slide.
fn styled() -> Document {
    let (mut doc, root) = one_slide();
    set(
        &mut doc,
        root,
        text(
            "Plain, then bold with a red word inside, then a red word alone.",
            TextStyle {
                size: 40.0,
                ..TextStyle::default()
            },
        ),
    );
    let red = || {
        SpanChoice::Custom(SpanStyle {
            colour: Some(Rgba {
                r: 200,
                g: 30,
                b: 30,
                a: 255,
            }),
            ..SpanStyle::default()
        })
    };
    let wrap = |doc: &mut Document, node, range, style| {
        apply(
            doc,
            Op::WrapSpan {
                area: root,
                node,
                range,
                style,
            },
        )
        .unwrap();
    };
    // "bold with a red word inside" is offsets 12..39; "red word" offsets 12..20 within it.
    wrap(
        &mut doc,
        TextNodeRef::Root,
        12..39,
        SpanChoice::Custom(SpanStyle {
            bold: Some(true),
            ..SpanStyle::default()
        }),
    );
    let bold = text_spans(&doc, root)[0];
    wrap(&mut doc, TextNodeRef::Span(bold), 12..20, red());
    // "red word alone": the root counts the bold span as one offset, 12, so it starts at 22.
    wrap(&mut doc, TextNodeRef::Root, 22..36, red());
    doc
}

fn text_spans(doc: &Document, area: NodeId) -> Vec<SpanId> {
    match doc.node(area) {
        Some(Node::Area(Area {
            content: Content::Text(t),
            ..
        })) => t.root.spans().iter().map(|s| s.id).collect(),
        _ => panic!("no text"),
    }
}

fn render_small(doc: &Document) -> rootpoint_render::Rendered {
    render_slide(
        doc,
        doc.slides[0].id,
        960,
        540,
        &mut FontSet::new(),
        &ImageSet::new(),
        Precision::Portable,
    )
    .unwrap()
}

#[test]
fn a_styled_text_matches_its_golden_image() {
    let doc = styled();
    let root = doc.slides[0].root.id();
    match doc.node(root) {
        Some(Node::Area(Area {
            content: Content::Text(t),
            ..
        })) => assert_eq!(
            t.plain_text(),
            "Plain, then bold with a red word inside, then a red word alone."
        ),
        _ => panic!("no text"),
    }
    assert_eq!(text_spans(&doc, root).len(), 3);
    let out = render_small(&doc);
    assert_eq!(out.report.font_fallbacks, vec![]);
    assert_golden(out.pixmap, "spans.png");
}

/// A span setting bold over a whole text draws as the text whose own style is bold: the span's
/// value replaces the area's over its range.
#[test]
fn a_span_over_the_whole_text_draws_as_the_style_it_sets() {
    let plain = |bold| {
        let (mut doc, root) = one_slide();
        set(
            &mut doc,
            root,
            text(
                "One line of text",
                TextStyle {
                    bold,
                    ..TextStyle::default()
                },
            ),
        );
        (doc, root)
    };
    let (bold, _) = plain(true);
    let (mut spanned, root) = plain(false);
    apply(
        &mut spanned,
        Op::WrapSpan {
            area: root,
            node: TextNodeRef::Root,
            range: 0..16,
            style: SpanChoice::Custom(SpanStyle {
                bold: Some(true),
                ..SpanStyle::default()
            }),
        },
    )
    .unwrap();
    let (a, b) = (render_small(&bold), render_small(&spanned));
    assert!(a.pixmap.data_as_u8_slice() == b.pixmap.data_as_u8_slice());
    let (not_bold, _) = plain(false);
    assert!(render_small(&not_bold).pixmap.data_as_u8_slice() != b.pixmap.data_as_u8_slice());
}

/// Each caret position of a nested text stands at its byte index, and each span covers its
/// bytes; the two positions at an edge stand at one index.
#[test]
fn the_map_places_every_caret_position_on_its_byte_index() {
    let doc = styled();
    let root = doc.slides[0].root.id();
    let Some(Node::Area(Area {
        content: Content::Text(t),
        ..
    })) = doc.node(root)
    else {
        panic!("no text")
    };
    let map = rootpoint_render::TextMap::new(t, &Default::default());
    let plain = map.text().to_owned();
    let spans = text_spans(&doc, root);
    let at = |node, offset| map.byte_index(CaretPos { node, offset }).unwrap();
    assert_eq!(
        &plain[map.span_range(spans[0]).unwrap()],
        "bold with a red word inside"
    );
    assert_eq!(&plain[map.span_range(spans[1]).unwrap()], "red word");
    assert_eq!(&plain[map.span_range(spans[2]).unwrap()], "red word alone");
    // Before the bold span in the root, and at its start inside it: one byte index.
    assert_eq!(
        at(TextNodeRef::Root, 12),
        at(TextNodeRef::Span(spans[0]), 0)
    );
    assert_eq!(at(TextNodeRef::Root, 12), plain.find("bold").unwrap());
    // After the bold span, offset 13 of the root.
    assert_eq!(at(TextNodeRef::Root, 13), plain.find(", then a").unwrap());
    // The root's last offset is the text's end.
    let last = map.positions(TextNodeRef::Root).unwrap() - 1;
    assert_eq!(at(TextNodeRef::Root, last), plain.len());
    assert_eq!(
        map.byte_index(CaretPos {
            node: TextNodeRef::Root,
            offset: last + 1
        }),
        None
    );
}

/// A span naming a span style the table lacks is reported and drawn with its parent's values;
/// a span whose font asset the set lacks falls back to the built-in sans, and its area is named.
#[test]
fn a_span_with_a_missing_style_or_font_is_reported() {
    let (mut doc, root) = one_slide();
    set(&mut doc, root, text("two words", TextStyle::default()));
    let asset = add_asset(&mut doc, "font.ttf");
    let wrap = |doc: &mut Document, range, style| {
        apply(
            doc,
            Op::WrapSpan {
                area: root,
                node: TextNodeRef::Root,
                range,
                style,
            },
        )
        .unwrap();
    };
    wrap(&mut doc, 0..3, SpanChoice::Named("Ghost".to_owned()));
    wrap(
        &mut doc,
        2..3,
        SpanChoice::Custom(SpanStyle {
            font: Some(FontRef::Asset(asset)),
            ..SpanStyle::default()
        }),
    );
    let ghost = text_spans(&doc, root)[0];
    let out = render_small(&doc);
    assert_eq!(
        out.report.missing_span_styles,
        vec![(root, ghost, "Ghost".to_owned())]
    );
    assert_eq!(out.report.font_fallbacks, vec![root]);
}

/// An empty span in the middle of a text, as a style action with no selection opens, draws the
/// text as if it were not there.
#[test]
fn an_empty_span_draws_nothing_and_moves_nothing() {
    let (mut doc, root) = one_slide();
    set(&mut doc, root, text("two words", TextStyle::default()));
    let plain = render_small(&doc);
    apply(
        &mut doc,
        Op::WrapSpan {
            area: root,
            node: TextNodeRef::Root,
            range: 4..4,
            style: SpanChoice::Custom(SpanStyle {
                size: Some(90.0),
                ..SpanStyle::default()
            }),
        },
    )
    .unwrap();
    assert_eq!(text_spans(&doc, root).len(), 1);
    let spanned = render_small(&doc);
    assert!(plain.pixmap.data_as_u8_slice() == spanned.pixmap.data_as_u8_slice());
    assert_eq!(plain.report.lines, spanned.report.lines);
}

/// Every caret position of every text node stands where an independent count puts it: the node's
/// first byte, plus the bytes of the items before the offset, a span counting its whole text.
#[test]
fn every_caret_position_stands_on_its_counted_byte() {
    fn len(node: &TextNode) -> usize {
        node.items
            .iter()
            .map(|i| match i {
                TextItem::Chars(s) => s.len(),
                TextItem::Span(s) => len(&s.text),
                // A formula stands on its mark, U+2060, three bytes; a reference with no number on
                // `(??)`.
                TextItem::Math(_) => 3,
                TextItem::Ref(_) => 4,
            })
            .sum()
    }
    fn check(map: &rootpoint_render::TextMap, node: TextNodeRef, text: &TextNode, start: usize) {
        let mut at = start;
        let mut offset = 0;
        let expect = |offset: usize, at: usize| {
            assert_eq!(
                map.byte_index(CaretPos { node, offset }),
                Some(at),
                "{node:?} {offset}"
            );
        };
        expect(0, at);
        for item in &text.items {
            match item {
                TextItem::Chars(s) => {
                    for c in s.chars() {
                        at += c.len_utf8();
                        offset += 1;
                        expect(offset, at);
                    }
                }
                TextItem::Span(span) => {
                    check(map, TextNodeRef::Span(span.id), &span.text, at);
                    at += len(&span.text);
                    offset += 1;
                    expect(offset, at);
                }
                TextItem::Math(_) | TextItem::Ref(_) => {
                    at += len(&TextNode {
                        items: vec![item.clone()],
                    });
                    offset += 1;
                    expect(offset, at);
                }
            }
        }
        assert_eq!(map.positions(node), Some(offset + 1));
    }
    let mut doc = styled();
    let root = doc.slides[0].root.id();
    // A character of more than one byte inside a span, so that offsets and bytes differ there.
    let bold = text_spans(&doc, root)[0];
    apply(
        &mut doc,
        Op::InsertText {
            area: root,
            node: TextNodeRef::Span(bold),
            offset: 1,
            text: "é".to_owned(),
        },
    )
    .unwrap();
    let Some(Node::Area(Area {
        content: Content::Text(t),
        ..
    })) = doc.node(root)
    else {
        panic!("no text")
    };
    check(
        &rootpoint_render::TextMap::new(t, &Default::default()),
        TextNodeRef::Root,
        &t.root,
        0,
    );
}
