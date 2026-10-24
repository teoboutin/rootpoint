//! The session's store: a workspace's root is canonical, so a file is named by one asset path
//! whatever form of the root was given; no access to an asset reaches outside the root, whatever a
//! file names; a copy in memory holds every asset's bytes; an import copies a file from outside the
//! root under a name no other bytes hold; `store` replies the store and the steps since a save.

mod common;

use std::path::Path;

use common::*;
use rootpoint_model::*;
use rootpoint_protocol::{Reply, Session, Store};

/// The reproduction of the save-mixes defect, `milestone@plans@asset-storage`'s "Names": the same
/// file imported through a relative and an absolute form of the root is one asset at one path, and
/// is never copied beside itself.
#[test]
fn a_file_imported_through_two_forms_of_the_root_is_one_asset() {
    for absolute_first in [false, true] {
        let dir = scratch(&format!("save-mixes-{absolute_first}"));
        let d = dir.join("d");
        std::fs::create_dir_all(d.join("img")).unwrap();
        std::fs::write(d.join("img/cat.png"), two_colour_png()).unwrap();
        let absolute = d.to_str().unwrap().to_owned();
        let forms = if absolute_first {
            [absolute.clone(), "d".to_owned()]
        } else {
            ["d".to_owned(), absolute.clone()]
        };
        let script = format!(
            "open {0}\nadd-slide\nimport {0}/img/cat.png\nimage n2 a3 fit\nsave\nopen {1}\nimport {1}/img/cat.png\nsave\n",
            forms[0], forms[1]
        );
        let out = headless(&script, &dir);
        assert_eq!(
            out, "ok\nok s1 root=n2\nok a3\nok\nok\nok\nok a3\nok\n",
            "{script}"
        );
        let text = std::fs::read_to_string(d.join("presentation.ron")).unwrap();
        let doc = load_ron(&text).unwrap();
        let paths: Vec<&str> = doc.assets.values().map(|a| a.path.as_str()).collect();
        assert_eq!(paths, ["img/cat.png"], "{text}");
        assert_eq!(text.matches("img/cat.png").count(), 1, "{text}");
        assert!(!d.join("cat.png").exists());
    }
}

/// A workspace `root`, under `dir`, whose presentation names `paths` as assets, each shown by an
/// image area of its one slide; and a file `outside.png` beside the root, and one in `dir/away/`.
fn workspace_naming(dir: &Path, paths: &[&str]) -> std::path::PathBuf {
    let root = dir.join("root");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(dir.join("away")).unwrap();
    std::fs::write(dir.join("outside.png"), two_colour_png()).unwrap();
    std::fs::write(dir.join("away/outside.png"), two_colour_png()).unwrap();
    let mut doc = Document::default();
    apply(&mut doc, Op::AddSlide { at: 0 }).unwrap();
    let mut area = doc.slides[0].root.id();
    for (i, path) in paths.iter().enumerate() {
        let Op::RemoveAsset { id } = apply(
            &mut doc,
            Op::AddAsset {
                asset: Asset {
                    path: path.to_string(),
                },
            },
        )
        .unwrap() else {
            panic!("AddAsset returned another inverse");
        };
        if i > 0 {
            apply(
                &mut doc,
                Op::Split {
                    node: area,
                    axis: Axis::LeftToRight,
                    at: 0.5,
                    side: Side::After,
                },
            )
            .unwrap();
            area = doc.slides[0].root.areas().last().unwrap().id;
        }
        apply(
            &mut doc,
            Op::SetContent {
                area,
                content: Content::Image(ImageContent {
                    asset: id,
                    fit: ImageFit::Fit,
                }),
            },
        )
        .unwrap();
    }
    std::fs::write(root.join("presentation.ron"), save_ron(&doc)).unwrap();
    root
}

/// The asset paths a file edited by hand can name that lead out of the root, the outside file
/// by its absolute path among them.
fn unconfined(dir: &Path) -> Vec<String> {
    vec![
        "../outside.png".to_owned(),
        dir.join("outside.png").to_str().unwrap().to_owned(),
        ".rootpoint/outside.png".to_owned(),
        "presentation.ron".to_owned(),
    ]
}

