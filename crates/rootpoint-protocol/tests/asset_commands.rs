//! The asset commands and the trash, in a workspace and in memory: a removal moves an asset's file
//! to the trash, and its undo moves it back with its bytes; an open of the workspace empties the
//! trash; a move keeps every slide; a relink repairs a missing asset; a removal with `into=` makes
//! every use name the replacement; `assets untracked` lists the files no asset names; and on an
//! asset whose path fails confinement, no command reaches outside the root.

mod common;

use std::path::{Path, PathBuf};

use common::*;
use rootpoint_model::*;
use rootpoint_protocol::{Files, Reply, Session, Store};

fn run_ok(s: &mut Session, files: &mut dyn Files, line: &str) -> String {
    match s.run(rootpoint_protocol::parse_command(line).unwrap(), files) {
        Reply::Ok(text) => text,
        other => panic!("`{line}` gave {other}"),
    }
}

fn refused(s: &mut Session, files: &mut dyn Files, line: &str) -> String {
    match s.run(rootpoint_protocol::parse_command(line).unwrap(), files) {
        Reply::Err(text) => text,
        other => panic!("`{line}` gave {other}"),
    }
}

/// A workspace whose root holds `img/a.png` and `b.png`, opened, its one slide showing `a3`, the
/// image `img/a.png`, and `b.png` imported as `a4`.
fn workspace(name: &str) -> (PathBuf, Session, RecordingFiles) {
    let root = std::fs::canonicalize(scratch(name)).unwrap();
    std::fs::create_dir_all(root.join("img")).unwrap();
    std::fs::write(root.join("img/a.png"), two_colour_png()).unwrap();
    std::fs::write(root.join("b.png"), b"b's bytes").unwrap();
    let mut files = RecordingFiles::default();
    let mut s = Session::new();
    let r = root.to_str().unwrap();
    for line in [
        format!("open {r}"),
        "add-slide".to_owned(),
        format!("import {r}/img/a.png"),
        "image n2 a3 fit".to_owned(),
        format!("import {r}/b.png"),
    ] {
        run_ok(&mut s, &mut files, &line);
    }
    (root, s, files)
}

/// The entries of the workspace's trash.
fn trash(root: &Path) -> Vec<String> {
    match std::fs::read_dir(root.join(".rootpoint/trash")) {
        Ok(entries) => {
            let mut names: Vec<String> = entries
                .map(|e| e.unwrap().file_name().into_string().unwrap())
                .collect();
            names.sort();
            names
        }
        Err(_) => Vec::new(),
    }
}

/// AC4 of `milestone@plans@asset-storage`: a removal moves the file to the trash, its undo moves
/// it back with its bytes, its redo moves it again; a new session's open of the workspace empties
/// the trash.
#[test]
fn a_removal_goes_to_the_trash_and_its_undo_brings_the_file_back() {
    let (root, mut s, mut files) = workspace("trash-undo");
    run_ok(&mut s, &mut files, "asset-remove a4");
    assert!(!root.join("b.png").exists());
    assert_eq!(trash(&root), ["a4-1"]);
    // A file put at the path meanwhile makes the undo fail whole: no move, no operation.
    std::fs::write(root.join("b.png"), b"another").unwrap();
    refused(&mut s, &mut files, "undo");
    assert!(!s.document().assets.contains_key(&AssetId(4)));
    assert_eq!(trash(&root), ["a4-1"]);
    std::fs::remove_file(root.join("b.png")).unwrap();
    run_ok(&mut s, &mut files, "undo");
    assert_eq!(std::fs::read(root.join("b.png")).unwrap(), b"b's bytes");
    assert_eq!(trash(&root), Vec::<String>::new());
    assert!(s.document().assets.contains_key(&AssetId(4)));
    run_ok(&mut s, &mut files, "redo");
    assert!(!root.join("b.png").exists());
    run_ok(&mut s, &mut files, "save");
    // The session ends; the next one's open of the workspace empties the trash.
    drop(s);
    let mut s = Session::new();
    run_ok(&mut s, &mut files, &format!("open {}", root.display()));
    assert_eq!(trash(&root), Vec::<String>::new());
}

