//! The asset commands and the trash, in a workspace and in memory: a removal moves an asset's file
//! to the trash, and its undo moves it back with its bytes; an open of the workspace empties the
//! trash; a move keeps every slide; a relink repairs a missing asset; a removal with `into=` makes
//! every use name the replacement; `assets untracked` lists the files no asset names; and on an
//! asset whose path fails confinement, no command reaches outside the root.

mod common;

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

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

/// A removal moves the file to the trash, `design@rootpoint-protocol@asset-trash`: its undo moves
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
        "fallback=- missing=- glyphs=- styles=- span-styles=- commands=- refs=-"
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
    // The store's own files, and those of a folder whose name starts with `.`, as `.git/`, are
    // never listed.
    std::fs::create_dir_all(root.join(".rootpoint")).unwrap();
    std::fs::write(root.join(".rootpoint/kept.png"), b"x").unwrap();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::write(root.join(".git/HEAD"), b"x").unwrap();
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

/// A workspace whose presentation names an asset for each path that fails confinement, unused, and
/// the files those paths would reach; `img/a.png` is a file no asset names.
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
        "mimetype".to_owned(),
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

/// `design@rootpoint-protocol@store-confines-access`, on the asset commands: on an asset whose
/// path fails confinement, `asset-move` and `asset-replace` reply `err`, `asset-remove` removes the
/// entry and moves no file, its undo restoring the entry, and `asset-relink` to a confined path
/// repairs it; no path outside the root reaches the host's files, but the host file
/// `asset-replace` reads.
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

fn reply(s: &mut Session, files: &mut dyn Files, line: &str) -> Reply {
    s.run(rootpoint_protocol::parse_command(line).unwrap(), files)
}

/// The workspace of `workspace`, and a directory `outside` beside its root.
fn workspace_beside(name: &str) -> (PathBuf, PathBuf, Session, RecordingFiles) {
    let (root, s, files) = workspace(&format!("{name}/root"));
    let outside = root.parent().unwrap().join("outside");
    let _ = std::fs::remove_dir_all(&outside);
    std::fs::create_dir_all(&outside).unwrap();
    (root, outside, s, files)
}

/// An `open` empties no `.rootpoint/trash` that a symlink places outside the root: the files of
/// the directory it names stay.
#[test]
fn an_open_empties_no_trash_a_symlink_places_outside() {
    let dir = std::fs::canonicalize(scratch("r5-trash-symlink-open")).unwrap();
    let root = dir.join("root");
    let victim = dir.join("victim");
    std::fs::create_dir_all(root.join(".rootpoint")).unwrap();
    std::fs::create_dir_all(victim.join("sub")).unwrap();
    std::fs::write(victim.join("thesis.txt"), b"years of work").unwrap();
    std::fs::write(victim.join("sub/notes.txt"), b"notes").unwrap();
    std::os::unix::fs::symlink(&victim, root.join(".rootpoint/trash")).unwrap();
    let mut files = RecordingFiles::default();
    let mut s = Session::new();
    run_ok(&mut s, &mut files, &format!("open {}", root.display()));
    assert!(
        victim.join("thesis.txt").exists(),
        "open deleted a file outside the root"
    );
    assert!(
        victim.join("sub/notes.txt").exists(),
        "open deleted a file outside the root"
    );
}

/// A removal moves no file through a trash that a symlink places outside the root.
#[test]
fn a_removal_moves_no_file_through_a_trash_symlink() {
    let (root, outside, mut s, mut files) = workspace_beside("r5-trash-symlink-remove");
    std::fs::create_dir_all(root.join(".rootpoint")).unwrap();
    let _ = std::fs::remove_dir_all(root.join(".rootpoint/trash"));
    std::os::unix::fs::symlink(&outside, root.join(".rootpoint/trash")).unwrap();
    let _ = reply(&mut s, &mut files, "asset-remove a4");
    let escaped: Vec<_> = std::fs::read_dir(&outside).unwrap().collect();
    assert!(
        escaped.is_empty(),
        "the file of a4 left the root: {escaped:?}"
    );
}

