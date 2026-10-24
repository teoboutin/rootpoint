//! The native window's workspace saves itself after every undo step, on a thread of its own, and
//! writes only the newest of a burst; its guard writes nothing over a `presentation.ron` changed
//! outside rootpoint, and asks; the watcher decodes an asset again when its file changes, and
//! nothing when `.git/` does. The benchmark of AC1 times the GUI thread's share of an autosave.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::{Context, RawInput, ViewportCommand, ViewportEvent, ViewportId};
use rootpoint_app::App;
use rootpoint_model::{Document, load_ron, save_ron};
use rootpoint_protocol::{Files, Reply, parse_command};

/// The native file system, paths relative to the working directory.
struct NativeFiles;

impl Files for NativeFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|e| format!("{path}: {e}"))
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        if let Some(dir) = Path::new(path).parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{path}: {e}"))?;
        }
        std::fs::write(path, bytes).map_err(|e| format!("{path}: {e}"))
    }

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        // Anything at the path, a symlink whose target is missing included, as `Files::exists` asks.
        match std::fs::symlink_metadata(path) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(format!("{path}: {e}")),
        }
    }

    fn canonical(&mut self, path: &str) -> Result<String, String> {
        let canonical = std::fs::canonicalize(path).map_err(|e| format!("{path}: {e}"))?;
        Ok(canonical.to_str().unwrap().to_owned())
    }
}

/// An empty directory of the build's scratch space, canonical.
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::canonicalize(dir).unwrap()
}

fn ok(app: &mut App, line: &str) {
    let reply = app.run(parse_command(line).unwrap());
    assert!(matches!(reply, Reply::Ok(_)), "{line}: {reply}");
}

fn frame(ctx: &Context, app: &mut App) {
    let mut out = ctx.run_ui(RawInput::default(), |ui| app.frame(ui));
    out.textures_delta.clear();
}

/// One frame in which the close of the window is asked: whether the window cancelled it.
fn close(ctx: &Context, app: &mut App) -> bool {
    let mut input = RawInput::default();
    input
        .viewports
        .entry(ViewportId::ROOT)
        .or_default()
        .events
        .push(ViewportEvent::Close);
    let mut out = ctx.run_ui(input, |ui| app.frame(ui));
    out.textures_delta.clear();
    out.viewport_output
        .get(&ViewportId::ROOT)
        .is_some_and(|v| v.commands.contains(&ViewportCommand::CancelClose))
}

/// A document of three slides, as another program writes it.
fn three_slides() -> Document {
    let mut outside = Document::default();
    for at in 0..3 {
        rootpoint_model::apply(&mut outside, rootpoint_model::Op::AddSlide { at }).unwrap();
    }
    outside
}

/// The document `presentation.ron` holds.
fn on_disk(root: &Path) -> Document {
    load_ron(&std::fs::read_to_string(root.join("presentation.ron")).unwrap()).unwrap()
}

/// A native window holding the workspace `root`, its autosave started.
fn holding(root: &Path) -> App {
    let mut app = App::native(Box::new(NativeFiles));
    ok(&mut app, &format!("open {}", root.display()));
    app
}

#[test]
fn an_autosave_follows_each_step_and_writes_the_newest_of_a_burst() {
    let root = scratch("autosave-burst");
    let mut app = holding(&root);
    ok(&mut app, "add-slide");
    app.autosave().unwrap().wait_idle();
    assert_eq!(app.autosave().unwrap().writes(), 1);
    assert_eq!(&on_disk(&root), app.session().document());
    // Held, the thread writes nothing while five steps come; then it writes the newest alone.
    app.autosave().unwrap().hold(true);
    for _ in 0..5 {
        ok(&mut app, "add-slide");
    }
    app.autosave().unwrap().hold(false);
    app.autosave().unwrap().wait_idle();
    assert_eq!(app.autosave().unwrap().writes(), 2);
    assert_eq!(&on_disk(&root), app.session().document());
    // A step that is no edit, as a view command, writes nothing.
    ok(&mut app, "select s1");
    app.autosave().unwrap().wait_idle();
    assert_eq!(app.autosave().unwrap().writes(), 2);
    assert_eq!(
        app.state_label(),
        format!("Workspace {}, saved", root.display())
    );
}

