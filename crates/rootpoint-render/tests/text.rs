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
        let len = t.text.len();
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
                text: "A title in its style".to_owned(),
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
