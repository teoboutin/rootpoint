//! The slide view, driven with `Context::run_ui` and no window: the texture holds the shown
//! slide's render at the view's size in pixels, and the status lines name what the render could
//! not draw.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use eframe::egui::{
    Color32, Context, ImageData, Pos2, RawInput, Rect, TextureId, TexturesDelta, Vec2,
};
use rootpoint_app::App;
use rootpoint_model::{Content, SlideId};
use rootpoint_protocol::{Command, Files, Reply, parse_command};
use rootpoint_render::{FontSet, ImageSet, Precision, render_slide};

/// Files held in memory, by path, shared with the test so that it can remove one.
#[derive(Clone, Default)]
struct MemoryFiles(Rc<RefCell<BTreeMap<String, Vec<u8>>>>);

impl Files for MemoryFiles {
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
        Ok(self.0.borrow().contains_key(path))
    }
}

fn ok(app: &mut App, line: &str) {
    let reply = app.run(parse_command(line).unwrap());
    assert!(matches!(reply, Reply::Ok(_)), "{line}: {reply}");
}

/// One frame of a window of 640×400 points at 1.5 pixels per point, and the images it set on
/// textures other than egui's font atlas.
fn frame(ctx: &Context, app: &mut App) -> Vec<(TextureId, [usize; 2], Vec<Color32>)> {
    let mut input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(640.0, 400.0))),
        max_texture_side: Some(16384),
        ..Default::default()
    };
    input
        .viewports
        .entry(input.viewport_id)
        .or_default()
        .native_pixels_per_point = Some(1.5);
    let mut delta = ctx.run_ui(input, |ui| app.frame(ui)).textures_delta;
    let images = slide_images(&delta);
    // No painter applies the deltas, and egui refuses to drop them unapplied.
    delta.clear();
    images
}

fn slide_images(delta: &TexturesDelta) -> Vec<(TextureId, [usize; 2], Vec<Color32>)> {
    delta
        .set
        .iter()
        .filter(|(id, _)| **id != TextureId::default())
        .flat_map(|(id, deltas)| deltas.iter().map(move |d| (*id, d)))
        .map(|(id, d)| match &d.image {
            ImageData::Color(image) => (id, image.size, image.pixels.clone()),
        })
        .collect()
}

/// The render of a slide, as the app must upload it: its images decoded from the session's
/// bytes, at the native precision.
fn expected(app: &App, files: &mut MemoryFiles, slide: SlideId, size: [usize; 2]) -> Vec<Color32> {
    let doc = app.session().document();
    let mut images = ImageSet::new();
    for area in doc.slide(slide).unwrap().root.areas() {
        if let Content::Image(i) = &area.content
            && let Ok(bytes) = app.session().asset_bytes(i.asset, files)
        {
            images.insert_bytes(i.asset, &bytes).unwrap();
        }
    }
    let out = render_slide(
        doc,
        slide,
        size[0] as u16,
        size[1] as u16,
        &mut FontSet::new(),
        &images,
        Precision::Native,
    )
    .unwrap();
    out.pixmap
        .data_as_u8_slice()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|&[r, g, b, a]| Color32::from_rgba_premultiplied(r, g, b, a))
        .collect()
}

#[test]
fn the_shown_slide_is_uploaded_as_rendered_and_only_when_it_changes() {
    let mut files = MemoryFiles::default();
    let mut app = App::new(Box::new(files.clone()));
    assert!(matches!(app.run(Command::Sample), Reply::Ok(_)));
    let ctx = Context::default();
    // The first frame shows the first slide, since none is shown. egui sizes its panels again on
    // the second frame, so the view's size can change once; after that, a frame with nothing
    // changed uploads nothing.
    let mut uploads = vec![frame(&ctx, &mut app)];
    while !uploads.last().unwrap().is_empty() {
        assert!(uploads.len() < 3, "the view does not settle");
        uploads.push(frame(&ctx, &mut app));
    }
    let last = &uploads[uploads.len() - 2];
    assert_eq!(last.len(), 1, "one slide texture is set");
    let (id, size, pixels) = &last[0];
    assert!(uploads.iter().flatten().all(|u| u.0 == *id));
    // The view is the window less its two panels, in physical pixels: 960 wide.
    assert_eq!(size[0], 960);
    assert!(size[1] > 400 && size[1] < 600, "{size:?}");
    let slide = app.session().document().slides[0].id;
    assert_eq!(*pixels, expected(&app, &mut files, slide, *size));
    // Showing the third slide, its image among it, sets the same texture to its render.
    let third = app.session().document().slides[2].id;
    ok(&mut app, &format!("select {third}"));
    let shown = frame(&ctx, &mut app);
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].0, *id);
    assert_eq!(shown[0].2, expected(&app, &mut files, third, *size));
    assert!(app.status().is_empty(), "{:?}", app.status());
}

#[test]
fn the_status_names_an_image_the_render_could_not_draw() {
    let files = MemoryFiles::default();
    let mut app = App::new(Box::new(files.clone()));
    ok(&mut app, "sample");
    ok(&mut app, "save out/deck.ron");
    // The sample's image is embedded, so the save writes it out under its asset's id.
    let asset = *app.session().document().assets.keys().next().unwrap();
    let image = format!("out/asset-{}.png", asset.0);
    assert!(files.0.borrow_mut().remove(&image).is_some(), "{image}");
    ok(&mut app, "load out/deck.ron");
    let third = app.session().document().slides[2].id;
    ok(&mut app, &format!("select {third}"));
    let ctx = Context::default();
    frame(&ctx, &mut app);
    assert_eq!(app.status(), [format!("missing images: {asset}")]);
    // A command that fails is named too.
    assert!(matches!(
        app.run(parse_command("load missing.ron").unwrap()),
        Reply::Err(_)
    ));
    assert_eq!(app.status()[0], "missing.ron: no such file");
}