/// In memory, the trash is the session's own: the undo of a removal gives the map its bytes back.
#[test]
fn in_memory_the_undo_of_a_removal_brings_the_bytes_back() {
    let mut files = MemoryFiles::holding(&["cat.png", "dog.png"]);
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nimport cat.png\nimport dog.png",
    );
    run_ok(&mut s, &mut files, "asset-remove a4");
    let Store::Memory(map) = s.store() else {
        panic!("not in memory")
    };
    assert!(!map.contains_key("dog.png"));
    run_ok(&mut s, &mut files, "undo");
    let Store::Memory(map) = s.store() else {
        panic!("not in memory")
    };
    assert_eq!(map["dog.png"], b"dog.png");
    // A replace keeps two versions of one asset: its undo and its redo take each back in turn.
    files.files.insert("new.png".into(), b"new bytes".to_vec());
    run_ok(&mut s, &mut files, "asset-replace a3 new.png");
    run_ok(&mut s, &mut files, "asset-replace a3 dog.png");
    assert_eq!(s.asset_bytes(AssetId(3), &mut files).unwrap(), b"dog.png");
    run_ok(&mut s, &mut files, "undo");
    assert_eq!(s.asset_bytes(AssetId(3), &mut files).unwrap(), b"new bytes");
    run_ok(&mut s, &mut files, "undo");
    assert_eq!(s.asset_bytes(AssetId(3), &mut files).unwrap(), b"cat.png");
    run_ok(&mut s, &mut files, "redo");
    assert_eq!(s.asset_bytes(AssetId(3), &mut files).unwrap(), b"new bytes");
}

/// A move renames the file, every area that named the asset still draws it, and the saved
/// presentation names the new path; a move to a path that holds a file, of an asset whose file is
/// missing, or to a path that fails confinement, is refused.
#[test]
fn a_move_keeps_every_slide() {
    let (root, mut s, mut files) = workspace("move");
    let r = root.display();
    run_ok(&mut s, &mut files, "asset-move a3 pictures/cat.png");
    assert!(root.join("pictures/cat.png").exists());
    assert!(!root.join("img/a.png").exists());
    assert_eq!(
        run_ok(
            &mut s,
            &mut files,
            &format!("render s1 {r}/out.png 192 108")
        ),
        "fallback=- missing=- glyphs=- styles=-"
    );
    run_ok(&mut s, &mut files, "save");
    let text = std::fs::read_to_string(root.join("presentation.ron")).unwrap();
    assert!(
        text.contains("pictures/cat.png") && !text.contains("img/a.png"),
        "{text}"
    );
    refused(&mut s, &mut files, "asset-move a3 b.png");
    refused(&mut s, &mut files, "asset-move a3 ../cat.png");
    refused(&mut s, &mut files, "asset-move a3 .rootpoint/cat.png");
    std::fs::remove_file(root.join("b.png")).unwrap();
    let reason = refused(&mut s, &mut files, "asset-move a4 c.png");
    assert!(reason.contains("asset-relink"), "{reason}");
    run_ok(&mut s, &mut files, "undo");
    assert!(root.join("img/a.png").exists());
    assert!(!root.join("pictures/cat.png").exists());
}

/// An asset whose file is gone, relinked to an untracked file, draws it, and `assets missing` no
/// longer lists it; `assets untracked` lists the files no asset names, never `.rootpoint/` or
/// `presentation.ron`.
#[test]
fn a_relink_repairs_a_missing_asset() {
    let (root, mut s, mut files) = workspace("relink");
    run_ok(&mut s, &mut files, "save");
    std::fs::rename(root.join("img/a.png"), root.join("moved.png")).unwrap();
    assert_eq!(
        run_ok(&mut s, &mut files, "assets missing"),
        "a3 img/a.png missing uses=n2"
    );
    assert_eq!(run_ok(&mut s, &mut files, "assets untracked"), "moved.png");
    refused(&mut s, &mut files, "asset-relink a3 nothing.png");
    refused(&mut s, &mut files, "asset-relink a3 b.png");
    run_ok(&mut s, &mut files, "asset-relink a3 moved.png");
    assert_eq!(run_ok(&mut s, &mut files, "assets missing"), "");
    assert_eq!(run_ok(&mut s, &mut files, "assets untracked"), "");
    assert_eq!(
        run_ok(&mut s, &mut files, "assets"),
        format!(
            "a3 moved.png {} uses=n2; a4 b.png 9 uses=-",
            two_colour_png().len()
        )
    );
    assert_eq!(
        run_ok(&mut s, &mut files, "assets unused"),
        "a4 b.png 9 uses=-"
    );
}

