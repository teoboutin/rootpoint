//! The edge cases the review of the renderer found: sizes, fonts and their faces, glyphs no font
//! has, photographs' orientation, empty texts, seams between areas, and output as PNG.

mod common;

use std::path::Path;

use common::*;
use rootpoint_model::*;
use rootpoint_render::{FontSet, ImageSet, MAX_SIDE, Precision, RenderError, render_slide};

fn render(
    doc: &Document,
    w: u16,
    h: u16,
    fonts: &mut FontSet,
    images: &ImageSet,
) -> rootpoint_render::Rendered {
    render_slide(
        doc,
        doc.slides[0].id,
        w,
        h,
        fonts,
        images,
        Precision::Portable,
    )
    .unwrap()
}

/// A font file installed on the owner's machine, or `None` elsewhere: the tests that need one
/// say so and pass vacuously without it.
fn system_font(path: &str) -> Option<Vec<u8>> {
    let bytes = std::fs::read(Path::new(path)).ok();
    if bytes.is_none() {
        eprintln!("{path} is not installed; the test checks nothing here");
    }
    bytes
}

#[test]
fn a_size_of_zero_or_above_the_limit_is_refused() {
    let (doc, _) = one_slide();
    let slide = doc.slides[0].id;
    for (w, h) in [
        (0, 10),
        (10, 0),
        (MAX_SIDE + 1, 10),
        (10, MAX_SIDE + 1),
        (65409, 1),
    ] {
        let result = render_slide(
            &doc,
            slide,
            w,
            h,
            &mut FontSet::new(),
            &ImageSet::new(),
            Precision::Native,
        );
        assert_eq!(
            result.err(),
            Some(RenderError::BadSize {
                width: w,
                height: h
            })
        );
    }
    assert!(
        render_slide(
            &doc,
            slide,
            MAX_SIDE,
            1,
            &mut FontSet::new(),
            &ImageSet::new(),
            Precision::Native
        )
        .is_ok()
    );
}

#[test]
fn a_character_the_chosen_font_lacks_is_drawn_in_the_built_in_sans() {
    // Quicksand has no Greek; Noto Sans has.
    let Some(quicksand) = system_font("/usr/share/fonts/truetype/quicksand/Quicksand-Regular.ttf")
    else {
        return;
    };
    let (mut doc, root) = one_slide();
    let asset = add_asset(&mut doc, "Quicksand-Regular.ttf");
    let mut fonts = FontSet::new();
    fonts.insert_bytes(asset, quicksand).unwrap();
    let style = TextStyle {
        font: FontRef::Asset(asset),
        size: 200.0,
        ..TextStyle::default()
    };
    set(&mut doc, root, text("AΩ", style));
    let out = render(&doc, 384, 216, &mut fonts, &ImageSet::new());
    assert_eq!(out.report.missing_glyphs, vec![]);
    assert_eq!(out.report.font_fallbacks, vec![]);
}

#[test]
fn a_character_no_font_has_is_reported() {
    let (mut doc, root) = one_slide();
    // Noto Sans has no emoji.
    set(&mut doc, root, text("A \u{1F600}", TextStyle::default()));
    let out = render(&doc, 384, 216, &mut FontSet::new(), &ImageSet::new());
    assert_eq!(out.report.missing_glyphs, vec![root]);
}

#[test]
fn two_assets_of_one_family_stay_two_fonts() {
    let (Some(bold), Some(regular)) = (
        system_font("/usr/share/fonts/truetype/lato/Lato-Bold.ttf"),
        system_font("/usr/share/fonts/truetype/lato/Lato-Regular.ttf"),
    ) else {
        return;
    };
    let pixels = |register_regular: bool| {
        let (mut doc, root) = one_slide();
        let a = add_asset(&mut doc, "Lato-Bold.ttf");
        let b = add_asset(&mut doc, "Lato-Regular.ttf");
        let mut fonts = FontSet::new();
        fonts.insert_bytes(a, bold.clone()).unwrap();
        if register_regular {
            fonts.insert_bytes(b, regular.clone()).unwrap();
        }
        set(
            &mut doc,
            root,
            text(
                "Lato",
                TextStyle {
                    font: FontRef::Asset(a),
                    ..TextStyle::default()
                },
            ),
        );
        render(&doc, 384, 216, &mut fonts, &ImageSet::new())
            .pixmap
            .data_as_u8_slice()
            .to_vec()
    };
    assert_eq!(
        pixels(true),
        pixels(false),
        "registering another file of the family changed the text"
    );
}

#[test]
fn a_builtin_name_other_than_sans_falls_back() {
    let (mut doc, root) = one_slide();
    set(
        &mut doc,
        root,
        text(
            "x",
            TextStyle {
                font: FontRef::Builtin("Noto Sans".into()),
                ..TextStyle::default()
            },
        ),
    );
    let out = render(&doc, 192, 108, &mut FontSet::new(), &ImageSet::new());
    assert_eq!(out.report.font_fallbacks, vec![root]);
}