/// In memory as in a workspace, a move onto a folder of the map is refused, so the map can always
/// be unpacked.
#[test]
fn in_memory_a_move_onto_a_folder_is_refused() {
    let mut mem = MemoryFiles::holding(&["cat.png", "dog.png"]);
    let mut s = Session::new();
    run(
        &mut s,
        &mut mem,
        "add-slide\nimport cat.png img/cat.png\nimport dog.png",
    );
    assert!(
        s.document()
            .assets
            .values()
            .any(|a| a.path == "img/cat.png")
    );
    let moved = reply(&mut s, &mut mem, "asset-move a4 img");
    let unpack_dir = std::fs::canonicalize(scratch("r5-unpack-dir")).unwrap();
    let mut files = RecordingFiles::default();
    let unpacked = reply(
        &mut s,
        &mut files,
        &format!("unpack {}", unpack_dir.join("w").display()),
    );
    assert!(
        matches!(moved, Reply::Err(_)),
        "memory accepted a move onto the directory `img` ({moved}); the unpack then gave {unpacked}"
    );
}

/// The undo of an import takes its file back, so the undo of a move before it finds its path free.
#[test]
fn the_undo_of_a_move_after_an_undone_import_succeeds() {
    let mut mem = MemoryFiles::holding(&["cat.png", "other.png"]);
    let mut s = Session::new();
    run(&mut s, &mut mem, "add-slide\nimport cat.png");
    run_ok(&mut s, &mut mem, "asset-move a3 moved.png");
    run_ok(&mut s, &mut mem, "import other.png cat.png");
    run_ok(&mut s, &mut mem, "undo");
    let undo = reply(&mut s, &mut mem, "undo");
    assert!(
        matches!(undo, Reply::Ok(_)),
        "the undo of the move gave {undo}"
    );
}

/// The trash keeps no bytes no step reaches: a replace undone, then an edit, which clears the redo
/// stack, takes the replace's new bytes out of the trash.
#[test]
fn the_trash_holds_no_entry_no_step_reaches() {
    let (root, outside, mut s, mut files) = workspace_beside("r5-trash-orphan");
    std::fs::write(outside.join("new.png"), b"new bytes").unwrap();
    for _ in 0..3 {
        run_ok(
            &mut s,
            &mut files,
            &format!("asset-replace a4 {}", outside.join("new.png").display()),
        );
        run_ok(&mut s, &mut files, "undo");
        run_ok(&mut s, &mut files, "add-slide");
    }
    // No step left can move anything back from the trash: the undo stack holds only the
    // add-slides and the imports, the redo stack is empty.
    let entries: Vec<String> = match std::fs::read_dir(root.join(".rootpoint/trash")) {
        Ok(rd) => rd
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect(),
        Err(_) => Vec::new(),
    };
    assert_eq!(entries, Vec::<String>::new(), "unreachable trash entries");
}

/// A move or a replace of a file another asset names too is refused: it would change that asset.
#[test]
fn a_file_two_assets_name_is_neither_moved_nor_replaced() {
    let mut doc = Document::default();
    for _ in 0..2 {
        apply(
            &mut doc,
            Op::AddAsset {
                asset: Asset {
                    path: "cat.png".to_owned(),
                },
            },
        )
        .unwrap();
    }
    let ids: Vec<AssetId> = doc.assets.keys().copied().collect();
    let dir = std::fs::canonicalize(scratch("r5-shared-move")).unwrap();
    std::fs::write(dir.join("presentation.ron"), save_ron(&doc)).unwrap();
    std::fs::write(dir.join("cat.png"), b"cat").unwrap();
    let mut files = RecordingFiles::default();
    let mut s = Session::new();
    run_ok(&mut s, &mut files, &format!("open {}", dir.display()));
    let moved = reply(
        &mut s,
        &mut files,
        &format!("asset-move {} dog.png", ids[0]),
    );
    let other = s.asset_bytes(ids[1], &mut files);
    assert!(
        other.is_ok(),
        "after `{moved}`, {} lost its file: {other:?}",
        ids[1]
    );
    std::fs::write(dir.join("new.png"), b"new").unwrap();
    let replaced = reply(
        &mut s,
        &mut files,
        &format!("asset-replace {} {}/new.png", ids[0], dir.display()),
    );
    assert!(matches!(replaced, Reply::Err(_)), "{replaced}");
    assert_eq!(s.asset_bytes(ids[1], &mut files).unwrap(), b"cat");
}

