//! The browser build, run natively and in a browser: in the browser, the image the app hands its
//! texture for the sample's first slide is the native golden image, so the browser lays the slide
//! out as the native build does; a picked directory's image is drawn; and, in a browser, F5 never reaches
//! the browser. In a browser, run with
//! `cargo test --target wasm32-unknown-unknown -p rootpoint-app --test web --test actions`, which
//! `wasm-bindgen-test-runner` runs in a headless browser.

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test as test;

use eframe::egui::{Color32, Context, ImageData, Pos2, RawInput, Rect, TextureId, Vec2};
use rootpoint_app::App;
use rootpoint_model::SlideId;

/// The native golden image of the sample's first slide at 640×360, drawn with
/// `Precision::Portable`, whose pixels do not depend on the machine's SIMD, as
/// `design@rootpoint-render@portable-precision` has golden images drawn. `ROOTPOINT_BLESS=1`
/// rewrites it from a native run after a deliberate change to the sample or the renderer.
const GOLDEN: &[u8] = include_bytes!("data/sample-s1.png");
const SIZE: [usize; 2] = [640, 360];

thread_local! {
    /// One context and its clock for every frame of a test: an app's texture belongs to the
    /// context it was made in, so a second call on the same app uploads to the same one.
    static FRAMES: (Context, std::cell::Cell<f64>) = (Context::default(), std::cell::Cell::new(0.0));
}

/// The image of the last frame that uploaded one, after frames with no input until one uploads
/// nothing.
fn presented_image(app: &mut App) -> ([usize; 2], Vec<Color32>) {
    let ctx = FRAMES.with(|(ctx, _)| ctx.clone());
    let mut last = None;
    for _ in 0..6 {
        let time = FRAMES.with(|(_, t)| {
            t.set(t.get() + 1.0 / 60.0);
            t.get()
        });
        let mut input = RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(SIZE[0] as f32, SIZE[1] as f32),
            )),
            max_texture_side: Some(8192),
            time: Some(time),
            ..Default::default()
        };
        input
            .viewports
            .entry(input.viewport_id)
            .or_default()
            .native_pixels_per_point = Some(1.0);
        let mut out = ctx.run_ui(input, |ui| app.frame(ui));
        let image = out
            .textures_delta
            .set
            .iter()
            .filter(|(id, _)| **id != TextureId::default())
            .flat_map(|(_, deltas)| deltas.iter())
            .map(|delta| match &delta.image {
                ImageData::Color(image) => (image.size, image.pixels.clone()),
            })
            .last();
        // No painter applies the texture deltas, and egui refuses to drop them unapplied.
        out.textures_delta.clear();
        match image {
            Some(image) => last = Some(image),
            None if last.is_some() => break,
            None => {}
        }
    }
    last.expect("the presenter draws the slide")
}

/// The sample's first slide at 640×360, as premultiplied RGBA: in the browser, the image the app
/// hands its texture in presenter mode, in a 640×360 window at one pixel per point; natively, the
/// golden image's own render.
fn sample_first_slide() -> Vec<u8> {
    let mut app = App::in_memory();
    app.open_sample();
    if cfg!(target_arch = "wasm32") {
        app.present();
        assert_eq!(app.session().view().presenting, Some(SlideId(1)));
        let (size, pixels) = presented_image(&mut app);
        assert_eq!(size, SIZE);
        pixels.iter().flat_map(|p| p.to_array()).collect()
    } else {
        let out = rootpoint_render::render_slide(
            app.session().document(),
            SlideId(1),
            SIZE[0] as u16,
            SIZE[1] as u16,
            &mut rootpoint_render::FontSet::new(),
            &rootpoint_render::ImageSet::new(),
            rootpoint_render::Precision::Portable,
        )
        .unwrap();
        out.pixmap.data_as_u8_slice().to_vec()
    }
}

#[test]
fn the_sample_s_first_slide_is_the_native_golden_image() {
    let bytes = sample_first_slide();
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var_os("ROOTPOINT_BLESS").is_some() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/sample-s1.png");
        image::RgbaImage::from_raw(SIZE[0] as u32, SIZE[1] as u32, bytes.clone())
            .unwrap()
            .save(&path)
            .unwrap();
    }
    let golden = image::load_from_memory(GOLDEN).unwrap().to_rgba8();
    assert_eq!(
        [golden.width() as usize, golden.height() as usize],
        SIZE,
        "the golden image's size"
    );
    let differing = golden
        .as_raw()
        .chunks(4)
        .zip(bytes.chunks(4))
        .filter(|(a, b)| a != b)
        .count();
    let largest = golden
        .as_raw()
        .iter()
        .zip(&bytes)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    assert_eq!(
        (differing, largest),
        (0, 0),
        "pixels that differ from the golden image, and the largest difference of a channel"
    );
}