fn run_ok(s: &mut Session, files: &mut RecordingFiles, line: &str) -> String {
    match s.execute(line, files) {
        Some(Reply::Ok(text)) => text,
        other => panic!("`{line}` gave {other:?}"),
    }
}

/// AC2 of `milestone@plans@asset-storage`, for the RON, `render` and `open-copy`: a file naming paths outside the
/// root loads, names each as a breach and draws each as missing, and no path outside the root
/// reaches the host's files; a symlink out of the root is refused at its read.
#[test]
fn no_read_of_a_workspace_reaches_outside_its_root() {
    let dir = std::fs::canonicalize(scratch("confined-read")).unwrap();
    let paths = unconfined(&dir);
    let mut named: Vec<&str> = paths.iter().map(String::as_str).collect();
    named.push("link.png");
    let root = workspace_naming(&dir, &named);
    std::os::unix::fs::symlink(dir.join("outside.png"), root.join("link.png")).unwrap();
    std::fs::create_dir_all(root.join(".rootpoint")).unwrap();
    std::fs::write(root.join(".rootpoint/outside.png"), two_colour_png()).unwrap();
    let root_text = root.to_str().unwrap();
    let mut files = RecordingFiles::default();
    let mut s = Session::new();
    run_ok(&mut s, &mut files, &format!("open {root_text}"));
    let breaches: Vec<Breach> = (1..=4)
        .map(|n| Breach::UnconfinedAsset {
            asset: AssetId(s.document().assets.keys().nth(n - 1).unwrap().0),
        })
        .collect();
    assert_eq!(s.document().breaches(), breaches);
    let ids: Vec<String> = s.document().assets.keys().map(|a| a.to_string()).collect();
    assert_eq!(
        run_ok(
            &mut s,
            &mut files,
            &format!("render s1 {root_text}/out.png 192 108")
        ),
        format!("fallback=- missing={} glyphs=- styles=-", ids.join(","))
    );
    for id in s.document().assets.keys() {
        assert!(s.asset_bytes(*id, &mut files).is_err(), "{id} was read");
    }
    assert_eq!(files.outside(&root), Vec::<String>::new());
    // A copy in memory reads through the same check, against the directory it copies.
    run_ok(&mut s, &mut files, &format!("open-copy {root_text}"));
    let Store::Memory(map) = s.store() else {
        panic!("open-copy did not open in memory");
    };
    assert!(map.is_empty(), "{:?}", map.keys().collect::<Vec<_>>());
    assert_eq!(files.outside(&root), Vec::<String>::new());
}

/// AC2 of `milestone@plans@asset-storage`, for `import`: a write through a symlink out of the root, its target there
/// or missing, and an import to a path that fails confinement, reply `err` and write nothing.
#[test]
fn no_import_writes_outside_its_root() {
    let dir = std::fs::canonicalize(scratch("confined-write")).unwrap();
    let root = workspace_naming(&dir, &[]);
    std::os::unix::fs::symlink(dir.join("away"), root.join("linked")).unwrap();
    std::os::unix::fs::symlink(dir.join("away/outside.png"), root.join("taken.png")).unwrap();
    // Two symlinks whose targets are missing: a write would create the target outside the root.
    std::os::unix::fs::symlink(dir.join("away/new.png"), root.join("dangling.png")).unwrap();
    std::os::unix::fs::symlink(dir.join("away/new"), root.join("nowhere")).unwrap();
    let root_text = root.to_str().unwrap();
    let source = dir.join("outside.png");
    let source = source.to_str().unwrap();
    let mut files = RecordingFiles::default();
    let mut s = Session::new();
    run_ok(&mut s, &mut files, &format!("open {root_text}"));
    let away = std::fs::read_dir(dir.join("away")).unwrap().count();
    let absolute = dir.join("cat.png");
    for to in [
        "linked/cat.png",
        "taken.png",
        "dangling.png",
        "nowhere/cat.png",
        "../cat.png",
        ".rootpoint/cat.png",
        "presentation.ron",
        absolute.to_str().unwrap(),
    ] {
        let before = s.document().clone();
        let reply = s.execute(&format!("import {source} {to}"), &mut files);
        assert!(matches!(reply, Some(Reply::Err(_))), "{to} gave {reply:?}");
        assert_eq!(s.document(), &before, "{to}");
    }
    assert_eq!(std::fs::read_dir(dir.join("away")).unwrap().count(), away);
    assert!(!dir.join("cat.png").exists());
    assert!(!dir.join("away/new.png").exists() && !dir.join("away/new").exists());
    // Outside the root, only the file imported was given to the host's files.
    let outside = files.outside(&root);
    assert!(outside.iter().all(|p| p == source), "{outside:?}");
}

