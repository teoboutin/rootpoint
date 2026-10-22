//! The browser build, run natively and in a browser: the image the app hands its texture for the
//! sample's first slide is the native golden image, so the browser lays the slide out as the
//! native build does; and the actions that open a picked directory and add a picked image issue
//! the commands `load` and `asset` on the files held in memory. In a browser, run with
//! `cargo test --target wasm32-unknown-unknown -p rootpoint-app --test web --test actions`, which
//! `wasm-bindgen-test-runner` runs in a headless browser.

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test as test;

use eframe::egui::{Color32, Context, ImageData, Pos2, RawInput, Rect, TextureId, Vec2};
use rootpoint_app::App;
use rootpoint_model::{AssetSource, SlideId};
use rootpoint_protocol::{Command, Files, Reply, Session, parse_command};

/// The native golden image of the sample's first slide in presenter mode, in a 640×360 window at
/// one pixel per point. `ROOTPOINT_BLESS=1` rewrites it from a native run after a deliberate
/// change to the sample or the renderer.
const GOLDEN: &[u8] = include_bytes!("data/sample-s1.png");
const SIZE: [usize; 2] = [640, 360];

/// The image of the last frame that uploaded one, after frames with no input until one uploads
/// nothing.
fn presented_image(app: &mut App) -> ([usize; 2], Vec<Color32>) {
    let ctx = Context::default();
    let mut last = None;
    for frame in 0..6 {
        let mut input = RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(SIZE[0] as f32, SIZE[1] as f32),
            )),
            max_texture_side: Some(8192),
            time: Some(f64::from(frame) / 60.0),
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

#[test]
fn the_sample_s_first_slide_is_the_native_golden_image() {
    let mut app = App::in_memory();
    app.open_sample();
    app.present();
    assert_eq!(app.session().view().presenting, Some(SlideId(1)));
    let (size, pixels) = presented_image(&mut app);
    assert_eq!(size, SIZE);
    let bytes: Vec<u8> = pixels.iter().flat_map(|p| p.to_array()).collect();
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

/// Files held in memory for the second session, which a picked directory's files are given to.
#[derive(Default)]
struct Held(std::collections::BTreeMap<String, Vec<u8>>);

impl Files for Held {
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
}

fn ok(session: &mut Session, files: &mut Held, line: &str) {
    let reply = session.run(parse_command(line).unwrap(), files);
    assert!(matches!(reply, Reply::Ok(_)), "{line}: {reply}");
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

#[test]
fn a_picked_directory_is_loaded_from_memory() {
    let mut app = App::in_memory();
    app.open_directory(picked());
    let mut session = Session::new();
    let mut files = Held::default();
    for (path, bytes) in picked() {
        files.0.insert(path, bytes);
    }
    ok(&mut session, &mut files, "load talk/mixed.ron");
    assert_eq!(app.session().document(), session.document());
    assert_eq!(app.session().view(), session.view());
    assert!(app.status().is_empty(), "{:?}", app.status());
    // The image is held as the file names it, a path read from memory, not as embedded bytes.
    assert!(
        app.session()
            .document()
            .assets
            .values()
            .all(|a| matches!(a.source, AssetSource::Path(_)))
    );
    // And its bytes reach the render: the image is drawn, not named missing.
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

#[test]
fn a_picked_image_is_added_by_its_name_or_a_numbered_one() {
    let png = include_bytes!("data/two-colours.png").to_vec();
    let mut app = App::in_memory();
    app.open_directory(picked());
    app.add_picked_image("two-colours.png".to_owned(), png.clone());
    app.add_picked_image("two-colours.png".to_owned(), vec![0, 1, 2]);
    // The files the app holds are the picked ones under their paths below the directory, with
    // the second image beside them.
    let mut files = Held::default();
    for (path, bytes) in picked() {
        files
            .0
            .insert(path.trim_start_matches("talk/").to_owned(), bytes);
    }
    files
        .0
        .insert("two-colours-2.png".to_owned(), vec![0, 1, 2]);
    let mut session = Session::new();
    ok(&mut session, &mut files, "load mixed.ron");
    ok(&mut session, &mut files, "asset two-colours.png");
    ok(&mut session, &mut files, "asset two-colours-2.png");
    assert_eq!(app.session().document(), session.document());
    let sources: Vec<&AssetSource> = app
        .session()
        .document()
        .assets
        .values()
        .map(|a| &a.source)
        .collect();
    assert!(sources.contains(&&AssetSource::Path("two-colours-2.png".to_owned())));
    assert!(matches!(
        app.run(Command::Save {
            path: "x.ron".to_owned()
        }),
        Reply::Err(_)
    ));
}
