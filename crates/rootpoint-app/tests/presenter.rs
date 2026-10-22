//! Presenter mode, with no window: at the same pixel size its image equals the image the editor
//! hands to its texture for the same slide, so no editor mark is in the slide's image, per
//! `design@rootpoint-app@editor-overlay`; its keys move through the slides and leave it; it asks
//! for full screen. And the timing of AC2's divider drag, run by hand in a release build.

use std::time::Instant;

use eframe::egui::{
    Color32, Context, Event, ImageData, Key, Modifiers, PointerButton, Pos2, RawInput, Rect,
    TextureId, Vec2, ViewportCommand, ViewportId,
};
use rootpoint_app::App;
use rootpoint_model::{NodeId, SlideId};
use rootpoint_protocol::{Command, Files, Reply};

/// The native file system, for the fixture beside this file.
struct NativeFiles;

impl Files for NativeFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|e| format!("{path}: {e}"))
    }

    fn write(&mut self, path: &str, _: &[u8]) -> Result<(), String> {
        Err(format!("{path}: the tests write no file"))
    }

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        std::fs::exists(path).map_err(|e| format!("{path}: {e}"))
    }
}

/// What one frame gave: the image set on the slide's texture, if any, and whether it asked the
/// window for full screen, or to leave it.
struct Output {
    image: Option<([usize; 2], Vec<Color32>)>,
    fullscreen: Option<bool>,
}

/// An app and the frames it is shown in, in a window of a given size at one pixel per point.
struct Driver {
    app: App,
    ctx: Context,
    time: f64,
    size: Vec2,
}

impl Driver {
    fn new(app: App, size: Vec2) -> Self {
        Driver {
            app,
            ctx: Context::default(),
            time: 0.0,
            size,
        }
    }

    fn frame(&mut self, mut events: Vec<Event>, modifiers: Modifiers) -> Output {
        self.time += 1.0 / 60.0;
        events.insert(0, Event::ModifiersChanged(modifiers));
        let mut input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.size)),
            max_texture_side: Some(8192),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        input
            .viewports
            .entry(input.viewport_id)
            .or_default()
            .native_pixels_per_point = Some(1.0);
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(input, |ui| app.frame(ui));
        let image = out
            .textures_delta
            .set
            .iter()
            .filter(|(id, _)| **id != TextureId::default())
            .flat_map(|(_, deltas)| deltas.iter())
            .map(|d| match &d.image {
                ImageData::Color(image) => (image.size, image.pixels.clone()),
            })
            .last();
        // No painter applies the texture deltas, and egui refuses to drop them unapplied.
        out.textures_delta.clear();
        let fullscreen = out.viewport_output.get(&ViewportId::ROOT).and_then(|v| {
            v.commands.iter().find_map(|c| match c {
                ViewportCommand::Fullscreen(on) => Some(*on),
                _ => None,
            })
        });
        Output { image, fullscreen }
    }

    /// Frames with no input until one uploads nothing, and the last image uploaded.
    fn settle(&mut self) -> Option<([usize; 2], Vec<Color32>)> {
        let mut last = None;
        for _ in 0..4 {
            match self.frame(vec![], Modifiers::NONE).image {
                Some(image) => last = Some(image),
                None => return last,
            }
        }
        panic!("the view does not settle");
    }

    fn key(&mut self, key: Key) -> Output {
        let event = |pressed| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        let out = self.frame(vec![event(true)], Modifiers::NONE);
        self.frame(vec![event(false)], Modifiers::NONE);
        out
    }

    fn presenting(&self) -> Option<SlideId> {
        self.app.session().view().presenting
    }
}

fn sample() -> App {
    let mut app = App::new(Box::new(NativeFiles));
    assert!(matches!(app.run(Command::Sample), Reply::Ok(_)));
    app
}