/// `open-copy` reads every asset of the presentation into the map, so that no later command
/// reads the host's files for one.
#[test]
fn a_copy_in_memory_reads_no_file_of_the_host_after_its_open() {
    let mut files = MemoryFiles::default();
    files
        .files
        .insert("talk/img/cat.png".into(), two_colour_png());
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "open talk\nadd-slide\nimport talk/img/cat.png\nimage n2 a3 fit\nsave",
    );
    let mut t = Session::new();
    ok(&mut t, &mut files, "open-copy talk");
    assert_eq!(t.document(), s.document());
    files.files.clear();
    assert_eq!(
        ok(&mut t, &mut files, "render s1 out.png 192 108"),
        "fallback=- missing=- glyphs=- styles=-"
    );
}

/// An import from outside the root writes the file at the root under its name, takes `-2` where
/// the name holds other bytes, and keeps the name where it holds the same bytes, in a workspace and
/// in memory; a file below a workspace's root is added at its path, with no copy.
#[test]
fn an_import_copies_a_file_under_a_name_no_other_bytes_hold() {
    let host = || {
        let mut files = MemoryFiles::default();
        files.files.insert("a/cat.png".into(), b"one".to_vec());
        files.files.insert("b/cat.png".into(), b"two".to_vec());
        files.files.insert("c/cat.png".into(), b"one".to_vec());
        files.files.insert("d/cat.png".into(), b"three".to_vec());
        files.files.insert("w/img/dog.png".into(), b"dog".to_vec());
        files
    };
    for workspace in [true, false] {
        let mut files = host();
        let mut s = Session::new();
        if workspace {
            ok(&mut s, &mut files, "open w");
        }
        assert_eq!(ok(&mut s, &mut files, "import a/cat.png"), "a1");
        assert_eq!(ok(&mut s, &mut files, "import b/cat.png"), "a2");
        assert_eq!(ok(&mut s, &mut files, "import c/cat.png"), "a1");
        assert_eq!(ok(&mut s, &mut files, "import d/cat.png"), "a3");
        assert_eq!(ok(&mut s, &mut files, "import b/cat.png img/x.png"), "a4");
        assert_eq!(ok(&mut s, &mut files, "import a/cat.png img/x.png"), "a5");
        let paths: Vec<&str> = s
            .document()
            .assets
            .values()
            .map(|a| a.path.as_str())
            .collect();
        assert_eq!(
            paths,
            [
                "cat.png",
                "cat-2.png",
                "cat-3.png",
                "img/x.png",
                "img/x-2.png"
            ]
        );
        let held = |path: &str| match s.store() {
            Store::Memory(map) => map.get(path).cloned(),
            Store::Workspace(root) => files.files.get(&format!("{root}/{path}")).cloned(),
        };
        assert_eq!(held("cat.png"), Some(b"one".to_vec()));
        assert_eq!(held("cat-2.png"), Some(b"two".to_vec()));
        assert_eq!(held("cat-3.png"), Some(b"three".to_vec()));
        assert_eq!(held("img/x.png"), Some(b"two".to_vec()));
        assert_eq!(held("img/x-2.png"), Some(b"one".to_vec()));
    }
    let mut files = host();
    let mut s = Session::new();
    ok(&mut s, &mut files, "open w");
    let before = files.files.len();
    assert_eq!(ok(&mut s, &mut files, "import w/img/dog.png"), "a1");
    assert_eq!(s.document().assets[&AssetId(1)].path, "img/dog.png");
    assert_eq!(
        files.files.len(),
        before,
        "a file below the root was copied"
    );
    assert!(matches!(
        s.execute("import w/img/dog.png dog.png", &mut files),
        Some(Reply::Err(_))
    ));
}

