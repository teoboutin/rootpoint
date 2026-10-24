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

    fn canonical(&mut self, path: &str) -> Result<String, String> {
        Ok(path.to_owned())
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
        self.0
            .borrow_mut()
            .remove(path)
            .map(drop)
            .ok_or(format!("{path}: no such file"))
    }
}

/// Write the app's document as `presentation.ron` of the directory `dir`, as `save` would in a
/// workspace there.
fn write_presentation(app: &App, files: &MemoryFiles, dir: &str) {
    files.0.borrow_mut().insert(
        format!("{dir}/presentation.ron"),
        rootpoint_model::save_ron(app.session().document()).into_bytes(),
    );
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
    // The view is the window less its panels, in physical pixels: less than 960×600. The side panel
    // keeps its 260 points with nothing selected, so the view is under 300 pixels wide here.
    assert!(size[0] > 200 && size[0] < 960, "{size:?}");
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
    // The sample's presentation in a workspace that lacks its image.
    write_presentation(&app, &files, "out");
    let asset = *app.session().document().assets.keys().next().unwrap();
    ok(&mut app, "open out");
    let third = app.session().document().slides[2].id;
    ok(&mut app, &format!("select {third}"));
    let ctx = Context::default();
    frame(&ctx, &mut app);
    assert_eq!(app.status(), [format!("missing images: {asset}")]);
    // A command that fails is named too.
    assert!(matches!(
        app.run(parse_command("open-copy missing").unwrap()),
        Reply::Err(_)
    ));
    assert_eq!(app.status()[0], "missing/presentation.ron: no such file");
}

/// A file edited by hand to name an asset the table lacks, below its counter, opens: the image is
/// drawn grey, and the status names it.
#[test]
fn a_file_naming_a_missing_asset_opens_and_the_status_names_it() {
    let files = MemoryFiles::default();
    files
        .0
        .borrow_mut()
        .insert("cat.png".into(), b"cat".to_vec());
    let mut app = App::new(Box::new(files.clone()));
    for line in [
        "open deck",
        "add-slide",
        "split n2 beside 0.5",
        "import cat.png",
        "image n3 a5 fit",
        "save",
    ] {
        ok(&mut app, line);
    }
    let saved = String::from_utf8(files.0.borrow()["deck/presentation.ron"].clone()).unwrap();
    let table = "    assets: {\n        5: (\n            path: \"cat.png\",\n        ),\n    },";
    let edited = saved
        .replacen("asset: 5,", "asset: 99,", 1)
        .replacen("next_id: 6,", "next_id: 100,", 1)
        .replacen(table, "    assets: {},", 1);
    assert!(
        !edited.contains("asset: 5,") && !edited.contains("next_id: 6,") && !edited.contains(table),
        "the saved file changed shape:\n{saved}"
    );
    files
        .0
        .borrow_mut()
        .insert("deck/presentation.ron".into(), edited.into_bytes());
    ok(&mut app, "open deck");
    let ctx = Context::default();
    frame(&ctx, &mut app);
    assert_eq!(app.status(), ["missing images: a99"]);
}

/// A file whose text names a style the table lacks opens, and the status line, the outline's row of
/// the area and the reply of `render` name the area and the style; that the text draws with the
/// default values is `a_missing_style_draws_with_the_default_and_is_reported` of the protocol's
/// tests.
#[test]
fn a_file_naming_a_missing_style_opens_and_the_status_names_it() {
    let files = MemoryFiles::default();
    let mut app = App::new(Box::new(files.clone()));
    ok(&mut app, "sample");
    write_presentation(&app, &files, "deck");
    let saved = String::from_utf8(files.0.borrow()["deck/presentation.ron"].clone()).unwrap();
    let edited = saved.replacen("Named(\"Title\")", "Named(\"Heading\")", 1);
    assert_ne!(edited, saved, "the saved sample changed shape");
    files
        .0
        .borrow_mut()
        .insert("deck/presentation.ron".into(), edited.into_bytes());
    ok(&mut app, "open deck");
    let ctx = Context::default();
    frame(&ctx, &mut app);
    assert_eq!(
        app.status(),
        ["missing styles, drawn in the default style: n2 Heading"]
    );
    frame(&ctx, &mut app);
    let row = app
        .outline_rows()
        .iter()
        .find(|r| r.node == Some(rootpoint_model::NodeId(2)))
        .expect("n2 has a row");
    assert!(row.label.contains("\u{26a0} Heading"), "{}", row.label);
    match app.run(parse_command("render s1 out.png 160 90").unwrap()) {
        Reply::Ok(reply) => assert!(reply.ends_with("styles=n2:Heading"), "{reply}"),
        other => panic!("render gave {other}"),
    }
}

