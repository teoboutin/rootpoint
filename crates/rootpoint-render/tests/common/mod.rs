//! Helpers shared by the render tests: the fixtures and the golden-image comparison.

#![allow(dead_code)]

use std::path::PathBuf;

use rootpoint_model::*;
use vello_cpu::Pixmap;

/// A 40×20 PNG: its left half red, its right half blue, encoded in memory.
pub fn two_colour_png() -> Vec<u8> {
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

pub fn one_slide() -> (Document, NodeId) {
    let mut doc = Document::default();
    apply(&mut doc, Op::AddSlide { at: 0 }).unwrap();
    let root = doc.slides[0].root.id();
    (doc, root)
}

pub fn split(doc: &mut Document, node: NodeId, axis: Axis, at: f64) {
    apply(
        doc,
        Op::Split {
            node,
            axis,
            at,
            side: Side::After,
        },
    )
    .unwrap();
}

pub fn set(doc: &mut Document, area: NodeId, content: Content) {
    apply(doc, Op::SetContent { area, content }).unwrap();
}

pub fn add_asset(doc: &mut Document, path: &str) -> AssetId {
    match apply(
        doc,
        Op::AddAsset {
            asset: Asset {
                source: AssetSource::Path(path.to_owned()),
            },
        },
    )
    .unwrap()
    {
        Op::RemoveAsset { id } => id,
        other => panic!("AddAsset returned {other:?}"),
    }
}

/// Compare the pixels of `pixmap` with the committed golden PNG `name`, pixel by pixel.
/// `ROOTPOINT_BLESS=1` rewrites the golden file after a deliberate change.
pub fn assert_golden(pixmap: Pixmap, name: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name);
    let (w, h) = (pixmap.width(), pixmap.height());
    let actual = pixmap.data_as_u8_slice().to_vec();
    if std::env::var_os("ROOTPOINT_BLESS").is_some() {
        std::fs::write(&path, pixmap.into_png().unwrap()).unwrap();
    }
    let bytes =
        std::fs::read(&path).unwrap_or_else(|_| panic!("the golden file {}", path.display()));
    let golden = Pixmap::from_png(std::io::Cursor::new(bytes)).unwrap();
    assert_eq!((golden.width(), golden.height()), (w, h), "{name}: size");
    let differ = actual
        .chunks(4)
        .zip(golden.data_as_u8_slice().chunks(4))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        differ, 0,
        "{name}: {differ} pixels differ from the golden image"
    );
}
