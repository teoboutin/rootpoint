//! An empty slide is white, scaled uniformly into the image and centred on black.

use rootpoint_model::{Document, Op, apply};
use rootpoint_render::{FontSet, ImageSet, Precision, ToPixels, render_slide, slide_to_pixels};

fn one_slide() -> Document {
    let mut doc = Document::default();
    apply(&mut doc, Op::AddSlide { at: 0 }).unwrap();
    doc
}

fn pixel(px: &vello_cpu::Pixmap, x: u16, y: u16) -> [u8; 4] {
    let p = px.sample(x, y);
    [p.r, p.g, p.b, p.a]
}

#[test]
fn an_empty_slide_fills_a_16_9_image_with_white() {
    let doc = one_slide();
    let out = render_slide(
        &doc,
        doc.slides[0].id,
        192,
        108,
        &mut FontSet::new(),
        &ImageSet::new(),
        Precision::Portable,
    )
    .unwrap();
    for (x, y) in [(0, 0), (191, 0), (0, 107), (191, 107), (96, 54)] {
        assert_eq!(
            pixel(&out.pixmap, x, y),
            [255, 255, 255, 255],
            "pixel {x},{y}"
        );
    }
}

#[test]
fn a_square_image_letterboxes_the_slide_in_black() {
    let doc = one_slide();
    // A 16:9 slide in a 160×160 image is 160×90, centred: rows 35 to 124 are white.
    let out = render_slide(
        &doc,
        doc.slides[0].id,
        160,
        160,
        &mut FontSet::new(),
        &ImageSet::new(),
        Precision::Portable,
    )
    .unwrap();
    assert_eq!(pixel(&out.pixmap, 80, 10), [0, 0, 0, 255]);
    assert_eq!(pixel(&out.pixmap, 80, 80), [255, 255, 255, 255]);
    assert_eq!(pixel(&out.pixmap, 80, 150), [0, 0, 0, 255]);
}

#[test]
fn slide_to_pixels_names_where_the_slide_is_drawn() {
    let doc = one_slide();
    // 1920×1080 slide units in a 160×160 image: a scale of 1/12, rows 35 to 124.
    assert_eq!(
        slide_to_pixels(&doc, 160, 160),
        ToPixels {
            scale: 160.0 / 1920.0,
            x: 0.0,
            y: 35.0
        }
    );
    // In a 300×108 image, columns 54 to 245.
    let t = slide_to_pixels(&doc, 300, 108);
    assert_eq!((t.scale, t.x, t.y), (0.1, 54.0, 0.0));
    let out = render_slide(
        &doc,
        doc.slides[0].id,
        300,
        108,
        &mut FontSet::new(),
        &ImageSet::new(),
        Precision::Portable,
    )
    .unwrap();
    assert_eq!(pixel(&out.pixmap, 53, 54), [0, 0, 0, 255]);
    assert_eq!(pixel(&out.pixmap, 54, 54), [255, 255, 255, 255]);
    assert_eq!(pixel(&out.pixmap, 245, 54), [255, 255, 255, 255]);
    assert_eq!(pixel(&out.pixmap, 246, 54), [0, 0, 0, 255]);
}

/// The time of an empty 3840×2160 render, the baseline of AC2's 4K bound, of the
/// milestone `milestone@plans@first-prototype`. Run with
/// `cargo test --release -p rootpoint-render --test empty -- --ignored --nocapture`.
#[test]
#[ignore]
fn time_an_empty_4k_render() {
    let doc = one_slide();
    let mut times = Vec::new();
    for _ in 0..10 {
        let start = std::time::Instant::now();
        render_slide(
            &doc,
            doc.slides[0].id,
            3840,
            2160,
            &mut FontSet::new(),
            &ImageSet::new(),
            Precision::Native,
        )
        .unwrap();
        times.push(start.elapsed());
    }
    times.sort();
    println!(
        "empty 3840x2160 render: min {:?}, max {:?}",
        times[0], times[9]
    );
}