/// An asset whose bytes `asset-replace` changes is drawn with the new ones, and with the old ones
/// again after its undo: the decoded assets are keyed on paths, so a move of the store's files
/// drops them.
#[test]
fn a_replaced_asset_is_drawn_with_its_new_bytes_and_its_undo_with_the_old() {
    let mut files = MemoryFiles::default();
    files.0.borrow_mut().insert(
        "two-colours.png".to_owned(),
        include_bytes!("data/two-colours.png").to_vec(),
    );
    let mut app = App::new(Box::new(files.clone()));
    ok(&mut app, "sample");
    let third = app.session().document().slides[2].id;
    ok(&mut app, &format!("select {third}"));
    let ctx = Context::default();
    while !frame(&ctx, &mut app).is_empty() {}
    let asset = *app.session().document().assets.keys().next().unwrap();
    ok(&mut app, &format!("asset-replace {asset} two-colours.png"));
    let replaced = frame(&ctx, &mut app);
    let size = replaced[0].1;
    let new = expected(&app, &mut files, third, size);
    assert_eq!(replaced[0].2, new);
    ok(&mut app, "undo");
    let undone = frame(&ctx, &mut app);
    assert_eq!(undone[0].2, expected(&app, &mut files, third, size));
    assert_ne!(undone[0].2, new, "the undo draws the old bytes");
}

/// A file dropped on the window, natively, is imported by its path.
#[test]
fn a_file_dropped_on_the_window_is_imported() {
    #[derive(Debug)]
    struct Dropped(std::path::PathBuf);
    impl eframe::egui::DroppedFile for Dropped {
        fn path(&self) -> &std::path::Path {
            &self.0
        }
        fn bytes(&self) -> Result<Vec<u8>, String> {
            Err("the test reads no bytes".to_owned())
        }
    }
    let files = MemoryFiles::default();
    files.0.borrow_mut().insert(
        "dropped.png".to_owned(),
        include_bytes!("data/two-colours.png").to_vec(),
    );
    let mut app = App::new(Box::new(files.clone()));
    ok(&mut app, "sample");
    let ctx = Context::default();
    let input = RawInput {
        dropped_files: vec![std::sync::Arc::new(Dropped("dropped.png".into()))],
        ..Default::default()
    };
    let mut out = ctx.run_ui(input, |ui| app.frame(ui));
    out.textures_delta.clear();
    assert!(
        app.session()
            .document()
            .assets
            .values()
            .any(|a| a.path == "dropped.png"),
        "{:?}",
        app.status()
    );
}

/// The Assets tab's thumbnail is decoded again when an open gives the selected asset's path other
/// bytes, as the slide is.
#[test]
fn the_thumbnail_follows_an_open_that_changes_the_bytes() {
    let png = |w: u32, h: u32| {
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(image::RgbaImage::new(w, h))
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
        bytes
    };
    let files = MemoryFiles::default();
    let mut app = App::new(Box::new(files.clone()));
    ok(&mut app, "sample");
    for (dir, size) in [("a", (40, 20)), ("b", (64, 36))] {
        write_presentation(&app, &files, dir);
        files
            .0
            .borrow_mut()
            .insert(format!("{dir}/splits.png"), png(size.0, size.1));
    }
    app.show_tab(rootpoint_app::SideTab::Assets);
    app.select_asset(rootpoint_model::AssetId(19));
    let ctx = Context::default();
    let thumbnail = |uploads: &[(TextureId, [usize; 2], Vec<Color32>)], size: [usize; 2]| {
        uploads.iter().any(|u| u.1 == size)
    };
    ok(&mut app, "open a");
    assert!(thumbnail(&frame(&ctx, &mut app), [40, 20]));
    ok(&mut app, "open b");
    assert!(thumbnail(&frame(&ctx, &mut app), [64, 36]));
}
