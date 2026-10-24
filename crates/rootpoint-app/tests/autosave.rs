//! The native window's workspace saves itself after every undo step, on a thread of its own, and
//! writes only the newest of a burst; its guard writes nothing over a `presentation.ron` changed
//! outside rootpoint, and asks; the watcher decodes an asset again when its file changes, and
//! nothing when `.git/` does. The benchmark of AC1 times the GUI thread's share of an autosave.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::{Context, RawInput};
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
        std::fs::exists(path).map_err(|e| format!("{path}: {e}"))
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
    assert_eq!(app.session().document(), &mine);
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
    frame(&ctx, &mut app);
    assert!(app.button_rect("Load the file").is_some());
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

/// AC3 of the milestone: a commit in a git repository at the root decodes nothing, though an asset
/// lies at the root; a changed asset file is decoded again.
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

/// AC1 of the milestone: the GUI thread's share of one autosave, the clone of the document and its
/// hand-over, as the median of 21 runs at 1000 slides. Outside the gates, since timings vary:
/// `cargo test -p rootpoint-app --release --test autosave -- --ignored --nocapture`.
#[test]
#[ignore]
fn autosave_gui_cost() {
    let root = scratch("autosave-cost");
    let mut app = holding(&root);
    ok(&mut app, "sample");
    let slides: Vec<_> = app
        .session()
        .document()
        .slides
        .iter()
        .map(|s| s.id)
        .collect();
    // The sample's slides duplicated to 1000, as the measurement of #arg-16 did.
    while app.session().document().slides.len() < 1000 {
        let slide = slides[app.session().document().slides.len() % slides.len()];
        ok(&mut app, &format!("duplicate-slide {slide}"));
    }
    let doc = app.session().document().clone();
    let mut workspace = holding(&scratch("autosave-cost-ws"));
    let autosave = workspace.autosave().unwrap();
    autosave.hold(true);
    let mut times: Vec<Duration> = (0..21)
        .map(|_| {
            let start = Instant::now();
            autosave.save(doc.clone());
            start.elapsed()
        })
        .collect();
    times.sort();
    let median = times[10];
    eprintln!(
        "autosave_gui_cost: {} slides, median {:.3} ms of 21",
        doc.slides.len(),
        median.as_secs_f64() * 1000.0
    );
    autosave.hold(false);
    let _ = &mut workspace;
    assert!(median < Duration::from_millis(1), "AC1 fires: {median:?}");
}
