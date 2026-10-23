//! The save file: a document as RON text, with the version of its format.

use std::collections::BTreeMap;

use ron::extensions::Extensions;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::document::{AspectRatio, Asset, AssetId, Document, Slide, TextStyle};

/// The version of the file format this crate writes and reads. A file of another version is
/// refused, per `design@rootpoint-model@save-format`.
pub const FORMAT_VERSION: u32 = 3;

/// Why a file was not loaded.
#[derive(Clone, Debug, PartialEq)]
pub enum LoadError {
    /// The text is not a document of this format; the parser's message says where.
    Parse(String),
    /// The file is of another version of the format.
    Version { found: u32, expected: u32 },
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
    slides: &'a [Slide],
}

/// The file as read; its version was checked by [`VersionOnly`] first.
#[derive(Deserialize)]
struct File {
    aspect: AspectRatio,
    next_id: u64,
    assets: BTreeMap<AssetId, Asset>,
    styles: BTreeMap<String, TextStyle>,
    slides: Vec<Slide>,
}

/// Only the version of a file, read before the rest so that a file of another version is refused
/// with its version rather than with a parse error.
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
        slides: &doc.slides,
    };
    // `unwrap_variant_newtypes` writes `Split(id: 3, ...)` rather than `Split((id: 3, ...))`; the
    // file declares the extension in its first line, so a reader needs no option to read it back.
    let config = PrettyConfig::new().extensions(Extensions::UNWRAP_VARIANT_NEWTYPES);
    ron::ser::to_string_pretty(&file, config).expect("a document always serializes")
}

/// The document a RON text holds.
pub fn load_ron(text: &str) -> Result<Document, LoadError> {
    let version: VersionOnly = ron::from_str(text).map_err(|e| LoadError::Parse(e.to_string()))?;
    if version.version != FORMAT_VERSION {
        return Err(LoadError::Version {
            found: version.version,
            expected: FORMAT_VERSION,
        });
    }
    let file: File = ron::from_str(text).map_err(|e| LoadError::Parse(e.to_string()))?;
    let doc = Document {
        aspect: file.aspect,
        slides: file.slides,
        assets: file.assets,
        styles: file.styles,
        next_id: file.next_id,
    };
    doc.validate()
        .map_err(|crate::validate::Invalid(reason)| LoadError::Invalid(reason))?;
    Ok(doc)
}
