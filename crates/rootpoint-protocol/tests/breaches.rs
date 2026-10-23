//! A document with a cosmetic breach loads, renders with its stand-in and names it, and its undo
//! is not refused; no command creates one.

mod common;

use common::*;
use rootpoint_model::*;
use rootpoint_protocol::{Reply, Session};

/// A session whose document has two areas, n2 a text and n3 an image of `a99`, which the table
/// lacks: a file edited by hand, whose counter, 100, is above the id.
fn loaded_with_a_missing_asset(files: &mut MemoryFiles) -> Session {
    let mut s = Session::new();
    run(
        &mut s,
        files,
        "add-slide\nsplit n2 beside 0.5\nasset cat.png\nimage n3 a5 fit\nsave deck.ron",
    );
    let saved = String::from_utf8(files.files["deck.ron"].clone()).unwrap();
    let edited =
        saved
            .replacen("asset: 5,", "asset: 99,", 1)
            .replacen("next_id: 6,", "next_id: 100,", 1);
    let table =
        "    assets: {\n        5: (\n            source: Path(\"cat.png\"),\n        ),\n    },";
    let edited = edited.replacen(table, "    assets: {},", 1);
    assert!(
        !edited.contains("asset: 5,") && !edited.contains("next_id: 6,") && !edited.contains(table),
        "the saved file changed shape:\n{saved}"
    );
    files.files.insert("deck.ron".into(), edited.into_bytes());
    let mut s = Session::new();
    ok(&mut s, files, "load deck.ron");
    s
}

#[test]
fn a_document_with_a_missing_asset_loads_and_its_render_names_it() {
    let mut files = MemoryFiles::default();
    let mut s = loaded_with_a_missing_asset(&mut files);
    assert_eq!(
        s.document().breaches(),
        vec![Breach::MissingAsset {
            area: NodeId(3),
            asset: AssetId(99)
        }]
    );
    assert_eq!(
        ok(&mut s, &mut files, "render s1 out.png 160 90"),
        "fallback=- missing=a99 glyphs=-"
    );
}

/// The inverse of an edit of an area loaded with a missing asset brings the asset back, and the
/// operation does not refuse it.
#[test]
fn an_undo_that_brings_back_a_missing_asset_is_not_refused() {
    let mut files = MemoryFiles::default();
    let mut s = loaded_with_a_missing_asset(&mut files);
    let loaded = s.document().clone();
    ok(&mut s, &mut files, "text n3 \"x\"");
    assert_eq!(s.document().breaches(), vec![]);
    ok(&mut s, &mut files, "undo");
    assert_eq!(s.document(), &loaded);
    ok(&mut s, &mut files, "redo");
    ok(&mut s, &mut files, "undo");
    assert_eq!(s.document(), &loaded);
}

/// `image` and the `font` option of `text` refuse an asset the table lacks, here one below the
/// counter, which no operation would refuse.
#[test]
fn no_command_names_a_missing_asset() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nasset font.ttf\nremove-asset a3",
    );
    for (line, asset) in [
        ("image n2 a3 fit", "a3"),
        ("text n2 \"x\" font=a3", "a3"),
        ("text n2 \"x\" font=a99", "a99"),
        ("image n2 a99 fit", "a99"),
    ] {
        let doc = s.document().clone();
        assert_eq!(
            s.execute(line, &mut files),
            Some(Reply::Err(format!("no asset {asset}"))),
            "`{line}`"
        );
        assert_eq!(s.document(), &doc, "`{line}` changed the document");
    }
}

/// A text whose font asset is missing is drawn in the built-in sans, and the render names its area.
#[test]
fn a_text_whose_font_is_missing_is_named_by_the_render() {
    let mut files = MemoryFiles::default();
    let mut s = loaded_with_a_missing_asset(&mut files);
    // The area holds an image of a99; the edit keeps the breach in the text's font instead.
    let text = String::from_utf8(files.files["deck.ron"].clone()).unwrap();
    let edited = text.replacen(
        "content: Image(\n                            asset: 99,\n                            fit: Fit,\n                        ),",
        "content: Text(text: \"x\", style: (font: Asset(99), size: 48.0, bold: false, italic: false, colour: (r: 0, g: 0, b: 0, a: 255), h_align: Left, v_align: Top)),",
        1,
    );
    assert_ne!(edited, text, "the saved file changed shape:\n{text}");
    files.files.insert("deck.ron".into(), edited.into_bytes());
    ok(&mut s, &mut files, "load deck.ron");
    assert_eq!(
        ok(&mut s, &mut files, "render s1 out.png 160 90"),
        "fallback=n3 missing=- glyphs=-"
    );
}
