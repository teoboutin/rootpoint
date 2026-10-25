//! Ids survive a save and an open; a headless script renders the renderer's golden image; a
//! render's reply carries its report.

mod common;

use common::*;
use rootpoint_protocol::{Reply, Session};

#[test]
fn ids_survive_save_and_open() {
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
            "s1 n6:stacked(0.5)[n3 text [] style=\"Body\" | n5 text [\"by id\"] style=\"Body\"]"
                .into()
        ))
    );
}

#[test]
fn a_headless_script_renders_the_renderers_golden_image() {
    let dir = scratch("golden");
    std::fs::create_dir_all(dir.join("talk")).unwrap();
    std::fs::write(dir.join("talk/two-colours.png"), two_colour_png()).unwrap();
    let out = headless(&script("golden.txt"), &dir);
    assert!(
        out.ends_with(
            "ok fallback=- missing=- glyphs=- styles=- span-styles=- commands=- refs=-\n"
        ),
        "{out}"
    );
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
    // The file's bytes are no image, so the image cannot be drawn.
    let mut files = MemoryFiles::holding(&["nowhere.png"]);
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nsplit n2 beside 0.5\nimport nowhere.png\nimage n2 a5 fit\ntext n3 \"A \u{1F600}\"",
    );
    assert_eq!(
        ok(&mut s, &mut files, "render s1 out.png 192 108"),
        "fallback=- missing=a5 glyphs=n3 styles=- span-styles=- commands=- refs=-"
    );
    assert!(files.files.contains_key("out.png"));
}
