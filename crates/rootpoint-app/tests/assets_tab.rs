//! The Assets tab: its thumbnail fits a painter's texture and follows the document; a drag of a
//! divider with the tab shown reads no file at each frame; and a path typed in its move field is
//! sent when another asset is selected.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use eframe::egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use rootpoint_app::{App, SideTab};
use rootpoint_model::AssetId;
use rootpoint_protocol::{Files, Reply, parse_command};

/// Files held in memory, by path, which count the reads of each path.
#[derive(Clone, Default)]
struct Counted {
    files: Rc<RefCell<BTreeMap<String, Vec<u8>>>>,
    reads: Rc<RefCell<BTreeMap<String, usize>>>,
}

impl Files for Counted {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        *self.reads.borrow_mut().entry(path.to_owned()).or_default() += 1;
        self.files
            .borrow()
            .get(path)
            .cloned()
            .ok_or(format!("{path}: no such file"))
    }
    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        self.files
            .borrow_mut()
            .insert(path.to_owned(), bytes.to_vec());
        Ok(())
    }
    fn exists(&mut self, path: &str) -> Result<bool, String> {
        let dir = format!("{path}/");
        let f = self.files.borrow();
        Ok(f.contains_key(path) || f.keys().any(|k| k.starts_with(&dir)))
    }
    fn canonical(&mut self, path: &str) -> Result<String, String> {
        Ok(path.to_owned())
    }
    fn list(&mut self, dir: &str) -> Result<Vec<String>, String> {
        let below = format!("{dir}/");
        Ok(self
            .files
            .borrow()
            .keys()
            .filter_map(|k| k.strip_prefix(&below).map(str::to_owned))
            .collect())
    }
    fn rename(&mut self, from: &str, to: &str) -> Result<(), String> {
        let bytes = self
            .files
            .borrow_mut()
            .remove(from)
            .ok_or(format!("{from}: no such file"))?;
        self.files.borrow_mut().insert(to.to_owned(), bytes);
        Ok(())
    }
    fn remove(&mut self, path: &str) -> Result<(), String> {
        self.files.borrow_mut().remove(path);
        Ok(())
    }
}

fn png(w: u32, h: u32) -> Vec<u8> {
    let img = image::RgbaImage::from_pixel(w, h, image::Rgba([200, 30, 30, 255]));
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    bytes
}

fn ok(app: &mut App, line: &str) -> String {
    match app.run(parse_command(line).unwrap()) {
        Reply::Ok(text) => text,
        Reply::Err(reason) => panic!("{line}: {reason}"),
    }
}

fn frame_with(ctx: &Context, app: &mut App, events: Vec<Event>) {
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
        events,
        ..Default::default()
    };
    let mut out = ctx.run_ui(input, |ui| app.frame(ui));
    out.textures_delta.clear();
}

fn frame(ctx: &Context, app: &mut App) {
    frame_with(ctx, app, Vec::new());
}

