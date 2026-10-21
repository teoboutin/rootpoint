//! Ids survive save and load; a headless script renders the renderer's golden image; a render's
//! reply carries its report.

mod common;

use common::*;
use rootpoint_protocol::{Reply, Session};

/// A 40×20 PNG: its left half red, its right half blue, as the renderer's fixture image.
fn two_colour_png() -> Vec<u8> {
    let img = image::RgbaImage::from_fn(40, 20, |x, _| {
        if x < 20 {
            image::Rgba([220, 30, 30, 255])
        } else {
            image::Rgba([30, 30, 220, 255])
        }
    });
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    bytes
}

#[test]
fn ids_survive_save_and_load() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    let replies = run(&mut s, &mut files, &script("ids-build.txt"));
    assert!(
        replies.iter().all(|r| matches!(r, Reply::Ok(_))),
        "{replies:?}"
    );
    let mut t = Session::new();
    // Addressed by the ids printed before the save.
    let after = run(&mut t, &mut files, &script("ids-readdress.txt"));
    assert!(after.iter().all(|r| matches!(r, Reply::Ok(_))), "{after:?}");
    assert_eq!(
        after.last(),
        Some(&Reply::Ok(
            "s1 n6:stacked(0.5)[n3 empty | n5 text \"by id\"]".into()
        ))
    );
}

#[test]
fn a_headless_script_renders_the_renderers_golden_image() {
    let dir = scratch("golden");
    std::fs::create_dir_all(dir.join("talk")).unwrap();
    std::fs::write(dir.join("talk/two-colours.png"), two_colour_png()).unwrap();
    let out = headless(&script("golden.txt"), &dir);
    assert!(out.ends_with("ok fallback=- missing=- glyphs=-\n"), "{out}");
    let golden = image::open(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../rootpoint-render/tests/golden/fit-and-fill.png"),
    )
    .unwrap()
    .to_rgba8();
    let rendered = image::open(dir.join("out.png")).unwrap().to_rgba8();
    assert_eq!(rendered.dimensions(), golden.dimensions());
    let differ = rendered
        .pixels()
        .zip(golden.pixels())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        differ, 0,
        "{differ} pixels differ from the renderer's golden image"
    );
}

#[test]
fn a_render_reports_what_it_could_not_draw() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nsplit n2 beside 0.5\nasset nowhere.png\nimage n2 a5 fit\ntext n3 \"A \u{1F600}\"",
    );
    assert_eq!(
        ok(&mut s, &mut files, "render s1 out.png 192 108"),
        "fallback=- missing=a5 glyphs=n3"
    );
    assert!(files.files.contains_key("out.png"));
}

// The asset is added before the first save, and its file is found only because the save sets the
// directory it is read against: `issue@rootpoint-protocol@asset-paths-follow-save`.
#[test]
fn assets_are_read_beside_the_presentation() {
    let mut files = MemoryFiles::default();
    files
        .files
        .insert("talk/two-colours.png".into(), two_colour_png());
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nasset two-colours.png\nimage n2 a3 fit\nsave talk/deck.ron",
    );
    let mut t = Session::new();
    ok(&mut t, &mut files, "load talk/deck.ron");
    assert_eq!(
        ok(&mut t, &mut files, "render s1 talk/out.png 192 108"),
        "fallback=- missing=- glyphs=-"
    );
}