#[test]
fn the_presenter_shows_the_image_the_editor_hands_its_texture() {
    let mut d = Driver::new(sample(), Vec2::new(1000.0, 700.0));
    d.settle();
    // A selection, so that the editor draws its marks: an outline and a grip.
    d.app.select_node(NodeId(3));
    let (size, editor) = d.settle().expect("the selection draws the slide again");
    // The presenter fills its window: a window of the editor view's size gives its image the
    // same pixel size.
    d.size = Vec2::new(size[0] as f32, size[1] as f32);
    d.app.present();
    let (presented, presenter) = d.settle().expect("the presenter draws the slide");
    assert_eq!(presented, size);
    assert_eq!(d.presenting(), Some(SlideId(1)));
    assert!(editor == presenter, "the two images differ");
}

#[test]
fn the_presenter_keys_move_through_the_slides_and_esc_leaves_full_screen() {
    let mut d = Driver::new(sample(), Vec2::new(1000.0, 700.0));
    d.settle();
    let entered = d.key(Key::F5);
    assert_eq!(d.presenting(), Some(SlideId(1)));
    assert_eq!(entered.fullscreen, Some(true));
    d.key(Key::ArrowRight);
    assert_eq!(d.presenting(), Some(SlideId(7)));
    d.key(Key::Space);
    assert_eq!(d.presenting(), Some(SlideId(13)));
    // At the last slide, the next key stays there.
    d.key(Key::PageDown);
    assert_eq!(d.presenting(), Some(SlideId(13)));
    d.key(Key::PageUp);
    assert_eq!(d.presenting(), Some(SlideId(7)));
    d.key(Key::ArrowLeft);
    assert_eq!(d.presenting(), Some(SlideId(1)));
    let left = d.key(Key::Escape);
    assert_eq!(d.presenting(), None);
    assert_eq!(left.fullscreen, Some(false));
    // F5 presents the shown slide.
    d.app.show_slide(SlideId(7));
    d.key(Key::F5);
    assert_eq!(d.presenting(), Some(SlideId(7)));
}

/// AC2's drag bound, of `milestone@plans@first-prototype`: 100 frames of a drag of one divider of
/// the fixture `path@rootpoint-app@tests/data/mixed.ron` in a 1600×900 editor view, each timed
/// with `Instant` around the whole `Context::run_ui`, which holds the app's `ui` call and its
/// `TextureHandle::set`, and egui's own work around them. The bound holds when every frame is
/// within 33 ms. Run with
/// `cargo test --release -p rootpoint-app --test presenter -- --ignored --nocapture`.
#[test]
#[ignore]
fn time_a_divider_drag_in_a_1600_by_900_view() {
    time_a_divider_drag(App::new(Box::new(NativeFiles)), "");
}

/// The same drag with a control socket given to the app and no line sent: claim 9 of
/// `spec@first-prototype@live-control`, that an idle listener costs the drag no time beyond the
/// same bound. Run with the drag above.
#[test]
#[ignore]
fn time_a_divider_drag_with_an_idle_control_socket() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("idle-socket");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let listener = rootpoint_app::control::Listener::bind(&dir.join("rp.sock"), || {}).unwrap();
    let mut app = App::new(Box::new(NativeFiles));
    app.listen(listener);
    time_a_divider_drag(app, ", idle control socket");
}