/// The last edit before the window closes is written: dropping the window ends its autosave after
/// the write still waiting.
#[test]
fn the_last_edit_before_a_close_is_written() {
    let root = scratch("autosave-close");
    let mut app = holding(&root);
    app.autosave().unwrap().hold(true);
    ok(&mut app, "add-slide");
    let doc = app.session().document().clone();
    drop(app);
    assert_eq!(on_disk(&root), doc);
}

#[test]
fn the_guard_writes_nothing_over_an_outside_change_and_asks() {
    let root = scratch("autosave-guard");
    let ctx = Context::default();
    let mut app = holding(&root);
    ok(&mut app, "add-slide");
    app.autosave().unwrap().wait_idle();
    // Another program writes the file: a document of three slides.
    let mut outside = Document::default();
    for at in 0..3 {
        rootpoint_model::apply(&mut outside, rootpoint_model::Op::AddSlide { at }).unwrap();
    }
    let theirs = save_ron(&outside);
    std::fs::write(root.join("presentation.ron"), &theirs).unwrap();
    ok(&mut app, "add-slide");
    let mine = app.session().document().clone();
    app.autosave().unwrap().wait_idle();
    assert!(app.autosave().unwrap().stopped());
    assert_eq!(
        std::fs::read_to_string(root.join("presentation.ron")).unwrap(),
        theirs
    );
    frame(&ctx, &mut app);
    assert!(
        app.button_rect("Load the file").is_some(),
        "the banner shows"
    );
    assert_eq!(
        app.state_label(),
        format!(
            "Workspace {}, not saved: the file changed outside rootpoint",
            root.display()
        )
    );
    // "Load the file" gives the outside document, and undo the window's.
    app.load_outside_file();
    assert!(app.session().document().same_content(&outside));
    app.autosave().unwrap().wait_idle();
    assert!(!app.autosave().unwrap().stopped(), "autosave resumes");
    app.undo();
    // The undo keeps the counter the reload reached, so the content alone is compared.
    assert!(app.session().document().same_content(&mine));
    app.autosave().unwrap().wait_idle();
    assert_eq!(&on_disk(&root), app.session().document());
    // Another outside change, then "Keep mine" writes the window's document.
    std::fs::write(root.join("presentation.ron"), &theirs).unwrap();
    ok(&mut app, "add-slide");
    app.autosave().unwrap().wait_idle();
    assert!(app.autosave().unwrap().stopped());
    app.keep_mine();
    app.autosave().unwrap().wait_idle();
    assert!(!app.autosave().unwrap().stopped());
    assert_eq!(&on_disk(&root), app.session().document());
    // A file that does not load is named, and the banner stays.
    std::fs::write(root.join("presentation.ron"), b"not ron").unwrap();
    ok(&mut app, "add-slide");
    app.autosave().unwrap().wait_idle();
    app.load_outside_file();
    assert!(app.autosave().unwrap().stopped());
    assert!(
        app.reload_error()
            .is_some_and(|e| e.contains("presentation.ron")),
        "{:?}",
        app.reload_error()
    );
    frame(&ctx, &mut app);
    assert!(app.button_rect("Load the file").is_some());
    // While the writes are stopped, the window's own `save` writes nothing over the file, and a
    // close asks, since the last edits are in no file.
    assert!(matches!(
        app.run(parse_command("save").unwrap()),
        Reply::Err(_)
    ));
    assert_eq!(
        std::fs::read(root.join("presentation.ron")).unwrap(),
        b"not ron"
    );
    assert!(close(&ctx, &mut app), "the close is cancelled");
    assert!(app.close_question());
}

