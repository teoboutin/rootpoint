//! Saving then loading a document gives it back, and a file of another version is refused.

use std::path::PathBuf;

use proptest::prelude::*;
use rootpoint_model::*;

fn data(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(name)
}

/// One slide: a vertical split of an image area on the left and, on the right, a horizontal split
/// of two text areas. The image is a file beside the presentation.
fn fixture() -> Document {
    let mut doc = Document::default();
    apply(&mut doc, Op::AddSlide { at: 0 }).unwrap();
    let left = doc.slides[0].root.id();
    let split = |doc: &mut Document, node, axis| {
        apply(
            doc,
            Op::Split {
                node,
                axis,
                at: 0.5,
                side: Side::After,
            },
        )
        .unwrap();
    };
    split(&mut doc, left, Axis::Vertical);
    let right = doc.slides[0].root.areas()[1].id;
    split(&mut doc, right, Axis::Horizontal);
    let Op::RemoveAsset { id: asset } = apply(
        &mut doc,
        Op::AddAsset {
            asset: Asset {
                source: AssetSource::Path("images/cat.png".to_owned()),
            },
        },
    )
    .unwrap() else {
        panic!("AddAsset returned another inverse");
    };
    let areas: Vec<NodeId> = doc.slides[0].root.areas().iter().map(|a| a.id).collect();
    let set = |doc: &mut Document, area, content| {
        apply(doc, Op::SetContent { area, content }).unwrap();
    };
    set(
        &mut doc,
        areas[0],
        Content::Image(ImageContent {
            asset,
            fit: ImageFit::Fit,
        }),
    );
    set(
        &mut doc,
        areas[1],
        Content::Text(TextContent {
            text: "A title".to_owned(),
            style: TextStyle {
                size: 72.0,
                bold: true,
                h_align: HAlign::Center,
                ..TextStyle::default()
            },
        }),
    );
    set(
        &mut doc,
        areas[2],
        Content::Text(TextContent {
            text: "Some wrapped text below the title.".to_owned(),
            style: TextStyle::default(),
        }),
    );
    doc
}

#[test]
fn the_fixture_round_trips_and_matches_its_committed_file() {
    let doc = fixture();
    let text = save_ron(&doc);
    assert_eq!(load_ron(&text), Ok(doc));
    let path = data("fixture.ron");
    let committed = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        // The first run writes the file, which is then committed.
        std::fs::write(&path, &text).unwrap();
        text.clone()
    });
    assert_eq!(
        committed,
        text,
        "the format changed; update {}",
        path.display()
    );
}

#[test]
fn the_file_states_its_version_first() {
    let text = save_ron(&fixture());
    let body = text.lines().find(|l| !l.starts_with("#![")).unwrap();
    assert_eq!(body.trim(), "(");
    let first_field = text
        .lines()
        .find(|l| l.trim().starts_with("version"))
        .unwrap();
    assert_eq!(first_field.trim(), format!("version: {FORMAT_VERSION},"));
}

#[test]
fn a_file_of_another_version_is_refused() {
    let text =
        save_ron(&fixture()).replacen(&format!("version: {FORMAT_VERSION},"), "version: 99,", 1);
    assert_eq!(
        load_ron(&text),
        Err(LoadError::Version {
            found: 99,
            expected: FORMAT_VERSION
        })
    );
}

#[test]
fn text_that_is_not_a_document_is_a_parse_error() {
    assert!(matches!(
        load_ron("(version: 1, slides: oops)"),
        Err(LoadError::Parse(_))
    ));
}

fn content() -> impl Strategy<Value = Content> {
    prop_oneof![
        Just(Content::Empty),
        (
            ".{0,20}",
            1.0f64..200.0,
            any::<bool>(),
            any::<bool>(),
            any::<[u8; 4]>(),
            0..3u8,
            0..3u8
        )
            .prop_map(|(text, size, bold, italic, [r, g, b, a], h, v)| {
                Content::Text(TextContent {
                    text,
                    style: TextStyle {
                        font: FontRef::Builtin("sans".to_owned()),
                        size,
                        bold,
                        italic,
                        colour: Rgba { r, g, b, a },
                        h_align: [HAlign::Left, HAlign::Center, HAlign::Right][h as usize],
                        v_align: [VAlign::Top, VAlign::Middle, VAlign::Bottom][v as usize],
                    },
                })
            }),
    ]
}

fn source() -> impl Strategy<Value = AssetSource> {
    prop_oneof![
        "[a-z/]{1,12}\\.png".prop_map(AssetSource::Path),
        prop::collection::vec(any::<u8>(), 0..16).prop_map(AssetSource::Embedded),
    ]
}

proptest! {
    /// Over random documents, of several slides, random trees, every kind of content and both
    /// kinds of asset source, loading the saved text gives the document back.
    #[test]
    fn saving_then_loading_gives_the_document_back(
        steps in prop::collection::vec((0u8..4, any::<usize>(), 0.05f64..0.95, any::<bool>(), content(), source()), 1..30)
    ) {
        let mut doc = Document::default();
        apply(&mut doc, Op::AddSlide { at: 0 }).unwrap();
        for (kind, pick, at, flag, content, source) in steps {
            let slide = &doc.slides[pick % doc.slides.len()];
            let areas: Vec<NodeId> = slide.root.areas().iter().map(|a| a.id).collect();
            let area = areas[pick % areas.len()];
            let op = match kind {
                0 => Op::AddSlide { at: 0 },
                1 => Op::Split {
                    node: area,
                    axis: if flag { Axis::Vertical } else { Axis::Horizontal },
                    at,
                    side: Side::After,
                },
                2 => Op::SetContent { area, content },
                _ => Op::AddAsset { asset: Asset { source } },
            };
            apply(&mut doc, op).unwrap();
        }
        let text = save_ron(&doc);
        prop_assert_eq!(load_ron(&text), Ok(doc));
    }
}