/// `store` replies the store and the undo steps since the open or the save, which an edit, a
/// group, an undo and a redo raise, and a save resets.
#[test]
fn store_replies_the_store_and_the_steps_since_the_save() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    assert_eq!(ok(&mut s, &mut files, "store"), "memory unsaved=0");
    run(
        &mut s,
        &mut files,
        "add-slide\nbegin\ntext n2 \"a\"\ntext n2 \"b\"",
    );
    assert_eq!(ok(&mut s, &mut files, "store"), "memory unsaved=1");
    run(&mut s, &mut files, "end\nundo\nredo");
    assert_eq!(ok(&mut s, &mut files, "store"), "memory unsaved=4");
    run(
        &mut s,
        &mut files,
        "begin\ntext n2 \"c\"\ncancel\nview\ntree",
    );
    assert_eq!(ok(&mut s, &mut files, "store"), "memory unsaved=4");
    ok(&mut s, &mut files, "open \"my talk\"");
    assert_eq!(
        ok(&mut s, &mut files, "store"),
        "workspace \"my talk\" unsaved=0"
    );
    ok(&mut s, &mut files, "add-slide");
    assert_eq!(
        ok(&mut s, &mut files, "store"),
        "workspace \"my talk\" unsaved=1"
    );
    ok(&mut s, &mut files, "save");
    assert_eq!(
        ok(&mut s, &mut files, "store"),
        "workspace \"my talk\" unsaved=0"
    );
    assert!(files.files.contains_key("my talk/presentation.ron"));
}

/// The headless binary's own files take a symlink whose target is missing for a file there, so an
/// import to it is refused and creates no file outside the root.
#[test]
fn the_headless_binary_writes_through_no_dangling_symlink() {
    let dir = std::fs::canonicalize(scratch("dangling-headless")).unwrap();
    std::fs::create_dir_all(dir.join("ws")).unwrap();
    std::fs::create_dir_all(dir.join("out")).unwrap();
    std::fs::write(dir.join("src.png"), b"bytes").unwrap();
    std::os::unix::fs::symlink(dir.join("out/escaped.png"), dir.join("ws/cat.png")).unwrap();
    let out = headless("open ws\nimport src.png cat.png\n", &dir);
    assert!(out.starts_with("ok\nerr "), "{out}");
    assert!(!dir.join("out/escaped.png").exists());
}

/// `reload` reads `presentation.ron` at a workspace's root again and makes it the document, one
/// undo step that brings the session's own back; a file that does not load, and a store in memory,
/// reply `err` and change nothing.
#[test]
fn reload_makes_the_file_the_document_one_undo_step() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(&mut s, &mut files, "open w\nadd-slide\nsave");
    let mut outside = Session::new();
    run(&mut outside, &mut files, "add-slide\nadd-slide\nadd-slide");
    write_presentation(&outside, &mut files, "w");
    ok(&mut s, &mut files, "add-slide");
    let mine = s.document().clone();
    assert_eq!(ok(&mut s, &mut files, "reload"), "");
    assert_eq!(s.document(), outside.document());
    ok(&mut s, &mut files, "undo");
    // The undo keeps the larger id counter, so the content alone is compared.
    assert!(s.document().same_content(&mine));
    files
        .files
        .insert("w/presentation.ron".into(), b"not ron".to_vec());
    assert!(matches!(
        s.execute("reload", &mut files),
        Some(Reply::Err(_))
    ));
    assert!(s.document().same_content(&mine));
    ok(&mut s, &mut files, "new");
    assert!(matches!(
        s.execute("reload", &mut files),
        Some(Reply::Err(_))
    ));
}
