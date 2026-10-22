//! The editor of rootpoint, in a window. `rootpoint [--control <socket>] [presentation.ron]` opens
//! the presentation given, or an empty one. With `--control`, the window listens on a Unix domain
//! socket at that path, and runs the commands `rootpoint-ctl` sends it; the window exits with an
//! error when another window listens there. Paths are relative to the working directory, as the
//! headless runner's are.
//!
//! Built for the wasm target, it is the browser build instead: it draws in the page's canvas of
//! id `rootpoint`, opens the built-in sample, and holds its files in memory.

use rootpoint_app::App;
#[cfg(unix)]
use rootpoint_app::control::Listener;
#[cfg(not(target_arch = "wasm32"))]
use rootpoint_protocol::Files;

// Ties this binary's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

/// The browser build: the sample, in the page's canvas.
#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    let mut app = App::in_memory();
    app.open_sample();
    rootpoint_app::keep_f5();
    wasm_bindgen_futures::spawn_local(async move {
        let canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("rootpoint"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .expect("the page has a canvas of id rootpoint");
        let started = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_| Ok(Box::new(app))),
            )
            .await;
        if let Err(e) = started {
            web_sys::console::error_1(&e);
        }
    });
}

/// The native file system, paths relative to the working directory.
#[cfg(not(target_arch = "wasm32"))]
struct NativeFiles;

#[cfg(not(target_arch = "wasm32"))]
impl Files for NativeFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|e| format!("{path}: {e}"))
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        std::fs::write(path, bytes).map_err(|e| format!("{path}: {e}"))
    }

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        std::fs::exists(path).map_err(|e| format!("{path}: {e}"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let mut args = std::env::args().skip(1);
    let (mut socket, mut presentation) = (None, None);
    while let Some(arg) = args.next() {
        if arg == "--control" {
            socket = args.next();
            if socket.is_none() {
                eprintln!("--control needs the path of a socket");
                std::process::exit(2);
            }
        } else {
            presentation = Some(arg);
        }
    }
    let mut app = App::new(Box::new(NativeFiles));
    if let Some(path) = presentation {
        // A file that cannot be loaded leaves the empty presentation, and the status line says
        // why. One that loads is the file Save writes.
        app.open_file(path);
    }
    eframe::run_native(
        "rootpoint",
        eframe::NativeOptions::default(),
        Box::new(move |cc| {
            // The control socket is a Unix domain socket; elsewhere `--control` opens none.
            #[cfg(not(unix))]
            let _ = (cc, socket);
            #[cfg(unix)]
            if let Some(socket) = socket {
                // The socket's thread wakes the window for each line, through the repaint that
                // eframe lets any thread request.
                let ctx = cc.egui_ctx.clone();
                let listener = Listener::bind(socket.as_ref(), move || ctx.request_repaint())?;
                app.listen(listener);
            }
            Ok(Box::new(app))
        }),
    )
}