/// The fixture of the app's tests and its image, as the browser's directory picker gives them,
/// under the picked directory's name.
fn picked() -> Vec<(String, Vec<u8>)> {
    vec![
        (
            "talk/presentation.ron".to_owned(),
            include_bytes!("data/mixed.ron").to_vec(),
        ),
        (
            "talk/two-colours.png".to_owned(),
            include_bytes!("data/two-colours.png").to_vec(),
        ),
    ]
}

/// The image a picked directory's presentation names is copied into the session's map at the path
/// its file names, and is drawn. `path@rootpoint-app@tests/actions.rs` checks the picker actions'
/// commands.
#[test]
fn the_image_of_a_picked_directory_is_drawn() {
    let mut app = App::in_memory();
    app.open_directory(picked());
    assert!(app.status().is_empty(), "{:?}", app.status());
    let rootpoint_protocol::Store::Memory(map) = app.session().store() else {
        panic!("a picked directory opens in memory");
    };
    assert!(
        app.session()
            .document()
            .assets
            .values()
            .all(|a| map.contains_key(&a.path))
    );
    app.present();
    presented_image(&mut app);
    assert!(app.status().is_empty(), "{:?}", app.status());
}

#[test]
fn a_directory_without_a_presentation_is_refused() {
    let mut app = App::in_memory();
    app.open_sample();
    let before = app.session().document().clone();
    app.open_directory(vec![("talk/a.png".to_owned(), vec![1])]);
    assert_eq!(app.session().document(), &before);
    assert_eq!(app.status().len(), 1, "{:?}", app.status());
}

/// A picked directory as `picked` gives it, under another name, with `png` as its image.
fn picked_with(name: &str, png: &[u8]) -> Vec<(String, Vec<u8>)> {
    picked()
        .into_iter()
        .map(|(path, bytes)| {
            let below = path.trim_start_matches("talk/");
            let bytes = if below == "two-colours.png" {
                png.to_vec()
            } else {
                bytes
            };
            (format!("{name}/{below}"), bytes)
        })
        .collect()
}

/// A second picked directory whose presentation names the same paths is drawn with its own images,
/// not the first's: each opens a map of its own, and the decoded assets are dropped with the first.
#[test]
fn a_second_directory_is_drawn_with_its_own_images() {
    let other = include_bytes!("data/sample-s1.png");
    let mut app = App::in_memory();
    app.open_directory(picked());
    app.present();
    let first = presented_image(&mut app);
    app.open_directory(picked_with("other", other));
    app.present();
    let second = presented_image(&mut app);
    let mut fresh = App::in_memory();
    fresh.open_directory(picked_with("other", other));
    fresh.present();
    let expected = presented_image(&mut fresh);
    assert!(first != expected, "the two directories' images differ");
    assert!(
        second == expected,
        "the second directory is drawn with its own image"
    );
}

/// Files held in memory that keep what is written, for a session that writes a pack.
#[derive(Default)]
struct Written(std::collections::BTreeMap<String, Vec<u8>>);

impl rootpoint_protocol::Files for Written {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        self.0
            .get(path)
            .cloned()
            .ok_or(format!("{path}: no such file"))
    }
    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        self.0.insert(path.to_owned(), bytes.to_vec());
        Ok(())
    }
    fn exists(&mut self, path: &str) -> Result<bool, String> {
        Ok(self.0.contains_key(path))
    }
    fn canonical(&mut self, path: &str) -> Result<String, String> {
        Ok(path.to_owned())
    }
}

/// The pack of a picked directory, as `open-copy` then `pack` write it.
fn pack_of(picked: Vec<(String, Vec<u8>)>) -> Vec<u8> {
    let dir = picked[0].0.split('/').next().unwrap().to_owned();
    let mut files = Written(picked.into_iter().collect());
    let mut session = rootpoint_protocol::Session::new();
    for line in [format!("open-copy {dir}"), "pack p.rpk".to_owned()] {
        let reply = session.run(
            rootpoint_protocol::parse_command(&line).unwrap(),
            &mut files,
        );
        assert!(
            matches!(reply, rootpoint_protocol::Reply::Ok(_)),
            "{line}: {reply}"
        );
    }
    files.0.remove("p.rpk").unwrap()
}

