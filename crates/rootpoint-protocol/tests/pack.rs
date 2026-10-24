//! A pack: a zip whose first entry names its type, then the presentation and its assets' files. It
//! round-trips, keeps a tree of assets through an unpack elsewhere, unzips with a common tool into
//! a workspace, adds only the zip's headers to its contents, and opens only when every entry is a
//! confined path; `unpack` turns a presentation held in memory into a workspace.

mod common;

/// Write a presentation's text as `presentation.ron` of the directory `dir`.
fn write_presentation_text(files: &mut common::MemoryFiles, dir: &str, text: &str) {
    files
        .files
        .insert(format!("{dir}/presentation.ron"), text.as_bytes().to_vec());
}

use std::io::Write;
use std::path::Path;

use common::*;
use rootpoint_model::*;
use rootpoint_protocol::{Reply, Session, Store};

/// The media type a pack's first entry holds.
const PACK_MIMETYPE: &str = "application/x-rootpoint";

fn run_ok(s: &mut Session, files: &mut RecordingFiles, line: &str) -> String {
    match s.execute(line, files) {
        Some(Reply::Ok(text)) => text,
        other => panic!("`{line}` gave {other:?}"),
    }
}

fn map(s: &Session) -> std::collections::BTreeMap<String, Vec<u8>> {
    match s.store() {
        Store::Memory(map) => map.clone(),
        Store::Workspace(root) => panic!("a workspace at {root}"),
    }
}

#[test]
fn a_pack_round_trips() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    ok(&mut s, &mut files, "sample");
    assert_eq!(ok(&mut s, &mut files, "pack s.rpk"), "missing=-");
    let mut t = Session::new();
    ok(&mut t, &mut files, "open-pack s.rpk");
    assert_eq!(t.document(), s.document());
    assert_eq!(map(&t), map(&s));
    assert!(!map(&t).is_empty());
}

/// A workspace whose assets are in two levels of folders: the sample's image twice, and the
/// built-in font, each shown by an area or named by a style.
fn tree_workspace(root: &Path) -> Session {
    let png = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/sample/splits.png")).unwrap();
    let font = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../rootpoint-render/fonts/NotoSans-Regular.ttf"
    ))
    .unwrap();
    std::fs::create_dir_all(root.join("img/b")).unwrap();
    std::fs::create_dir_all(root.join("fonts")).unwrap();
    std::fs::write(root.join("img/a.png"), &png).unwrap();
    std::fs::write(root.join("img/b/c.png"), [png.as_slice(), b"x"].concat()).unwrap();
    std::fs::write(root.join("fonts/f.ttf"), &font).unwrap();
    let r = root.to_str().unwrap();
    let mut files = RecordingFiles::default();
    let mut s = Session::new();
    for line in [
        format!("open {r}"),
        "add-slide".to_owned(),
        "split n2 beside 0.5".to_owned(),
        format!("import {r}/img/a.png"),
        format!("import {r}/img/b/c.png"),
        format!("import {r}/fonts/f.ttf"),
        "image n2 a5 fit".to_owned(),
        "image n3 a6 fill".to_owned(),
        "style set \"Body\" font=a7".to_owned(),
        "save".to_owned(),
    ] {
        run_ok(&mut s, &mut files, &line);
    }
    s
}