/// An asset whose file is a symlink out of the root can still leave the table, and a replace of
/// it, refused, leaves nothing in the trash.
#[test]
fn an_asset_whose_file_leads_out_of_the_root_leaves_the_table() {
    let (root, outside, s, mut files) = workspace_beside("symlinked-asset");
    std::fs::write(outside.join("away.png"), b"away").unwrap();
    std::os::unix::fs::symlink(outside.join("away.png"), root.join("l.png")).unwrap();
    drop(s);
    let mut doc = Document::default();
    apply(
        &mut doc,
        Op::AddAsset {
            asset: Asset {
                path: "l.png".to_owned(),
            },
        },
    )
    .unwrap();
    std::fs::write(root.join("presentation.ron"), save_ron(&doc)).unwrap();
    let mut s = Session::new();
    run_ok(&mut s, &mut files, &format!("open {}", root.display()));
    let id = *s.document().assets.keys().next().unwrap();
    std::fs::write(outside.join("new.png"), b"new").unwrap();
    refused(
        &mut s,
        &mut files,
        &format!("asset-replace {id} {}/new.png", outside.display()),
    );
    assert_eq!(trash(&root), Vec::<String>::new());
    run_ok(&mut s, &mut files, &format!("asset-remove {id}"));
    assert!(s.document().assets.is_empty());
    assert_eq!(std::fs::read(outside.join("away.png")).unwrap(), b"away");
}

/// In memory, an import whose `to` lies below a file of the map is refused, and returns: no suffix
/// of the name frees the folder the file holds.
#[test]
fn in_memory_an_import_below_a_file_returns() {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut mem = MemoryFiles::holding(&["pic.png", "other.png"]);
        let mut s = Session::new();
        run_ok(&mut s, &mut mem, "add-slide");
        run_ok(&mut s, &mut mem, "import pic.png logo");
        let r = reply(&mut s, &mut mem, "import other.png logo/x.png");
        let _ = tx.send(format!("{r}"));
    });
    let got = rx.recv_timeout(Duration::from_secs(10));
    assert!(
        got.is_ok(),
        "`import other.png logo/x.png` gave no reply in 10 s: the suffix loop never ends"
    );
}

/// A group that ends takes with it the trash's entries only its copy of the redo stack reached: a
/// replace undone, then a group of an edit, leaves an empty trash.
#[test]
fn a_group_that_ends_leaves_no_unreachable_trash_entry() {
    let root = std::fs::canonicalize(scratch("r6-group-trash")).unwrap();
    std::fs::write(root.join("b.png"), b"b's bytes").unwrap();
    let outside = root
        .join("..")
        .canonicalize()
        .unwrap()
        .join("r6-group-trash-new.png");
    std::fs::write(&outside, b"new bytes").unwrap();
    let mut files = RecordingFiles::default();
    let mut s = Session::new();
    let r = root.to_str().unwrap();
    run_ok(&mut s, &mut files, &format!("open {r}"));
    run_ok(&mut s, &mut files, "add-slide");
    let id = run_ok(&mut s, &mut files, &format!("import {r}/b.png"));
    run_ok(
        &mut s,
        &mut files,
        &format!("asset-replace {id} {}", outside.display()),
    );
    run_ok(&mut s, &mut files, "undo");
    run_ok(&mut s, &mut files, "begin");
    run_ok(&mut s, &mut files, "add-slide");
    run_ok(&mut s, &mut files, "end");
    run_ok(&mut s, &mut files, "add-slide");
    let entries: Vec<String> = match std::fs::read_dir(root.join(".rootpoint/trash")) {
        Ok(rd) => rd
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect(),
        Err(_) => Vec::new(),
    };
    // The undo stack holds the import, the add-slides and the group; the redo stack is empty.
    assert_eq!(entries, Vec::<String>::new(), "unreachable trash entries");
}

/// In a workspace, an import at a name a folder holds takes the name with a suffix, as the store in
/// memory does.
#[test]
fn in_a_workspace_an_import_at_a_folder_s_name_takes_a_suffix() {
    let root = std::fs::canonicalize(scratch("r6-import-folder")).unwrap();
    std::fs::create_dir_all(root.join("img")).unwrap();
    std::fs::write(root.join("img/a.png"), b"a").unwrap();
    let outside = root
        .join("..")
        .canonicalize()
        .unwrap()
        .join("r6-import-folder-x");
    std::fs::write(&outside, b"x").unwrap();
    let mut files = RecordingFiles::default();
    let mut s = Session::new();
    run_ok(&mut s, &mut files, &format!("open {}", root.display()));
    let got = reply(
        &mut s,
        &mut files,
        &format!("import {} img", outside.display()),
    );
    // The same import in memory, for comparison.
    let mut mem = MemoryFiles::holding(&["x", "img/a.png"]);
    let mut m = Session::new();
    run_ok(&mut m, &mut mem, "import img/a.png img/a.png");
    let in_memory = reply(&mut m, &mut mem, "import x img");
    assert!(
        matches!(got, Reply::Ok(_)),
        "on disk: {got}; in memory: {in_memory}"
    );
}