/// The window's own writes are no outside change: a `save` while an autosave waits, a `save` of a
/// file written by hand, a `reload` sent by a script and an `open` of the workspace held each
/// leave the guard running.
#[test]
fn the_windows_own_writes_and_reads_do_not_stop_the_guard() {
    let root = scratch("autosave-own");
    let mut app = holding(&root);
    ok(&mut app, "add-slide");
    app.autosave().unwrap().wait_idle();
    app.autosave().unwrap().hold(true);
    ok(&mut app, "add-slide");
    // Held, the `save` would wait for ever: the thread runs it once let go.
    app.autosave().unwrap().hold(false);
    ok(&mut app, "save");
    ok(&mut app, "add-slide");
    app.autosave().unwrap().wait_idle();
    assert!(!app.autosave().unwrap().stopped(), "after a save");
    assert_eq!(&on_disk(&root), app.session().document());
    // A file in other bytes than rootpoint writes, saved with no edit, then edited.
    let mut text = save_ron(&three_slides());
    text.push_str("\n\n");
    std::fs::write(root.join("presentation.ron"), &text).unwrap();
    ok(&mut app, &format!("open {}", root.display()));
    assert_eq!(
        app.session().document().slides.len(),
        3,
        "the open reads it"
    );
    ok(&mut app, "save");
    ok(&mut app, "add-slide");
    app.autosave().unwrap().wait_idle();
    assert!(
        !app.autosave().unwrap().stopped(),
        "after an open and a save"
    );
    // A script's `reload` of a file changed outside takes the file in.
    std::fs::write(root.join("presentation.ron"), save_ron(&three_slides())).unwrap();
    ok(&mut app, "reload");
    app.autosave().unwrap().wait_idle();
    assert!(!app.autosave().unwrap().stopped(), "after a reload");
    ok(&mut app, "add-slide");
    app.autosave().unwrap().wait_idle();
    assert!(!app.autosave().unwrap().stopped());
    assert_eq!(&on_disk(&root), app.session().document());
}

/// A write that fails is named by the state label, and a close then asks.
#[test]
fn a_failed_write_is_shown_and_a_close_asks() {
    let root = scratch("autosave-fails");
    let ctx = Context::default();
    let mut app = holding(&root);
    // A directory where the temporary file goes makes every write fail.
    std::fs::create_dir_all(root.join(".rootpoint/presentation.ron.new")).unwrap();
    ok(&mut app, "add-slide");
    app.autosave().unwrap().wait_idle();
    let label = app.state_label();
    assert!(
        label.contains("not saved") && label.contains("presentation.ron.new"),
        "{label}"
    );
    assert!(close(&ctx, &mut app), "the close is cancelled");
    std::fs::remove_dir(root.join(".rootpoint/presentation.ron.new")).unwrap();
    ok(&mut app, "add-slide");
    app.autosave().unwrap().wait_idle();
    assert_eq!(
        app.state_label(),
        format!("Workspace {}, saved", root.display())
    );
}

/// Frames until `done` holds, or two seconds pass.
fn frames_until(ctx: &Context, app: &mut App, done: impl Fn(&App) -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        frame(ctx, app);
        if done(app) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

fn git(root: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(root)
        .stdout(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

/// A commit in a git repository at the root decodes nothing, though an asset lies at the root; a
/// changed asset file is decoded again, `design@rootpoint-app@asset-watch`.
#[test]
fn the_watcher_decodes_a_changed_asset_and_nothing_of_git() {
    let root = scratch("watch-assets");
    let png = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/two-colours.png"
    ))
    .unwrap();
    std::fs::write(root.join("a.png"), &png).unwrap();
    let ctx = Context::default();
    let mut app = holding(&root);
    for line in [
        "add-slide",
        &format!("import {}/a.png", root.display()),
        "image n2 a3 fit",
    ] {
        ok(&mut app, line);
    }
    frame(&ctx, &mut app);
    app.autosave().unwrap().wait_idle();
    // Let the events of the setup's own writes pass.
    std::thread::sleep(Duration::from_millis(200));
    frame(&ctx, &mut app);
    let before = app.asset_decodings();
    git(&root, &["init", "-q"]);
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "x"]);
    std::thread::sleep(Duration::from_millis(300));
    for _ in 0..5 {
        frame(&ctx, &mut app);
    }
    assert_eq!(app.asset_decodings(), before, "a commit decodes nothing");
    std::fs::write(root.join("a.png"), [png.as_slice(), b"changed"].concat()).unwrap();
    assert!(
        frames_until(&ctx, &mut app, |app| app.asset_decodings() > before),
        "the changed asset is decoded again"
    );
}