/// The files below a directory, `.rootpoint/` left out, by their paths under it.
fn tree(dir: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    fn walk(dir: &Path, below: &str, out: &mut std::collections::BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name().into_string().unwrap();
            let path = if below.is_empty() {
                name
            } else {
                format!("{below}/{name}")
            };
            if entry.file_type().unwrap().is_dir() {
                walk(&entry.path(), &path, out);
            } else {
                out.insert(path, std::fs::read(entry.path()).unwrap());
            }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    walk(dir, "", &mut out);
    out
}

/// The closed issue `pack-and-unpack` asked that packing, then unpacking elsewhere, keeps a tree of
/// assets: the same document, the same paths, the same bytes.
#[test]
fn a_pack_then_an_unpack_elsewhere_keeps_the_tree_of_assets() {
    let dir = std::fs::canonicalize(scratch("pack-tree")).unwrap();
    let root = dir.join("ws");
    let s = tree_workspace(&root);
    let paths: Vec<&str> = s
        .document()
        .assets
        .values()
        .map(|a| a.path.as_str())
        .collect();
    assert_eq!(paths, ["img/a.png", "img/b/c.png", "fonts/f.ttf"]);
    let mut files = RecordingFiles::default();
    let mut s = s;
    let pack = dir.join("ws.rpk");
    let pack = pack.to_str().unwrap();
    assert_eq!(
        run_ok(&mut s, &mut files, &format!("pack {pack}")),
        "missing=-"
    );
    let out = dir.join("elsewhere");
    let mut t = Session::new();
    run_ok(&mut t, &mut files, &format!("open-pack {pack}"));
    run_ok(
        &mut t,
        &mut files,
        &format!("unpack {}", out.to_str().unwrap()),
    );
    let mut u = Session::new();
    run_ok(
        &mut u,
        &mut files,
        &format!("open {}", out.to_str().unwrap()),
    );
    assert_eq!(u.document(), s.document());
    assert_eq!(tree(&out), tree(&root));
}

/// A pack extracted by `unzip` is a workspace that opens to the same presentation,
/// `design@rootpoint-protocol@pack-container`. Skipped, saying so, where `unzip` is not installed.
#[test]
fn a_pack_unzips_with_a_common_tool_into_a_workspace() {
    let Ok(status) = std::process::Command::new("unzip").arg("-v").output() else {
        eprintln!("skipped: no `unzip` on this machine");
        return;
    };
    assert!(status.status.success());
    let dir = std::fs::canonicalize(scratch("pack-unzip")).unwrap();
    let root = dir.join("ws");
    let mut s = tree_workspace(&root);
    let mut files = RecordingFiles::default();
    let pack = dir.join("ws.rpk");
    run_ok(
        &mut s,
        &mut files,
        &format!("pack {}", pack.to_str().unwrap()),
    );
    let out = dir.join("unzipped");
    let unzip = std::process::Command::new("unzip")
        .arg("-q")
        .arg(&pack)
        .arg("-d")
        .arg(&out)
        .status()
        .unwrap();
    assert!(unzip.success());
    std::fs::remove_file(out.join("mimetype")).unwrap();
    let mut t = Session::new();
    run_ok(
        &mut t,
        &mut files,
        &format!("open {}", out.to_str().unwrap()),
    );
    assert_eq!(t.document(), s.document());
    assert_eq!(tree(&out), tree(&root));
}

/// The first entry is `mimetype`, stored, with no extra field, so its name and the media type are
/// at fixed offsets of the file's first bytes, as EPUB's container places its own.
#[test]
fn a_pack_s_first_entry_names_its_type_at_a_fixed_offset() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(&mut s, &mut files, "sample\npack s.rpk");
    let bytes = &files.files["s.rpk"];
    assert_eq!(&bytes[0..4], b"PK\x03\x04", "a local file header");
    assert_eq!(&bytes[8..10], &[0, 0], "stored, with no compression");
    assert_eq!(&bytes[26..28], &[8, 0], "a name of 8 bytes");
    assert_eq!(&bytes[28..30], &[0, 0], "no extra field");
    assert_eq!(&bytes[30..38], b"mimetype");
    assert_eq!(
        &bytes[38..38 + PACK_MIMETYPE.len()],
        PACK_MIMETYPE.as_bytes()
    );
}

/// The bound of `design@rootpoint-protocol@pack-container`: the pack's size, less its assets'
/// bytes, the deflated `presentation.ron` and the media type, is at most 22 bytes plus, for each
/// entry, 100 bytes and twice its name's length. The figures print with `--nocapture`.
#[test]
fn a_pack_adds_only_the_zip_s_headers() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    ok(&mut s, &mut files, "sample");
    let dir = std::fs::canonicalize(scratch("pack-overhead")).unwrap();
    let tree = tree_workspace(&dir.join("ws"));
    for (name, session) in [("the sample", &mut s), ("the tree", &mut { tree })] {
        let mut files = MemoryFiles::default();
        if let Store::Workspace(root) = session.store().clone() {
            for (path, bytes) in self::tree(Path::new(&root)) {
                files.files.insert(format!("{root}/{path}"), bytes);
            }
        }
        ok(session, &mut files, "pack p.rpk");
        let pack = files.files["p.rpk"].clone();
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&pack)).unwrap();
        let (mut assets, mut ron, mut bound) = (0, 0, 22);
        for i in 0..zip.len() {
            let entry = zip.by_index(i).unwrap();
            let entry_name = entry.name().unwrap().into_owned();
            bound += 100 + 2 * entry_name.len() as u64;
            match entry_name.as_str() {
                "mimetype" => {}
                "presentation.ron" => ron = entry.compressed_size(),
                _ => assets += entry.size(),
            }
        }
        let overhead = pack.len() as u64 - assets - ron - PACK_MIMETYPE.len() as u64;
        eprintln!(
            "{name}: pack {} bytes, assets {assets}, deflated RON {ron}, overhead {overhead}, bound {bound}",
            pack.len()
        );
        assert!(overhead <= bound, "{name}: {overhead} above {bound}");
    }
}