fn time_a_divider_drag(mut app: App, label: &str) {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/mixed.ron").to_owned();
    app.open_file(fixture);
    assert!(app.status().is_empty(), "{:?}", app.status());
    // Size the window so that the slide view is 1600×900 points, at one pixel per point.
    let mut d = Driver::new(app, Vec2::new(2000.0, 1000.0));
    let (size, _) = d.settle().unwrap();
    d.size += Vec2::new(1600.0 - size[0] as f32, 900.0 - size[1] as f32);
    let (size, _) = d.settle().unwrap();
    assert_eq!(size, [1600, 900]);
    assert!(d.app.status().is_empty(), "{:?}", d.app.status());
    let layout = d.app.layout().unwrap().clone();
    let divider = layout
        .dividers
        .iter()
        .max_by(|a, b| {
            (a.to - a.from)
                .length()
                .total_cmp(&(b.to - b.from).length())
        })
        .unwrap();
    let start = divider.from + (divider.to - divider.from) / 2.0;
    let across = match divider.axis {
        rootpoint_model::Axis::LeftToRight => Vec2::new(1.0, 0.0),
        rootpoint_model::Axis::TopToBottom => Vec2::new(0.0, 1.0),
    };
    let button = |pos, pressed| {
        vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            },
        ]
    };
    d.frame(button(start, true), Modifiers::NONE);
    let mut times = Vec::new();
    for i in 1..=100 {
        // A sawtooth over 98 points, 2 points a frame and a jump back of 98 points every 50
        // frames, so every frame moves the divider.
        let offset = -50.0 + 2.0 * (i % 50) as f32;
        let pos = start + across * offset;
        let t = Instant::now();
        let out = d.frame(vec![Event::PointerMoved(pos)], Modifiers::NONE);
        times.push(t.elapsed());
        assert!(out.image.is_some(), "frame {i} rendered nothing");
    }
    d.frame(button(start, false), Modifiers::NONE);
    let largest = times.iter().max().unwrap();
    println!(
        "divider drag, 1600x900 view{label}, 100 frames (ms): {:?}",
        times
            .iter()
            .map(|t| t.as_secs_f64() * 1000.0)
            .map(|ms| (ms * 10.0).round() / 10.0)
            .collect::<Vec<_>>()
    );
    println!("largest {largest:?}");
    assert!(
        largest.as_secs_f64() * 1000.0 <= 33.0,
        "the largest frame took {largest:?}"
    );
}

/// Give the socket control of the driver's app: a listener bound in a scratch directory, and
/// `control` sent from a thread while the driver shows frames.
fn socket_takes_control(d: &mut Driver, name: &str) -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("rp.sock");
    d.app
        .listen(rootpoint_app::control::Listener::bind(&path, || {}).unwrap());
    let client = path.clone();
    let handle =
        std::thread::spawn(move || rootpoint_protocol::control::send(&client, "control").unwrap());
    while !handle.is_finished() {
        d.frame(vec![], Modifiers::NONE);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(handle.join().unwrap().as_deref(), Some("ok"));
    path
}

/// Claim 8 of `spec@first-prototype@live-control`: with the socket in control, presenter mode
/// draws the editor's image, places the button "Take control" where the frame records it, and a
/// click there gives control to the window; Esc leaves presenter mode, the socket keeping control.
#[test]
fn the_take_over_button_works_in_presenter_mode_and_leaves_the_slide_as_the_editors() {
    let mut d = Driver::new(sample(), Vec2::new(1000.0, 700.0));
    d.settle();
    let (size, editor) = d.settle().unwrap_or_else(|| {
        d.app.select_node(NodeId(3));
        d.settle().unwrap()
    });
    d.size = Vec2::new(size[0] as f32, size[1] as f32);
    socket_takes_control(&mut d, "presenter-control");
    d.app.present();
    let (presented, presenter) = d.settle().expect("the presenter draws the slide");
    assert_eq!(presented, size);
    assert!(editor == presenter, "the two images differ");
    // Esc leaves presenter mode and the socket keeps control.
    d.key(Key::Escape);
    assert_eq!(d.presenting(), None);
    assert_eq!(d.app.controller(), rootpoint_app::Controller::Socket);
    d.app.present();
    d.settle();
    let button = d
        .app
        .take_control_rect()
        .expect("the button is shown")
        .center();
    let click = |pressed| Event::PointerButton {
        pos: button,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    d.frame(
        vec![Event::PointerMoved(button), click(true)],
        Modifiers::NONE,
    );
    d.frame(vec![click(false)], Modifiers::NONE);
    assert_eq!(
        d.app.controller(),
        rootpoint_app::Controller::TakenFromSocket
    );
    d.frame(vec![], Modifiers::NONE);
    assert_eq!(d.app.take_control_rect(), None);
}
