//! Text is laid out once in slide units and drawn at any size; fonts come only from the built-in
//! set and the document's assets.

mod common;

use common::*;
use rootpoint_model::*;
use rootpoint_render::{FontSet, ImageSet, Precision, render_slide};

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

/// AC2's 4K bound, of the milestone `milestone@plans@first-prototype`: 10 renders of the mixed slide at 3840×2160, each within 200 ms. Run in a
/// release build with `cargo test --release -p rootpoint-render --test text -- --ignored
/// --nocapture`.
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
        "a render took {max:?}, over AC2's 200 ms"
    );
}
