//! The save file: a document as RON text, with the version of its format.

use std::collections::BTreeMap;

use ron::extensions::Extensions;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::document::{AspectRatio, Asset, AssetId, Document, Slide, SpanStyle, TextStyle};

/// The version of the file format this crate writes. A file of a version from [`OLDEST_VERSION`]
/// up to it loads, per `design@rootpoint@saved-presentations-stay-loadable`, and a file of any
/// other version is refused, per `design@rootpoint-model@save-format`.
pub const FORMAT_VERSION: u32 = 7;

/// The oldest version of the file format that loads. An older file is refused.
pub const OLDEST_VERSION: u32 = 7;

/// Why a file was not loaded.
#[derive(Clone, Debug, PartialEq)]
pub enum LoadError {
    /// The text is not a document of this format; the parser's message says where.
    Parse(String),
    /// The file is of a version of the format outside `oldest..=newest`, or of a version inside it
    /// that no migration reads yet.
    Version {
        found: u32,
        oldest: u32,
        newest: u32,
    },
    /// The file parses, but its document breaks an invariant, which the reason names: a file
    /// edited by hand can, and loading it would let a later operation meet a broken tree.
    Invalid(String),
}

/// The file as written: the version first, so a reader of the text sees it before the rest.
#[derive(Serialize)]
struct FileRef<'a> {
    version: u32,
    aspect: &'a AspectRatio,
    next_id: u64,
    assets: &'a BTreeMap<AssetId, Asset>,
    styles: &'a BTreeMap<String, TextStyle>,
    span_styles: &'a BTreeMap<String, SpanStyle>,
    slides: &'a [Slide],
}

/// The file as read; its version was checked by [`VersionOnly`] first.
#[derive(Deserialize)]
struct File {
    aspect: AspectRatio,
    next_id: u64,
    assets: BTreeMap<AssetId, Asset>,
    styles: BTreeMap<String, TextStyle>,
    span_styles: BTreeMap<String, SpanStyle>,
    slides: Vec<Slide>,
}

/// Only the version of a file, read before the rest so that a file of a version that does not load
/// is refused with its version rather than with a parse error, and one that loads is migrated first.
#[derive(Deserialize)]
struct VersionOnly {
    version: u32,
}

/// The document as RON text.
pub fn save_ron(doc: &Document) -> String {
    let file = FileRef {
        version: FORMAT_VERSION,
        aspect: &doc.aspect,
        next_id: doc.next_id,
        assets: &doc.assets,
        styles: &doc.styles,
        span_styles: &doc.span_styles,
        slides: &doc.slides,
    };
    // `unwrap_variant_newtypes` writes `Split(id: 3, ...)` rather than `Split((id: 3, ...))`, and
    // `implicit_some` a span's set value as `bold: true` rather than `bold: Some(true)`; the file
    // declares the extensions in its first lines, so a reader needs no option to read it back.
    // `escape_strings(false)` writes a string holding a `"` or a `\` as a raw string, so a formula
    // reads `r#"\frac{a}{b}"#` rather than with every backslash doubled; it applies to every
    // string of the file, as ron 0.12.2's `serialize_unescaped_or_raw_str` writes it.
    let config = PrettyConfig::new()
        .extensions(Extensions::UNWRAP_VARIANT_NEWTYPES | Extensions::IMPLICIT_SOME)
        .escape_strings(false);
    ron::ser::to_string_pretty(&file, config).expect("a document always serializes")
}

/// The document a RON text holds.
pub fn load_ron(text: &str) -> Result<Document, LoadError> {
    let version: VersionOnly = ron::from_str(text).map_err(|e| LoadError::Parse(e.to_string()))?;
    match version.version {
        FORMAT_VERSION => {}
        // A file of a version from `OLDEST_VERSION` on is migrated here to `FORMAT_VERSION`, one step
        // a version, before it is read, per `design@rootpoint@saved-presentations-stay-loadable`.
        // No step exists while the two versions are equal; how one is written is
        // `issue@rootpoint-model@migration-shape`. Without its step, a file of an older version is
        // refused, and the test of every frozen version in `path@rootpoint-model@tests/formats.rs`
        // fails.
        found => {
            return Err(LoadError::Version {
                found,
                oldest: OLDEST_VERSION,
                newest: FORMAT_VERSION,
            });
        }
    }
    let file: File = ron::from_str(text).map_err(|e| LoadError::Parse(e.to_string()))?;
    let mut doc = Document {
        aspect: file.aspect,
        slides: file.slides,
        assets: file.assets,
        styles: file.styles,
        span_styles: file.span_styles,
        next_id: file.next_id,
    };
    // A file edited by hand may split a run of characters in two, or leave one empty; the text
    // reads the same, so it loads in the canonical form operations keep.
    crate::ops::normalise_texts(&mut doc);
    doc.validate()
        .map_err(|crate::validate::Invalid(reason)| LoadError::Invalid(reason))?;
    Ok(doc)
}