/// In a workspace, an import whose `to` lies below a file is refused, as in memory, and leaves no
/// file in the trash.
#[test]
fn in_a_workspace_an_import_below_a_file_is_refused_and_leaves_no_trash() {
    let root = std::fs::canonicalize(scratch("import-below-file")).unwrap();
    std::fs::write(root.join("logo"), b"a file").unwrap();
    let outside = root.parent().unwrap().join("import-below-file-x.png");
    std::fs::write(&outside, b"x").unwrap();
    let mut files = RecordingFiles::default();
    let mut s = Session::new();
    run_ok(&mut s, &mut files, &format!("open {}", root.display()));
    run_ok(&mut s, &mut files, "add-slide");
    let got = reply(
        &mut s,
        &mut files,
        &format!("import {} logo/x.png", outside.display()),
    );
    assert!(matches!(got, Reply::Err(_)), "{got}");
    assert_eq!(trash(&root), Vec::<String>::new(), "{got}");
}

/// A font a span or a span style sets is a use of its asset, as the model counts it: `assets` names
/// its area or the style, a removal is refused with its use, `asset-remove-unused` leaves it, and a
/// removal into another asset makes the span or the style name the other. A file holds such a span before any command makes one.
#[test]
fn a_font_a_span_sets_is_a_use_of_its_asset() {
    let mut files = MemoryFiles::holding(&["font.ttf", "other.ttf", "spare.ttf"]);
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\ntext n2 \"abc\"\nimport font.ttf\nimport other.ttf\nimport spare.ttf",
    );
    write_presentation(&s, &mut files, "deck");
    for name in ["font.ttf", "other.ttf", "spare.ttf"] {
        files
            .files
            .insert(format!("deck/{name}"), b"bytes".to_vec());
    }
    let saved = String::from_utf8(files.files["deck/presentation.ron"].clone()).unwrap();
    let mut doc = load_ron(&saved).unwrap();
    apply(
        &mut doc,
        Op::WrapSpan {
            area: NodeId(2),
            node: TextNodeRef::Root,
            range: 0..1,
            style: SpanChoice::Custom(SpanStyle {
                font: Some(FontRef::Asset(AssetId(3))),
                ..SpanStyle::default()
            }),
        },
    )
    .unwrap();
    // And a span style whose font is a5.
    apply(
        &mut doc,
        Op::AddSpanStyle {
            name: "Code".to_owned(),
            style: SpanStyle {
                font: Some(FontRef::Asset(AssetId(5))),
                ..SpanStyle::default()
            },
        },
    )
    .unwrap();
    files
        .files
        .insert("deck/presentation.ron".into(), save_ron(&doc).into_bytes());
    let mut s = Session::new();
    run_ok(&mut s, &mut files, "open deck");
    let assets = run_ok(&mut s, &mut files, "assets");
    assert!(assets.contains("a3 font.ttf 5 uses=n2"), "{assets}");
    assert!(
        assets.contains("a5 spare.ttf 5 uses=span:\"Code\""),
        "{assets}"
    );
    assert_eq!(
        refused(&mut s, &mut files, "asset-remove a3"),
        "a3 is used by n2"
    );
    assert_eq!(run_ok(&mut s, &mut files, "asset-remove-unused"), "a4");
    run_ok(&mut s, &mut files, "undo");
    run_ok(&mut s, &mut files, "asset-remove a5 into=a4");
    assert_eq!(
        s.document().span_styles["Code"].font,
        Some(FontRef::Asset(AssetId(4)))
    );
    run_ok(&mut s, &mut files, "asset-remove a3 into=a4");
    let Some(Node::Area(Area {
        content: Content::Text(text),
        ..
    })) = s.document().node(NodeId(2))
    else {
        panic!("n2 holds no text")
    };
    // The text names Body, so the span's font is its one own font.
    assert_eq!(text.own_fonts(), vec![&FontRef::Asset(AssetId(4))]);
}