/// A second pack whose presentation names the same paths is drawn with its own images, not the
/// first's: opening a pack drops the decoded assets of the store it replaces.
#[test]
fn a_second_pack_is_drawn_with_its_own_images() {
    let other = include_bytes!("data/sample-s1.png");
    let mut app = App::in_memory();
    app.open_picked_pack("a.rpk".to_owned(), pack_of(picked()));
    app.present();
    let first = presented_image(&mut app);
    app.stop_presenting();
    app.open_picked_pack("b.rpk".to_owned(), pack_of(picked_with("other", other)));
    app.present();
    let second = presented_image(&mut app);
    let mut fresh = App::in_memory();
    fresh.open_picked_pack("b.rpk".to_owned(), pack_of(picked_with("other", other)));
    fresh.present();
    let expected = presented_image(&mut fresh);
    assert!(first != expected, "the two packs' images differ");
    assert!(
        second == expected,
        "the second pack is drawn with its own image"
    );
}

/// A directory that loads no presentation leaves the open one with its files: an edit that decodes
/// its images again, undone, draws the same image as before.
#[test]
fn a_refused_directory_keeps_the_open_presentation_s_files() {
    let mut app = App::in_memory();
    app.open_directory(picked());
    app.present();
    let before = presented_image(&mut app);
    app.stop_presenting();
    let doc = app.session().document().clone();
    app.open_directory(vec![(
        "bad/presentation.ron".to_owned(),
        b"not ron".to_vec(),
    )]);
    assert_eq!(app.status().len(), 1, "{:?}", app.status());
    assert_eq!(app.session().document(), &doc);
    // Setting the image's area to a text and undoing it changes the assets in use twice.
    app.select_node(rootpoint_model::NodeId(3));
    app.set_text_content();
    app.present();
    presented_image(&mut app);
    app.stop_presenting();
    app.undo();
    assert_eq!(app.session().document(), &doc);
    app.present();
    assert!(
        presented_image(&mut app) == before,
        "the image is drawn as before"
    );
}

/// An image imported at the path the document already names, drawn missing until then, is drawn:
/// the import writes the asset's file, and adds no asset.
#[test]
fn an_image_added_at_a_missing_asset_s_path_is_drawn() {
    let mut app = App::in_memory();
    let ron = picked()
        .into_iter()
        .filter(|(p, _)| p.ends_with(".ron"))
        .collect();
    app.open_directory(ron);
    app.present();
    presented_image(&mut app);
    assert_eq!(
        app.status().len(),
        1,
        "the image is missing: {:?}",
        app.status()
    );
    app.stop_presenting();
    let png = include_bytes!("data/two-colours.png").to_vec();
    app.add_picked_image("two-colours.png".to_owned(), png);
    app.present();
    presented_image(&mut app);
    assert!(app.status().is_empty(), "{:?}", app.status());
}

/// F5 presents and never reloads the page: the window's listener prevents its default before the
/// canvas's listener, which stops the propagation of every key as eframe 0.37.0's does.
#[cfg(target_arch = "wasm32")]
#[test]
fn f5_is_kept_from_the_browser_when_the_canvas_stops_its_propagation() {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::Closure;

    rootpoint_app::keep_f5();
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.create_element("canvas").unwrap();
    document.body().unwrap().append_child(&canvas).unwrap();
    let stop = Closure::<dyn FnMut(web_sys::Event)>::new(|e: web_sys::Event| e.stop_propagation());
    canvas
        .add_event_listener_with_callback("keydown", stop.as_ref().unchecked_ref())
        .unwrap();
    let key = |name: &str| {
        let init = web_sys::KeyboardEventInit::new();
        init.set_key(name);
        init.set_bubbles(true);
        init.set_cancelable(true);
        let event =
            web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap();
        canvas.dispatch_event(&event).unwrap();
        event.default_prevented()
    };
    assert!(key("F5"), "F5's default");
    assert!(!key("F6"), "another key's default");
    stop.forget();
}

/// A pack round-trips in the browser build: opened from its bytes, edited, saved as the pack the
/// session targets, whose bytes the page would download, and opened again, it holds the edit.
#[test]
fn a_pack_round_trips_through_its_download() {
    let mut app = App::in_memory();
    app.open_picked_pack("talk.rpk".to_owned(), pack_of(picked()));
    let reply = app.run(rootpoint_protocol::parse_command("add-slide").unwrap());
    assert!(matches!(reply, rootpoint_protocol::Reply::Ok(_)), "{reply}");
    let edited = app.session().document().clone();
    let (name, bytes) = app.pack_for_download(false).expect("the pack's bytes");
    assert_eq!(name, "talk.rpk");
    let mut again = App::in_memory();
    again.open_picked_pack(name, bytes);
    assert!(again.session().document().same_content(&edited));
    // With no pack its target, the presentation is packed under a name of its own.
    let mut fresh = App::in_memory();
    fresh.open_directory(picked());
    let (name, _) = fresh.pack_for_download(false).expect("a new pack");
    assert_eq!(name, "presentation.rpk");
}