fn click_at(ctx: &Context, app: &mut App, at: Pos2) {
    for pressed in [true, false] {
        frame_with(
            ctx,
            app,
            vec![
                Event::PointerMoved(at),
                Event::PointerButton {
                    pos: at,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
    frame(ctx, app);
}

fn click(ctx: &Context, app: &mut App, label: &str) {
    let at = app
        .button_rect(label)
        .unwrap_or_else(|| panic!("no button {label}"))
        .center();
    click_at(ctx, app, at);
}

/// The sizes of the textures egui holds under a name.
fn texture_sizes(ctx: &Context, name: &str) -> Vec<[usize; 2]> {
    ctx.tex_manager()
        .read()
        .allocated()
        .filter(|(_, m)| m.name == name)
        .map(|(_, m)| m.size)
        .collect()
}

/// An image wider than the largest texture side a painter takes gives a thumbnail shrunk to fit:
/// egui's `Context::load_texture` debug-asserts against `max_texture_side`, 2048 with no painter
/// and 8192 for egui-wgpu's device.
#[test]
fn a_thumbnail_of_a_large_image_loads_no_texture_beyond_the_painter_s_limit() {
    let files = Counted::default();
    files
        .files
        .borrow_mut()
        .insert("big.png".to_owned(), png(9000, 2));
    let mut app = App::new(Box::new(files));
    ok(&mut app, "add-slide");
    let id = ok(&mut app, "import big.png");
    let ctx = Context::default();
    frame(&ctx, &mut app);
    app.show_tab(SideTab::Assets);
    frame(&ctx, &mut app);
    click(&ctx, &mut app, &format!("asset {id}"));
    let max = ctx.input(|i| i.max_texture_side);
    for size in texture_sizes(&ctx, &format!("thumbnail {id}")) {
        assert!(size[0] <= max && size[1] <= max, "{size:?} > {max}");
    }
}

/// A new document whose asset has the id and the path of the one shown before shows its own
/// asset's thumbnail.
#[test]
fn a_new_document_shows_its_own_asset_s_thumbnail() {
    let files = Counted::default();
    files
        .files
        .borrow_mut()
        .insert("a.png".to_owned(), png(40, 20));
    files
        .files
        .borrow_mut()
        .insert("b.png".to_owned(), png(80, 10));
    let mut app = App::new(Box::new(files));
    let ctx = Context::default();
    ok(&mut app, "add-slide");
    let id = ok(&mut app, "import a.png");
    frame(&ctx, &mut app);
    app.show_tab(SideTab::Assets);
    frame(&ctx, &mut app);
    click(&ctx, &mut app, &format!("asset {id}"));
    assert_eq!(texture_sizes(&ctx, &format!("thumbnail {id}")), [[40, 20]]);
    // Another document: its asset of the same id at the same path holds other bytes.
    ok(&mut app, "new");
    ok(&mut app, "add-slide");
    let again = ok(&mut app, "import b.png a.png");
    assert_eq!(again, id);
    assert_eq!(
        app.selected_asset(),
        Some(AssetId(id.trim_start_matches('a').parse().unwrap()))
    );
    frame(&ctx, &mut app);
    frame(&ctx, &mut app);
    assert_eq!(
        texture_sizes(&ctx, &format!("thumbnail {id}")),
        [[80, 10]],
        "the thumbnail is the old document's"
    );
}

/// A divider drag runs a command at each frame; with the Assets tab shown, as with the Selection
/// tab, its frames read no asset's file.
#[test]
fn a_divider_drag_with_the_assets_tab_shown_reads_no_file_per_frame() {
    let reads_during_drag = |tab: SideTab| {
        let files = Counted::default();
        files
            .files
            .borrow_mut()
            .insert("ws/a.png".to_owned(), png(40, 20));
        let reads = files.reads.clone();
        let mut app = App::new(Box::new(files));
        let ctx = Context::default();
        ok(&mut app, "open ws");
        ok(&mut app, "add-slide");
        ok(&mut app, "import ws/a.png");
        let ids = ok(&mut app, "split n2 beside 0.5");
        let split = ids
            .split_whitespace()
            .find_map(|w| w.strip_prefix("split="))
            .unwrap_or("n2")
            .to_owned();
        app.show_tab(tab);
        frame(&ctx, &mut app);
        frame(&ctx, &mut app);
        reads.borrow_mut().clear();
        ok(&mut app, "begin");
        for i in 1..=10 {
            let line = format!("divider {split} 0 {}", 0.5 + 0.01 * i as f64);
            match app.run(parse_command(&line).unwrap()) {
                Reply::Ok(_) => {}
                Reply::Err(e) => panic!("{line}: {e} (ids {ids})"),
            }
            frame(&ctx, &mut app);
        }
        ok(&mut app, "end");
        reads.borrow().get("ws/a.png").copied().unwrap_or(0)
    };
    let selection = reads_during_drag(SideTab::Selection);
    let assets = reads_during_drag(SideTab::Assets);
    assert_eq!(
        assets, selection,
        "reads of the asset's file over 10 frames of a drag: {assets} with the Assets tab, \
         {selection} with the Selection tab"
    );
}

/// A path typed in the move field, then a click on another asset's row: the typed path is sent.
#[test]
fn a_path_typed_then_another_asset_clicked_is_not_lost() {
    let files = Counted::default();
    files
        .files
        .borrow_mut()
        .insert("cat.png".to_owned(), png(10, 10));
    let mut app = App::new(Box::new(files));
    let ctx = Context::default();
    ok(&mut app, "sample");
    let cat = ok(&mut app, "import cat.png");
    frame(&ctx, &mut app);
    app.show_tab(SideTab::Assets);
    frame(&ctx, &mut app);
    click(&ctx, &mut app, "asset a19");
    let before = app.session().document().assets[&AssetId(19)].path.clone();
    // The path field is the row below the uses' buttons.
    let use_ = app.button_rect("use n14").expect("the use's button");
    let at = Pos2::new(use_.left() + 80.0, use_.bottom() + 14.0);
    click_at(&ctx, &mut app, at);
    assert!(
        ctx.memory(|m| m.focused().is_some()),
        "the click at {at:?} focused no field"
    );
    frame_with(
        &ctx,
        &mut app,
        vec![Event::Key {
            key: eframe::egui::Key::End,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frame_with(&ctx, &mut app, vec![Event::Text("x".to_owned())]);
    frame(&ctx, &mut app);
    click(&ctx, &mut app, &format!("asset {cat}"));
    frame(&ctx, &mut app);
    let after = app.session().document().assets[&AssetId(19)].path.clone();
    let cat_path = app.session().document().assets
        [&AssetId(cat.trim_start_matches('a').parse().unwrap())]
        .path
        .clone();
    assert_ne!(
        after,
        before,
        "the typed path {before}x was not sent; {cat} is at {cat_path}; status {:?}; undo depth n/a",
        app.status()
    );
}

/// A path typed, then Enter, sends the move.
#[test]
fn control_a_path_typed_then_enter_is_sent() {
    let files = Counted::default();
    let mut app = App::new(Box::new(files));
    let ctx = Context::default();
    ok(&mut app, "sample");
    frame(&ctx, &mut app);
    app.show_tab(SideTab::Assets);
    frame(&ctx, &mut app);
    click(&ctx, &mut app, "asset a19");
    let before = app.session().document().assets[&AssetId(19)].path.clone();
    let use_ = app.button_rect("use n14").expect("the use's button");
    let at = Pos2::new(use_.left() + 80.0, use_.bottom() + 14.0);
    click_at(&ctx, &mut app, at);
    let key = |key| Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    frame_with(&ctx, &mut app, vec![key(eframe::egui::Key::End)]);
    frame_with(&ctx, &mut app, vec![Event::Text("x".to_owned())]);
    frame_with(&ctx, &mut app, vec![key(eframe::egui::Key::Enter)]);
    frame(&ctx, &mut app);
    let after = app.session().document().assets[&AssetId(19)].path.clone();
    assert_eq!(after, format!("{before}x"), "{:?}", app.status());
}
