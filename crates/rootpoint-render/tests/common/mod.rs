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

pub fn text(text: &str, style: TextStyle) -> Content {
    Content::Text(TextContent {
        text: text.to_owned(),
        style,
    })
}

/// One slide of nine text areas, three columns by three rows: each column one horizontal
/// alignment, left to right, each row one vertical alignment, top to bottom.
pub fn alignments() -> Document {
    let (mut doc, root) = one_slide();
    split(&mut doc, root, Axis::LeftToRight, 1.0 / 3.0);
    let second = doc.slides[0].root.areas()[1].id;
    split(&mut doc, second, Axis::LeftToRight, 0.5);
    let columns: Vec<NodeId> = doc.slides[0].root.areas().iter().map(|a| a.id).collect();
    for column in &columns {
        split(&mut doc, *column, Axis::TopToBottom, 1.0 / 3.0);
        let rest = match doc.node(*column) {
            Some(_) => {
                let rect_areas = doc.slides[0].root.areas();
                let i = rect_areas.iter().position(|a| a.id == *column).unwrap();
                rect_areas[i + 1].id
            }
            None => unreachable!(),
        };
        split(&mut doc, rest, Axis::TopToBottom, 0.5);
    }
    let areas: Vec<NodeId> = doc.slides[0].root.areas().iter().map(|a| a.id).collect();
    let h = [HAlign::Left, HAlign::Center, HAlign::Right];
    let v = [VAlign::Top, VAlign::Middle, VAlign::Bottom];
    for (i, area) in areas.into_iter().enumerate() {
        let style = TextStyle {
            size: 40.0,
            h_align: h[i / 3],
            v_align: v[i % 3],
            ..TextStyle::default()
        };
        set(
            &mut doc,
            area,
            text(&format!("{:?} {:?}", h[i / 3], v[i % 3]), style),
        );
    }
    doc
}

/// One slide of five areas mixing both axes: a title across the top, and below it an image beside
/// a column of a bold paragraph, an italic coloured line and an empty text.
pub fn mixed() -> (Document, AssetId) {
    let (mut doc, title) = one_slide();
    split(&mut doc, title, Axis::TopToBottom, 0.2);
    let body = doc.slides[0].root.areas()[1].id;
    split(&mut doc, body, Axis::LeftToRight, 0.4);
    let column = doc.slides[0].root.areas()[2].id;
    split(&mut doc, column, Axis::TopToBottom, 0.6);
    let last = doc.slides[0].root.areas()[3].id;
    split(&mut doc, last, Axis::TopToBottom, 0.5);
    let areas: Vec<NodeId> = doc.slides[0].root.areas().iter().map(|a| a.id).collect();
    assert_eq!(areas.len(), 5);
    let asset = add_asset(&mut doc, "two-colours.png");
    set(
        &mut doc,
        areas[0],
        text(
            "A title in the built-in sans",
            TextStyle {
                size: 72.0,
                bold: true,
                h_align: HAlign::Center,
                v_align: VAlign::Middle,
                ..TextStyle::default()
            },
        ),
    );
    set(
        &mut doc,
        areas[1],
        Content::Image(ImageContent {
            asset,
            fit: ImageFit::Fit,
        }),
    );
    set(
        &mut doc,
        areas[2],
        text(
            "A paragraph long enough to wrap over several lines of its area, so that the \
             line breaks of the layout are visible in the image.",
            TextStyle {
                bold: true,
                ..TextStyle::default()
            },
        ),
    );
    set(
        &mut doc,
        areas[3],
        text(
            "An italic line in blue",
            TextStyle {
                italic: true,
                colour: Rgba {
                    r: 30,
                    g: 60,
                    b: 200,
                    a: 255,
                },
                ..TextStyle::default()
            },
        ),
    );
    (doc, asset)
}