#[test]
fn bold_and_italic_are_synthesised_for_a_font_with_one_face() {
    let Some(quicksand) = system_font("/usr/share/fonts/truetype/quicksand/Quicksand-Regular.ttf")
    else {
        return;
    };
    let pixels = |bold: bool, italic: bool| {
        let (mut doc, root) = one_slide();
        let asset = add_asset(&mut doc, "Quicksand-Regular.ttf");
        let mut fonts = FontSet::new();
        fonts.insert_bytes(asset, quicksand.clone()).unwrap();
        let style = TextStyle {
            font: FontRef::Asset(asset),
            size: 120.0,
            bold,
            italic,
            ..TextStyle::default()
        };
        set(&mut doc, root, text("Quick", style));
        render(&doc, 384, 216, &mut fonts, &ImageSet::new())
            .pixmap
            .data_as_u8_slice()
            .to_vec()
    };
    let plain = pixels(false, false);
    assert_ne!(pixels(true, false), plain, "bold drew as plain");
    assert_ne!(pixels(false, true), plain, "italic drew as plain");
}

#[test]
fn a_photographs_exif_orientation_is_applied() {
    // A 40×20 JPEG, red on the left and blue on the right, tagged to be shown rotated 90°
    // clockwise: upright it is 20×40, red on top.
    let img = image::RgbImage::from_fn(40, 20, |x, _| {
        if x < 20 {
            image::Rgb([220, 30, 30])
        } else {
            image::Rgb([30, 30, 220])
        }
    });
    let mut jpg = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(
            &mut std::io::Cursor::new(&mut jpg),
            image::ImageFormat::Jpeg,
        )
        .unwrap();
    let mut app1 = b"Exif\0\0MM\0\x2a\0\0\0\x08".to_vec();
    app1.extend([
        0, 1, 0x01, 0x12, 0, 3, 0, 0, 0, 1, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    let len = (app1.len() + 2) as u16;
    let mut photo = vec![0xFF, 0xD8, 0xFF, 0xE1, (len >> 8) as u8, len as u8];
    photo.extend(app1);
    photo.extend(&jpg[2..]);
    let (mut doc, root) = one_slide();
    let asset = add_asset(&mut doc, "photo.jpg");
    set(
        &mut doc,
        root,
        Content::Image(ImageContent {
            asset,
            fit: ImageFit::Fit,
        }),
    );
    let mut images = ImageSet::new();
    images.insert_bytes(asset, &photo).unwrap();
    let out = render(&doc, 192, 108, &mut FontSet::new(), &images);
    let at = |x, y| {
        let p = out.pixmap.sample(x, y);
        [p.r, p.g, p.b]
    };
    // Fitted, the upright image is 54×108 px, centred: white at its sides, red above, blue below.
    assert_eq!(at(20, 54), [255, 255, 255]);
    let (top, bottom) = (at(96, 10), at(96, 100));
    assert!(top[0] > 150 && top[2] < 100, "top is {top:?}, not red");
    assert!(
        bottom[2] > 150 && bottom[0] < 100,
        "bottom is {bottom:?}, not blue"
    );
}

#[test]
fn the_lines_of_an_empty_text_stay_within_it() {
    let (mut doc, root) = one_slide();
    set(&mut doc, root, text("", TextStyle::default()));
    let out = render(&doc, 192, 108, &mut FontSet::new(), &ImageSet::new());
    for (_, lines) in &out.report.lines {
        for r in lines {
            assert_eq!(r.end, 0, "line {r:?} of an empty text");
        }
    }
}

#[test]
fn adjacent_areas_leave_no_seam_at_a_fractional_edge() {
    let red = image::RgbaImage::from_pixel(4, 4, image::Rgba([200, 0, 0, 255]));
    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(red)
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .unwrap();
    let (mut doc, left) = one_slide();
    split(&mut doc, left, Axis::Vertical, 1.0 / 3.0);
    let right = doc.slides[0].root.areas()[1].id;
    let asset = add_asset(&mut doc, "red.png");
    for area in [left, right] {
        set(
            &mut doc,
            area,
            Content::Image(ImageContent {
                asset,
                fit: ImageFit::Fill,
            }),
        );
    }
    let mut images = ImageSet::new();
    images.insert_bytes(asset, &png).unwrap();
    // At 1000 px wide the divider falls at 333.3 px.
    let out = render(&doc, 1000, 562, &mut FontSet::new(), &images);
    for x in 330..337 {
        let p = out.pixmap.sample(x, 200);
        assert_eq!([p.r, p.g, p.b, p.a], [200, 0, 0, 255], "pixel {x}");
    }
}

#[test]
fn a_slide_with_text_renders_the_same_twice() {
    let (doc, asset) = mixed();
    let mut images = ImageSet::new();
    images.insert_bytes(asset, &two_colour_png()).unwrap();
    let mut fonts = FontSet::new();
    let once = render(&doc, 960, 540, &mut fonts, &images)
        .pixmap
        .data_as_u8_slice()
        .to_vec();
    let again = render(&doc, 960, 540, &mut fonts, &images)
        .pixmap
        .data_as_u8_slice()
        .to_vec();
    let fresh = render(&doc, 960, 540, &mut FontSet::new(), &images)
        .pixmap
        .data_as_u8_slice()
        .to_vec();
    assert_eq!(once, again);
    assert_eq!(once, fresh);
}

#[test]
fn a_render_writes_a_png_that_reads_back_the_same() {
    let doc = alignments();
    let out = render(&doc, 384, 216, &mut FontSet::new(), &ImageSet::new());
    let pixels = out.pixmap.data_as_u8_slice().to_vec();
    let png = out.png();
    let back = vello_cpu::Pixmap::from_png(std::io::Cursor::new(png)).unwrap();
    assert_eq!(back.data_as_u8_slice(), &pixels[..]);
}
