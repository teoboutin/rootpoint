//! A native window on a workspace on disk: its saves, and what the Assets tab reads.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rootpoint_app::App;
use rootpoint_protocol::{Files, Reply, parse_command};

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

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::canonicalize(dir).unwrap()
}

fn ok(app: &mut App, line: &str) -> String {
    match app.run(parse_command(line).unwrap()) {
        Reply::Ok(text) => text,
        reply => panic!("{line}: {reply}"),
    }
}

/// A `save` that an autosaving window sends to its autosave thread resets the steps `store`
/// replies, which `design@rootpoint-protocol@store-query` counts since the last save, as the
/// headless binary does.
#[test]
fn a_save_of_an_autosaving_window_resets_the_steps_store_replies() {
    let root = scratch("save-unsaved");
    let mut app = App::native(Box::new(NativeFiles));
    ok(&mut app, &format!("open {}", root.display()));
    ok(&mut app, "add-slide");
    ok(&mut app, "save");
    assert_eq!(
        ok(&mut app, "store"),
        format!("workspace {} unsaved=0", root.display())
    );
}

/// Files held in memory, by path, as the Assets tab's tests hold them.
#[derive(Clone, Default)]
struct Held(std::rc::Rc<std::cell::RefCell<std::collections::BTreeMap<String, Vec<u8>>>>);

impl Files for Held {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        self.0
            .borrow()
            .get(path)
            .cloned()
            .ok_or(format!("{path}: no such file"))
    }
    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        self.0.borrow_mut().insert(path.to_owned(), bytes.to_vec());
        Ok(())
    }
    fn exists(&mut self, path: &str) -> Result<bool, String> {
        let dir = format!("{path}/");
        let f = self.0.borrow();
        Ok(f.contains_key(path) || f.keys().any(|k| k.starts_with(&dir)))
    }
    fn canonical(&mut self, path: &str) -> Result<String, String> {
        Ok(path.to_owned())
    }
    fn list(&mut self, dir: &str) -> Result<Vec<String>, String> {
        if self.0.borrow().contains_key(dir) {
            return Err(format!("{dir}: not a directory"));
        }
        let below = format!("{dir}/");
        Ok(self
            .0
            .borrow()
            .keys()
            .filter_map(|k| k.strip_prefix(&below).map(str::to_owned))
            .collect())
    }
    fn rename(&mut self, from: &str, to: &str) -> Result<(), String> {
        let bytes = self
            .0
            .borrow_mut()
            .remove(from)
            .ok_or(format!("{from}: no such file"))?;
        self.0.borrow_mut().insert(to.to_owned(), bytes);
        Ok(())
    }
    fn remove(&mut self, path: &str) -> Result<(), String> {
        self.0.borrow_mut().remove(path);
        Ok(())
    }
}

/// A `render` written into the workspace adds a file to the store's directory: the Assets tab
/// lists it again, so a relink offers it among the untracked files.
#[test]
fn a_file_a_command_writes_into_the_workspace_is_offered_for_a_relink() {
    let files = Held::default();
    files
        .0
        .borrow_mut()
        .insert("ws/presentation.ron".into(), Vec::new());
    let mut app = App::new(Box::new(files.clone()));
    files.0.borrow_mut().remove("ws/presentation.ron");
    ok(&mut app, "open ws");
    let slide = ok(&mut app, "add-slide");
    let slide = slide.split_whitespace().next().unwrap().to_owned();
    // An asset whose file is missing, which the tab offers to relink.
    files
        .0
        .borrow_mut()
        .insert("gone.png".into(), b"x".to_vec());
    let id = ok(&mut app, "import gone.png");
    files.0.borrow_mut().retain(|k, _| !k.ends_with("gone.png"));
    ok(&mut app, "text n2 \"x\"");
    let asset = rootpoint_model::AssetId(id.trim_start_matches('a').parse().unwrap());
    assert_eq!(app.relink_candidates(asset), Vec::<String>::new());
    ok(&mut app, &format!("render {slide} ws/shot.png"));
    assert_eq!(app.relink_candidates(asset), vec!["shot.png".to_owned()]);
}

/// The native file system, counting the reads of each path.
#[derive(Clone, Default)]
struct CountedNative(std::rc::Rc<std::cell::RefCell<std::collections::BTreeMap<String, usize>>>);

impl Files for CountedNative {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        *self.0.borrow_mut().entry(path.to_owned()).or_default() += 1;
        NativeFiles.read(path)
    }
    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        NativeFiles.write(path, bytes)
    }
    fn exists(&mut self, path: &str) -> Result<bool, String> {
        NativeFiles.exists(path)
    }
    fn canonical(&mut self, path: &str) -> Result<String, String> {
        NativeFiles.canonical(path)
    }
}

/// In the native window, with its autosave and its watcher, each step's autosave renames
/// `presentation.ron` into the root, which the watcher sees: an event for the store's own file
/// keeps the Assets tab's listing, so the tab reads an asset's file no more often than the
/// Selection tab does.
#[test]
fn a_drag_in_a_native_workspace_with_the_assets_tab_shown_reads_no_file_per_step() {
    use eframe::egui::{Context, RawInput};
    let reads_during_edits = |tab: rootpoint_app::SideTab| {
        let root = scratch(&format!("native-drag-{tab:?}"));
        let png = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../rootpoint-protocol/sample/splits.png"
        ))
        .unwrap();
        std::fs::write(root.join("cat.png"), png).unwrap();
        let files = CountedNative::default();
        let reads = files.0.clone();
        let mut app = App::native(Box::new(files));
        let ctx = Context::default();
        let frame = |app: &mut App| {
            let mut out = ctx.run_ui(RawInput::default(), |ui| app.frame(ui));
            out.textures_delta.clear();
        };
        ok(&mut app, &format!("open {}", root.display()));
        ok(&mut app, "add-slide");
        let id = ok(&mut app, &format!("import {}/cat.png", root.display()));
        ok(&mut app, &format!("image n2 {id} fit"));
        app.show_tab(tab);
        for _ in 0..3 {
            frame(&mut app);
        }
        app.autosave().unwrap().wait_idle();
        std::thread::sleep(std::time::Duration::from_millis(300));
        frame(&mut app);
        frame(&mut app);
        let cat = root.join("cat.png").display().to_string();
        reads.borrow_mut().clear();
        for _ in 0..5 {
            ok(&mut app, "add-slide");
            app.autosave().unwrap().wait_idle();
            std::thread::sleep(std::time::Duration::from_millis(150));
            frame(&mut app);
            frame(&mut app);
        }
        reads.borrow().get(&cat).copied().unwrap_or(0)
    };
    let selection = reads_during_edits(rootpoint_app::SideTab::Selection);
    let assets = reads_during_edits(rootpoint_app::SideTab::Assets);
    assert_eq!(
        assets, selection,
        "reads of the asset's file over 5 steps: {assets} with the Assets tab, {selection} with \
         the Selection tab"
    );
}