/// The state label reads each state a presentation held in memory has: not saved, saved to its
/// pack, and the changes since.
#[test]
fn the_state_label_reads_each_state_in_memory() {
    let mut app = App::in_memory();
    app.open_directory(picked());
    assert_eq!(app.state_label(), "In memory, not saved");
    app.pack_for_download(true).expect("a pack");
    assert_eq!(app.state_label(), "In memory, saved to presentation.rpk");
    let reply = app.run(rootpoint_protocol::parse_command("add-slide").unwrap());
    assert!(matches!(reply, rootpoint_protocol::Reply::Ok(_)), "{reply}");
    assert_eq!(
        app.state_label(),
        "In memory, saved to presentation.rpk, 1 change since"
    );
}

/// The page asks before it unloads only while an edit is in no file: the listener of the window
/// prevents `beforeunload`'s default after an edit, and not before.
#[cfg(target_arch = "wasm32")]
#[test]
fn the_page_asks_before_it_unloads_only_with_an_edit() {
    let unload = || {
        let init = web_sys::EventInit::new();
        init.set_cancelable(true);
        let event = web_sys::Event::new_with_event_init_dict("beforeunload", &init).unwrap();
        web_sys::window().unwrap().dispatch_event(&event).unwrap();
        event.default_prevented()
    };
    let mut app = App::in_memory();
    app.open_directory(picked());
    assert!(!unload(), "nothing unsaved asks nothing");
    let reply = app.run(rootpoint_protocol::parse_command("add-slide").unwrap());
    assert!(matches!(reply, rootpoint_protocol::Reply::Ok(_)), "{reply}");
    assert!(unload(), "an edit asks");
    app.pack_for_download(true).expect("a pack");
    assert!(!unload(), "a saved pack asks nothing");
    drop(app);
}

/// Enter editing the sample's text area n3 in place, and type into it: the stay's group is open.
fn type_in_place(app: &mut App) {
    use rootpoint_model::NodeId;
    let doc = app.session().document();
    let view = Rect::from_min_size(Pos2::ZERO, Vec2::new(960.0, 540.0));
    let layout = rootpoint_app::hit::Layout::new(doc, SlideId(1), view, [960, 540]).unwrap();
    let p = layout.rect(NodeId(3)).unwrap().center();
    app.select_node(NodeId(3));
    app.edit_at(NodeId(3), p, &layout);
    assert_eq!(app.editing(), Some(NodeId(3)));
    app.type_str("typed and never saved");
    assert!(app.session().group_open());
}

/// Text typed in place after a pack is saved, its editing not ended, is a change in no file: the
/// state label counts it.
#[test]
fn text_typed_in_place_is_counted_as_unsaved() {
    let mut app = App::in_memory();
    app.open_sample();
    let (name, bytes) = app.pack_for_download(true).expect("a pack");
    app.open_picked_pack(name, bytes);
    assert_eq!(app.state_label(), "In memory, saved to presentation.rpk");
    type_in_place(&mut app);
    assert_eq!(
        app.state_label(),
        "In memory, saved to presentation.rpk, 1 change since"
    );
}

/// The page asks before it unloads with text typed in place, its editing not ended.
#[cfg(target_arch = "wasm32")]
#[test]
fn the_page_asks_before_it_unloads_with_text_typed_in_place() {
    let unload = || {
        let init = web_sys::EventInit::new();
        init.set_cancelable(true);
        let event = web_sys::Event::new_with_event_init_dict("beforeunload", &init).unwrap();
        web_sys::window().unwrap().dispatch_event(&event).unwrap();
        event.default_prevented()
    };
    let mut app = App::in_memory();
    app.open_sample();
    app.pack_for_download(true).expect("a pack");
    assert!(!unload(), "nothing unsaved asks nothing");
    type_in_place(&mut app);
    assert!(unload(), "text typed in place, in no file, asks");
    drop(app);
}

/// An editor whose files are held in memory holds no workspace: an unpack, or an open, is refused.
#[test]
fn an_editor_in_memory_holds_no_workspace() {
    let mut app = App::in_memory();
    app.open_sample();
    app.unpack_to("ws".to_owned());
    assert_eq!(app.session().root(), None);
    let reply = app.run(rootpoint_protocol::parse_command("open ws").unwrap());
    assert!(
        matches!(reply, rootpoint_protocol::Reply::Err(_)),
        "{reply}"
    );
}