/// A removal with `into=` makes every image, text and named style that named the asset name the
/// replacement, one undo step; with none, a used asset is refused.
#[test]
fn a_removal_into_another_asset_renames_every_use() {
    let mut files = MemoryFiles::holding(&["cat.png", "font.ttf", "dog.png"]);
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nimport cat.png\nimage n2 a3 fit\nsplit n2 beside 0.5\n\
         import font.ttf\nimport dog.png\ntext n4 \"x\" font=a6\nstyle set \"Title\" font=a6",
    );
    let reason = refused(&mut s, &mut files, "asset-remove a6");
    assert_eq!(reason, "a6 is used by n4 \"Title\"");
    refused(&mut s, &mut files, "asset-remove a6 into=a6");
    refused(&mut s, &mut files, "asset-remove a6 into=a99");
    let before = s.document().clone();
    run_ok(&mut s, &mut files, "asset-remove a6 into=a7");
    let doc = s.document();
    assert!(!doc.assets.contains_key(&AssetId(6)));
    assert_eq!(doc.styles["Title"].font, FontRef::Asset(AssetId(7)));
    let Some(Node::Area(area)) = doc.node(NodeId(4)) else {
        panic!("no area n4")
    };
    let Content::Text(TextContent {
        style: StyleChoice::Custom(style),
        ..
    }) = &area.content
    else {
        panic!("n4 holds no custom text")
    };
    assert_eq!(style.font, FontRef::Asset(AssetId(7)));
    run_ok(&mut s, &mut files, "asset-remove a3 into=a7");
    run_ok(&mut s, &mut files, "undo");
    run_ok(&mut s, &mut files, "undo");
    assert!(s.document().same_content(&before), "one undo step each");
    // The unused assets leave in one step; with none unused, no step: the undo then undoes the
    // step before.
    run_ok(&mut s, &mut files, "image n2 a7 fit");
    assert_eq!(run_ok(&mut s, &mut files, "asset-remove-unused"), "a3");
    assert_eq!(run_ok(&mut s, &mut files, "asset-remove-unused"), "");
    run_ok(&mut s, &mut files, "undo");
    assert!(s.document().assets.contains_key(&AssetId(3)));
    run_ok(&mut s, &mut files, "undo");
    assert!(s.document().same_content(&before));
}

/// A workspace whose presentation names, beside `img/a.png`, an asset for each path that fails
/// confinement, unused, and the files those paths would reach.
fn workspace_with_unconfined(name: &str) -> (PathBuf, PathBuf, Session, RecordingFiles) {
    let dir = std::fs::canonicalize(scratch(name)).unwrap();
    let root = dir.join("root");
    std::fs::create_dir_all(root.join("img")).unwrap();
    std::fs::create_dir_all(root.join(".rootpoint")).unwrap();
    std::fs::write(root.join("img/a.png"), two_colour_png()).unwrap();
    std::fs::write(dir.join("outside.png"), b"outside").unwrap();
    std::fs::write(root.join(".rootpoint/outside.png"), b"inside .rootpoint").unwrap();
    let mut doc = Document::default();
    for path in [
        "../outside.png".to_owned(),
        dir.join("outside.png").to_str().unwrap().to_owned(),
        ".rootpoint/outside.png".to_owned(),
        "presentation.ron".to_owned(),
    ] {
        apply(
            &mut doc,
            Op::AddAsset {
                asset: Asset { path },
            },
        )
        .unwrap();
    }
    std::fs::write(root.join("presentation.ron"), save_ron(&doc)).unwrap();
    let mut files = RecordingFiles::default();
    let mut s = Session::new();
    run_ok(&mut s, &mut files, &format!("open {}", root.display()));
    (dir, root, s, files)
}

/// AC2 of `milestone@plans@asset-storage`, on the asset commands: on an asset whose path fails
/// confinement, `asset-move` and `asset-replace` reply `err`, `asset-remove` removes the entry and
/// moves no file, its undo restoring the entry, and `asset-relink` to a confined path repairs it;
/// no path outside the root reaches the host's files, but the host file `asset-replace` reads.
#[test]
fn no_asset_command_reaches_outside_the_root() {
    let (dir, root, mut s, mut files) = workspace_with_unconfined("confined-commands");
    let ids: Vec<AssetId> = s.document().assets.keys().copied().collect();
    std::fs::write(dir.join("new.png"), b"new").unwrap();
    for id in &ids {
        refused(&mut s, &mut files, &format!("asset-move {id} moved.png"));
        refused(
            &mut s,
            &mut files,
            &format!("asset-replace {id} {}/new.png", dir.display()),
        );
    }
    run_ok(&mut s, &mut files, &format!("asset-remove {}", ids[0]));
    run_ok(&mut s, &mut files, &format!("asset-remove {}", ids[2]));
    assert_eq!(trash(&root), Vec::<String>::new(), "no file moved");
    run_ok(&mut s, &mut files, "undo");
    assert!(s.document().assets.contains_key(&ids[2]));
    run_ok(
        &mut s,
        &mut files,
        &format!("asset-relink {} img/a.png", ids[1]),
    );
    assert_eq!(s.document().assets[&ids[1]].path, "img/a.png");
    run_ok(&mut s, &mut files, "asset-remove-unused");
    run_ok(&mut s, &mut files, "assets");
    run_ok(&mut s, &mut files, "assets untracked");
    run_ok(&mut s, &mut files, "undo");
    assert_eq!(std::fs::read(dir.join("outside.png")).unwrap(), b"outside");
    assert_eq!(
        std::fs::read(root.join(".rootpoint/outside.png")).unwrap(),
        b"inside .rootpoint"
    );
    let new = dir.join("new.png").to_str().unwrap().to_owned();
    let outside: Vec<String> = files
        .outside(&root)
        .into_iter()
        .filter(|p| *p != new)
        .collect();
    assert_eq!(outside, Vec::<String>::new());
}
