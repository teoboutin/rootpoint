//! Ids survive save and load; a headless script renders the renderer's golden image; a render's
//! reply carries its report; a save leaves every asset readable beside the saved file, and
//! overwrites no other file.

mod common;

use common::*;
use rootpoint_model::AssetSource;
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

// The reproduction of the issue the save rule closed: an asset added before a first save into
// another directory is read beside the saved file once the save has copied it there.
#[test]
fn a_save_into_another_directory_keeps_every_asset_readable() {
    let mut files = MemoryFiles::default();
    files.files.insert("pic.png".into(), two_colour_png());
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nasset pic.png\nimage n2 a3 fit",
    );
    assert_eq!(
        ok(&mut s, &mut files, "render s1 out.png 192 108"),
        "fallback=- missing=- glyphs=-"
    );
    ok(&mut s, &mut files, "save talk/deck.ron");
    assert_eq!(files.files.get("talk/pic.png"), Some(&two_colour_png()));
    assert_eq!(
        ok(&mut s, &mut files, "render s1 out.png 192 108"),
        "fallback=- missing=- glyphs=-"
    );
    let mut t = Session::new();
    ok(&mut t, &mut files, "load talk/deck.ron");
    assert_eq!(t.document(), s.document());
    assert_eq!(
        ok(&mut t, &mut files, "render s1 talk/out.png 192 108"),
        "fallback=- missing=- glyphs=-"
    );
}

/// The source of each asset, by id, as the protocol prints ids.
fn sources(s: &Session) -> Vec<(String, AssetSource)> {
    s.document()
        .assets
        .iter()
        .map(|(id, a)| (id.to_string(), a.source.clone()))
        .collect()
}

fn path(p: &str) -> AssetSource {
    AssetSource::Path(p.to_owned())
}

#[test]
fn a_save_keeps_the_files_inside_its_directory_and_copies_the_others() {
    let mut files = MemoryFiles::default();
    for name in ["a/talk/img/logo.png", "a/cat.png", "Pictures/cat.png"] {
        files.files.insert(name.into(), name.as_bytes().to_vec());
    }
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nsave a/first.ron\nasset talk/img/logo.png\nasset ../Pictures/cat.png\nasset cat.png",
    );
    ok(&mut s, &mut files, "save a/talk/deck.ron");
    // Inside the directory at any depth: kept, its path rewritten. Outside: copied under its
    // file name; a second file of that name and other bytes takes a suffix.
    assert_eq!(
        sources(&s),
        vec![
            ("a3".into(), path("img/logo.png")),
            ("a4".into(), path("cat.png")),
            ("a5".into(), path("cat-2.png")),
        ]
    );
    assert_eq!(files.files["a/talk/cat.png"], b"Pictures/cat.png");
    assert_eq!(files.files["a/talk/cat-2.png"], b"a/cat.png");
    assert!(!files.files.contains_key("a/talk/logo.png"));
    assert_eq!(s.directory(), std::path::Path::new("a/talk"));
}

#[test]
fn a_save_writes_an_embedded_asset_as_a_file() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    ok(&mut s, &mut files, "sample");
    let embedded: Vec<_> = sources(&s);
    ok(&mut s, &mut files, "save out/sample.ron");
    for ((id, before), (_, after)) in embedded.iter().zip(sources(&s)) {
        let AssetSource::Embedded(bytes) = before else {
            panic!("{id} was not embedded");
        };
        let AssetSource::Path(name) = after else {
            panic!("{id} is still embedded");
        };
        assert_eq!(&files.files[&format!("out/{name}")], bytes);
    }
    let text = String::from_utf8(files.files["out/sample.ron"].clone()).unwrap();
    assert!(!text.contains("Embedded"), "{text}");
}

#[test]
fn a_save_overwrites_no_file_of_other_bytes() {
    let mut files = MemoryFiles::default();
    files.files.insert("cat.png".into(), b"mine".to_vec());
    files.files.insert("out/cat.png".into(), b"theirs".to_vec());
    files
        .files
        .insert("out/cat-2.png".into(), b"also theirs".to_vec());
    files.files.insert("dog.png".into(), b"same".to_vec());
    files.files.insert("out/dog.png".into(), b"same".to_vec());
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nasset cat.png\nasset dog.png",
    );
    ok(&mut s, &mut files, "save out/deck.ron");
    assert_eq!(files.files["out/cat.png"], b"theirs");
    assert_eq!(files.files["out/cat-2.png"], b"also theirs");
    assert_eq!(files.files["out/cat-3.png"], b"mine");
    // A file of the same bytes is used as it is.
    assert_eq!(
        sources(&s),
        vec![
            ("a3".into(), path("cat-3.png")),
            ("a4".into(), path("dog.png"))
        ]
    );
}

#[test]
fn a_save_that_cannot_read_an_asset_writes_nothing() {
    let mut files = MemoryFiles::default();
    files.files.insert("cat.png".into(), b"cat".to_vec());
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nasset cat.png\nasset gone.png",
    );
    let doc = s.document().clone();
    let before = files.files.clone();
    let reply = s.execute("save out/deck.ron", &mut files);
    assert!(
        matches!(&reply, Some(Reply::Err(e)) if e.starts_with("a4: ")),
        "{reply:?}"
    );
    assert_eq!(files.files, before);
    assert_eq!(s.document(), &doc);
    assert_eq!(s.directory(), std::path::Path::new(""));
}

#[test]
fn an_undo_after_a_save_restores_an_asset_that_names_its_file() {
    let mut files = MemoryFiles::default();
    files.files.insert("cat.png".into(), two_colour_png());
    let mut s = Session::new();
    // a3 is removed before the save, so only the undo stack holds it.
    let build = "add-slide\nasset cat.png\nimage n2 a3 fit\nclear n2\nremove-asset a3";
    run(&mut s, &mut files, build);
    ok(&mut s, &mut files, "save out/deck.ron");
    run(&mut s, &mut files, "undo\nundo");
    assert_eq!(sources(&s), vec![("a3".into(), path("../cat.png"))]);
    assert_eq!(
        ok(&mut s, &mut files, "render s1 out.png 192 108"),
        "fallback=- missing=- glyphs=-"
    );
    // Where no path names the file from the new directory, the stack holds its bytes.
    let mut s = Session::new();
    run(&mut s, &mut files, build);
    ok(&mut s, &mut files, "save ../elsewhere/deck.ron");
    run(&mut s, &mut files, "undo\nundo");
    assert_eq!(
        sources(&s),
        vec![("a3".into(), AssetSource::Embedded(two_colour_png()))]
    );
}
