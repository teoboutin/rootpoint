//! A pack: a presentation and its assets in one zip file, `design@rootpoint-protocol@pack-container`:
//! first the entry `mimetype`, stored, holding `PACK_MIMETYPE`; then `presentation.ron`, deflated;
//! then each asset's file at its asset path, stored, its bytes as they are.

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};

use rootpoint_model::asset_path_is_confined;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

use crate::store::PRESENTATION;

/// The media type a pack's first entry holds.
pub(crate) const PACK_MIMETYPE: &str = "application/x-rootpoint";

/// The name of a pack's first entry.
const MIMETYPE: &str = "mimetype";

/// A pack of the RON text and the assets' files, by asset path, in the order given.
pub(crate) fn write_pack(ron: &str, assets: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let fail = |e: zip::result::ZipError| format!("the pack: {e}");
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    // A fixed time, so that one presentation always packs to the same bytes.
    let stored = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(DateTime::default());
    let deflated = stored.compression_method(CompressionMethod::Deflated);
    zip.start_file(MIMETYPE, stored).map_err(fail)?;
    zip.write_all(PACK_MIMETYPE.as_bytes())
        .map_err(|e| e.to_string())?;
    zip.start_file(PRESENTATION, deflated).map_err(fail)?;
    zip.write_all(ron.as_bytes()).map_err(|e| e.to_string())?;
    // Two assets a file edited by hand names at one path are one file of the pack.
    let mut written = std::collections::BTreeSet::new();
    for (path, bytes) in assets {
        if !written.insert(path) {
            continue;
        }
        zip.start_file(path.as_str(), stored).map_err(fail)?;
        zip.write_all(bytes).map_err(|e| e.to_string())?;
    }
    Ok(zip.finish().map_err(fail)?.into_inner())
}

/// The RON text and the files of a pack, by asset path. Refused whole: a file whose first entry is
/// not `mimetype` holding `PACK_MIMETYPE`, a pack with no `presentation.ron`, an entry that is a
/// symlink or whose name, a directory's included, fails `asset_path_is_confined`, and a file whose
/// name is the folder of another, which no directory can hold. A directory entry, which a zip tool
/// writes for each folder, holds no file and is skipped once its name is checked.
pub(crate) fn read_pack(bytes: &[u8]) -> Result<(String, BTreeMap<String, Vec<u8>>), String> {
    let fail = |e: zip::result::ZipError| format!("the pack: {e}");
    let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(fail)?;
    let mut ron = None;
    let mut files = BTreeMap::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(fail)?;
        let name = entry.name().map_err(fail)?.into_owned();
        if i == 0 {
            let mut held = Vec::new();
            entry.read_to_end(&mut held).map_err(|e| e.to_string())?;
            if name != MIMETYPE || held != PACK_MIMETYPE.as_bytes() {
                return Err(format!(
                    "not a pack: its first entry is not {MIMETYPE} holding {PACK_MIMETYPE}"
                ));
            }
            continue;
        }
        if entry.is_symlink() {
            return Err(format!("the pack's entry {name} is a symlink"));
        }
        if entry.is_dir() {
            if !asset_path_is_confined(name.trim_end_matches('/')) {
                return Err(format!(
                    "the pack's entry {name} is not a path inside the root"
                ));
            }
            continue;
        }
        let mut held = Vec::new();
        entry.read_to_end(&mut held).map_err(|e| e.to_string())?;
        if name == PRESENTATION {
            ron = Some(
                String::from_utf8(held)
                    .map_err(|_| format!("the pack's {PRESENTATION} is not UTF-8"))?,
            );
        } else if asset_path_is_confined(&name) {
            files.insert(name, held);
        } else {
            return Err(format!(
                "the pack's entry {name} is not a path inside the root"
            ));
        }
    }
    let ron = ron.ok_or(format!("the pack holds no {PRESENTATION}"))?;
    // A file of the pack, its two own included, cannot be the folder of another: each folder of
    // each path is looked up among the files, in time linear in the entries and their depth.
    for path in files.keys() {
        for (at, _) in path.match_indices('/') {
            let folder = &path[..at];
            if files.contains_key(folder) || folder == MIMETYPE || folder == PRESENTATION {
                return Err(format!("the pack's entry {folder} is a file and a folder"));
            }
        }
    }
    Ok((ron, files))
}