/// A directory of assets removed and made again, as a checkout does, is watched again.
#[test]
fn a_directory_made_again_is_watched_again() {
    let root = scratch("watch-again");
    let png = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/two-colours.png"
    ))
    .unwrap();
    std::fs::create_dir_all(root.join("img")).unwrap();
    std::fs::write(root.join("img/a.png"), &png).unwrap();
    let ctx = Context::default();
    let mut app = holding(&root);
    for line in [
        "add-slide",
        &format!("import {}/img/a.png", root.display()),
        "image n2 a3 fit",
    ] {
        ok(&mut app, line);
    }
    frame(&ctx, &mut app);
    std::fs::remove_dir_all(root.join("img")).unwrap();
    std::fs::create_dir_all(root.join("img")).unwrap();
    std::fs::write(root.join("img/a.png"), &png).unwrap();
    // The events of the removal and of the new file pass, with no command after them.
    std::thread::sleep(Duration::from_millis(300));
    for _ in 0..3 {
        frame(&ctx, &mut app);
    }
    let before = app.asset_decodings();
    std::fs::write(
        root.join("img/a.png"),
        [png.as_slice(), b"changed"].concat(),
    )
    .unwrap();
    assert!(
        frames_until(&ctx, &mut app, |app| app.asset_decodings() > before),
        "the asset of the directory made again is decoded again"
    );
}

/// The measurement of `design@rootpoint-app@workspace-autosave`: the GUI thread's share of one
/// autosave, the clone of the document and its hand-over, as the median of 21 runs, at 50, 200 and
/// 1000 slides, beside the median of `save_ron`, which the thread runs; the bound is on the share at
/// 1000 slides. Outside the gates, since timings vary:
/// `cargo test -p rootpoint-app --release --test autosave -- --ignored --nocapture`.
#[test]
#[ignore]
fn autosave_gui_cost() {
    let root = scratch("autosave-cost");
    let mut app = holding(&root);
    ok(&mut app, "sample");
    // The sample's slides; the document holds no image's bytes, which are in the store.
    let slides: Vec<_> = app
        .session()
        .document()
        .slides
        .iter()
        .map(|s| s.id)
        .collect();
    let mut workspace = holding(&scratch("autosave-cost-ws"));
    let autosave = workspace.autosave().unwrap();
    autosave.hold(true);
    let median = |run: &mut dyn FnMut()| {
        let mut times: Vec<Duration> = (0..21)
            .map(|_| {
                let start = Instant::now();
                run();
                start.elapsed()
            })
            .collect();
        times.sort();
        times[10]
    };
    let mut share = Duration::ZERO;
    for size in [50, 200, 1000] {
        while app.session().document().slides.len() < size {
            let slide = slides[app.session().document().slides.len() % slides.len()];
            ok(&mut app, &format!("duplicate-slide {slide}"));
        }
        let doc = app.session().document().clone();
        share = median(&mut || autosave.save(doc.clone()));
        let serialize = median(&mut || drop(save_ron(&doc)));
        eprintln!(
            "autosave_gui_cost: {size} slides, GUI thread {:.3} ms, save_ron {:.3} ms, medians of 21",
            share.as_secs_f64() * 1000.0,
            serialize.as_secs_f64() * 1000.0
        );
    }
    autosave.hold(false);
    let _ = &mut workspace;
    assert!(
        share < Duration::from_millis(1),
        "the GUI thread's share passes its bound: {share:?}"
    );
}