/// A zip of these entries, in order, each a name and its bytes; a name ending in `@` is written
/// as a symlink to `../x`.
fn crafted(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let stored =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, bytes) in entries {
        match name.strip_suffix('@') {
            Some(link) => zip.add_symlink(link, "../x", stored).unwrap(),
            None => {
                zip.start_file(*name, stored).unwrap();
                zip.write_all(bytes).unwrap();
            }
        }
    }
    zip.finish().unwrap().into_inner()
}

/// Open a crafted pack after `sample`, and check it is refused and the store unchanged.
fn refused(entries: &[(&str, &[u8])]) {
    let ron = {
        let mut files = MemoryFiles::default();
        let mut s = Session::new();
        ok(&mut s, &mut files, "sample");
        save_ron(s.document())
    };
    let mut all: Vec<(&str, &[u8])> = vec![
        ("mimetype", PACK_MIMETYPE.as_bytes()),
        ("presentation.ron", ron.as_bytes()),
    ];
    all.extend_from_slice(entries);
    let mut files = MemoryFiles::default();
    files.files.insert("bad.rpk".into(), crafted(&all));
    let mut s = Session::new();
    ok(&mut s, &mut files, "sample");
    let (doc, store) = (s.document().clone(), s.store().clone());
    let reply = s.execute("open-pack bad.rpk", &mut files);
    assert!(
        matches!(reply, Some(Reply::Err(_))),
        "{entries:?} gave {reply:?}"
    );
    assert_eq!(s.document(), &doc);
    assert_eq!(s.store(), &store);
}

#[test]
fn a_pack_with_an_entry_above_its_root_is_refused_whole() {
    refused(&[("splits.png", b"ok"), ("../x", b"x")]);
}

#[test]
fn a_pack_with_an_absolute_entry_is_refused_whole() {
    refused(&[("/x", b"x")]);
}

#[test]
fn a_pack_with_an_entry_in_rootpoint_is_refused_whole() {
    refused(&[(".rootpoint/lock", b"x")]);
}

#[test]
fn a_pack_with_a_symlink_entry_is_refused_whole() {
    refused(&[("img/link.png@", b"")]);
}

#[test]
fn a_pack_with_a_directory_entry_above_its_root_is_refused_whole() {
    refused(&[("../evil/", b"")]);
}

#[test]
fn a_pack_whose_file_is_also_a_folder_is_refused_whole() {
    refused(&[("a", b"a"), ("a/b", b"b")]);
    // The pack's own presentation is a file too.
    refused(&[("presentation.ron/x", b"x")]);
}

/// A zip whose first entry is not `mimetype` holding the media type is no pack: one whose first
/// entry holds another type, and one whose `mimetype` comes second.
#[test]
fn a_zip_whose_first_entry_is_not_mimetype_is_refused() {
    let mut files = MemoryFiles::default();
    files.files.insert(
        "epub.rpk".into(),
        crafted(&[
            ("mimetype", b"application/epub+zip"),
            ("presentation.ron", b"x"),
        ]),
    );
    let mut s = Session::new();
    assert!(matches!(
        s.execute("open-pack epub.rpk", &mut files),
        Some(Reply::Err(e)) if e.contains("not a pack")
    ));
    let ron = {
        let mut files = MemoryFiles::default();
        let mut s = Session::new();
        ok(&mut s, &mut files, "sample");
        save_ron(s.document())
    };
    let mut files = MemoryFiles::default();
    files.files.insert(
        "late.rpk".into(),
        crafted(&[
            ("presentation.ron", ron.as_bytes()),
            ("mimetype", PACK_MIMETYPE.as_bytes()),
        ]),
    );
    let mut s = Session::new();
    assert!(matches!(
        s.execute("open-pack late.rpk", &mut files),
        Some(Reply::Err(_))
    ));
}

/// Two assets a file edited by hand names at one path are packed as one file.
#[test]
fn two_assets_at_one_path_pack_as_one_file() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    ok(&mut s, &mut files, "sample");
    let mut doc = s.document().clone();
    let path = doc.assets.values().next().unwrap().path.clone();
    apply(
        &mut doc,
        Op::AddAsset {
            asset: Asset { path },
        },
    )
    .unwrap();
    write_presentation_text(&mut files, "ws", &save_ron(&doc));
    files.files.insert("ws/splits.png".into(), b"png".to_vec());
    ok(&mut s, &mut files, "open ws");
    assert_eq!(ok(&mut s, &mut files, "pack d.rpk"), "missing=-");
    let mut t = Session::new();
    ok(&mut t, &mut files, "open-pack d.rpk");
    assert_eq!(t.document(), s.document());
}

