//! Images are drawn into their areas, fitted or filling them; a slide renders the same twice; and
//! the pixels match the committed golden images.

mod common;

use common::*;
use rootpoint_model::*;
use rootpoint_render::{FontSet, ImageSet, Precision, render_slide};

/// One slide, a side-by-side split: the image fitted in the left half, filling the right half.
fn fit_and_fill() -> (Document, ImageSet, AssetId) {
    let (mut doc, left) = one_slide();
    split(&mut doc, left, Axis::LeftToRight, 0.5);
    let right = doc.slides[0].root.areas()[1].id;
    let asset = add_asset(&mut doc, "two-colours.png");
    for (area, fit) in [(left, ImageFit::Fit), (right, ImageFit::Fill)] {
        set(&mut doc, area, Content::Image(ImageContent { asset, fit }));
    }
    let mut images = ImageSet::new();
    images.insert_bytes(asset, &two_colour_png()).unwrap();
    (doc, images, asset)
}

fn rgba(px: &vello_cpu::Pixmap, x: u16, y: u16) -> [u8; 4] {
    let p = px.sample(x, y);
    [p.r, p.g, p.b, p.a]
}

#[test]
fn fit_shows_the_whole_image_and_fill_covers_the_area() {
    let (doc, images, _) = fit_and_fill();
    // 384×216: each half is 192×216. The 2:1 image fits as 192×96, rows 60 to 155, and fills as
    // 432×216, cropped to its middle 192 columns: red on their left, blue on their right.
    let out = render_slide(
        &doc,
        doc.slides[0].id,
        384,
        216,
        &mut FontSet::new(),
        &images,
        Precision::Portable,
    )
    .unwrap();
    let px = &out.pixmap;
    assert_eq!(rgba(px, 48, 108), [220, 30, 30, 255], "fitted, red half");
    assert_eq!(rgba(px, 144, 108), [30, 30, 220, 255], "fitted, blue half");
    assert_eq!(
        rgba(px, 96, 20),
        [255, 255, 255, 255],
        "fitted, white above"
    );
    assert_eq!(
        rgba(px, 200, 5),
        [220, 30, 30, 255],
        "filled, top left corner"
    );
    assert_eq!(
        rgba(px, 380, 210),
        [30, 30, 220, 255],
        "filled, bottom right corner"
    );
    assert_eq!(out.report.missing_images, vec![]);
}

#[test]
fn an_image_with_no_decoded_bytes_is_drawn_grey_and_reported() {
    let (doc, _, asset) = fit_and_fill();
    let out = render_slide(
        &doc,
        doc.slides[0].id,
        384,
        216,
        &mut FontSet::new(),
        &ImageSet::new(),
        Precision::Portable,
    )
    .unwrap();
    assert_eq!(rgba(&out.pixmap, 96, 20), [224, 224, 224, 255]);
    assert_eq!(out.report.missing_images, vec![asset, asset]);
}

#[test]
fn a_slide_renders_the_same_twice() {
    let (doc, images, _) = fit_and_fill();
    let render = || {
        render_slide(
            &doc,
            doc.slides[0].id,
            384,
            216,
            &mut FontSet::new(),
            &images,
            Precision::Native,
        )
        .unwrap()
        .pixmap
        .data_as_u8_slice()
        .to_vec()
    };
    assert_eq!(render(), render());
}

#[test]
fn fit_and_fill_match_their_golden_image() {
    let (doc, images, _) = fit_and_fill();
    let out = render_slide(
        &doc,
        doc.slides[0].id,
        384,
        216,
        &mut FontSet::new(),
        &images,
        Precision::Portable,
    )
    .unwrap();
    assert_golden(out.pixmap, "fit-and-fill.png");
}
