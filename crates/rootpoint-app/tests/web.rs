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
use rootpoint_model::{AssetSource, SlideId};

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
            "talk/mixed.ron".to_owned(),
            include_bytes!("data/mixed.ron").to_vec(),
        ),
        (
            "talk/two-colours.png".to_owned(),
            include_bytes!("data/two-colours.png").to_vec(),
        ),
    ]
}

/// The image a picked directory's presentation names is held as its file names it, a path read
/// from memory, and is drawn. `path@rootpoint-app@tests/actions.rs` checks the picker actions'
/// commands.
#[test]
fn the_image_of_a_picked_directory_is_drawn() {
    let mut app = App::in_memory();
    app.open_directory(picked());
    assert!(app.status().is_empty(), "{:?}", app.status());
    assert!(
        app.session()
            .document()
            .assets
            .values()
            .all(|a| matches!(a.source, AssetSource::Path(_)))
    );
    app.present();
    presented_image(&mut app);
    assert!(app.status().is_empty(), "{:?}", app.status());
}

#[test]
fn a_directory_without_one_presentation_is_refused() {
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

/// Every picked directory loads at the same directory, the root of the files held, so a second
/// one whose presentation names the same paths is drawn with its own images, not the first's.
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
    app.open_directory(vec![("bad/x.ron".to_owned(), b"not ron".to_vec())]);
    assert_eq!(app.status().len(), 1, "{:?}", app.status());
    assert_eq!(app.session().document(), &doc);
    // Clearing the image's area and undoing it changes the assets in use twice.
    app.select_node(rootpoint_model::NodeId(3));
    app.clear_content();
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