#[test]
fn a_zip_that_is_no_pack_is_refused() {
    let mut files = MemoryFiles::default();
    files
        .files
        .insert("a.zip".into(), crafted(&[("presentation.ron", b"x")]));
    files.files.insert(
        "no-ron.rpk".into(),
        crafted(&[("mimetype", PACK_MIMETYPE.as_bytes())]),
    );
    let mut s = Session::new();
    for line in ["open-pack a.zip", "open-pack no-ron.rpk"] {
        assert!(
            matches!(s.execute(line, &mut files), Some(Reply::Err(_))),
            "{line}"
        );
    }
}

/// At `pack`, an asset whose path fails confinement gets no bytes in the pack, the reply names it,
/// and no path outside the root reaches the host's files,
/// `design@rootpoint-protocol@store-confines-access`.
#[test]
fn a_pack_writes_no_bytes_of_an_unconfined_asset() {
    let dir = std::fs::canonicalize(scratch("pack-unconfined")).unwrap();
    let root = dir.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(dir.join("outside.png"), b"outside").unwrap();
    std::fs::write(root.join("in.png"), b"inside").unwrap();
    let mut doc = Document::default();
    for path in ["in.png", "../outside.png"] {
        apply(
            &mut doc,
            Op::AddAsset {
                asset: Asset {
                    path: path.to_owned(),
                },
            },
        )
        .unwrap();
    }
    std::fs::write(root.join("presentation.ron"), save_ron(&doc)).unwrap();
    let r = root.to_str().unwrap();
    let mut files = RecordingFiles::default();
    let mut s = Session::new();
    run_ok(&mut s, &mut files, &format!("open {r}"));
    assert_eq!(
        run_ok(&mut s, &mut files, &format!("pack {r}/p.rpk")),
        "missing=a2"
    );
    assert_eq!(files.outside(&root), Vec::<String>::new());
    let mut t = Session::new();
    let mut memory = MemoryFiles::default();
    memory
        .files
        .insert("p.rpk".into(), std::fs::read(root.join("p.rpk")).unwrap());
    ok(&mut t, &mut memory, "open-pack p.rpk");
    assert_eq!(map(&t).keys().collect::<Vec<_>>(), ["in.png"]);
}

/// `unpack` makes the store a workspace, refuses a directory that holds a file and a store that is
/// a workspace, and a `save` then writes `presentation.ron` there.
#[test]
fn unpack_makes_the_store_a_workspace() {
    let mut files = MemoryFiles::default();
    files.files.insert("full/a.txt".into(), b"a".to_vec());
    let mut s = Session::new();
    ok(&mut s, &mut files, "sample");
    assert!(matches!(
        s.execute("unpack full", &mut files),
        Some(Reply::Err(_))
    ));
    assert!(matches!(s.store(), Store::Memory(_)));
    // A path that is a file is no directory to unpack into.
    files.files.insert("a-file".into(), b"x".to_vec());
    assert!(matches!(
        s.execute("unpack a-file", &mut files),
        Some(Reply::Err(_))
    ));
    ok(&mut s, &mut files, "unpack out");
    assert_eq!(ok(&mut s, &mut files, "store"), "workspace out unsaved=0");
    assert!(files.files.contains_key("out/splits.png"));
    files.files.remove("out/presentation.ron");
    ok(&mut s, &mut files, "add-slide");
    ok(&mut s, &mut files, "save");
    let text = String::from_utf8(files.files["out/presentation.ron"].clone()).unwrap();
    assert_eq!(&load_ron(&text).unwrap(), s.document());
    assert!(matches!(
        s.execute("unpack other", &mut files),
        Some(Reply::Err(_))
    ));
}

/// In memory, `save` writes the pack last opened or written, `store` names it, and a pack resets
/// the steps since the save; with no pack, `save` replies `err`.
#[test]
fn a_save_in_memory_writes_its_pack_again() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    ok(&mut s, &mut files, "sample");
    assert!(matches!(s.execute("save", &mut files), Some(Reply::Err(_))));
    ok(&mut s, &mut files, "pack s.rpk");
    assert_eq!(
        ok(&mut s, &mut files, "store"),
        "memory unsaved=0 pack=s.rpk"
    );
    ok(&mut s, &mut files, "add-slide");
    assert_eq!(
        ok(&mut s, &mut files, "store"),
        "memory unsaved=1 pack=s.rpk"
    );
    assert_eq!(ok(&mut s, &mut files, "save"), "missing=-");
    let mut t = Session::new();
    ok(&mut t, &mut files, "open-pack s.rpk");
    assert_eq!(t.document(), s.document());
    assert_eq!(
        ok(&mut t, &mut files, "store"),
        "memory unsaved=0 pack=s.rpk"
    );
    // A pack of a workspace leaves its store as it was.
    ok(&mut t, &mut files, "open w");
    ok(&mut t, &mut files, "add-slide");
    ok(&mut t, &mut files, "pack w.rpk");
    assert_eq!(ok(&mut t, &mut files, "store"), "workspace w unsaved=1");
}
