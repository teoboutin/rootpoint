//! The editor of rootpoint, in a window. `rootpoint [--control <socket>] [<dir>]` opens the
//! directory given as a workspace, or an empty presentation in memory. With `--control`, the window
//! listens on a Unix domain socket at that path, and runs the commands `rootpoint-ctl` sends it; the
//! window exits with an error when another window listens there. Host paths are relative to the
//! working directory, as the headless runner's are.
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

    /// Writes the file, creating the directories above it, as an import at `img/cat.png` needs.
    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        if let Some(dir) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{path}: {e}"))?;
        }
        std::fs::write(path, bytes).map_err(|e| format!("{path}: {e}"))
    }

    /// Anything at the path, a symlink whose target is missing included, as `Files::exists` asks.
    fn exists(&mut self, path: &str) -> Result<bool, String> {
        match std::fs::symlink_metadata(path) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(format!("{path}: {e}")),
        }
    }

    fn canonical(&mut self, path: &str) -> Result<String, String> {
        let canonical = std::fs::canonicalize(path).map_err(|e| format!("{path}: {e}"))?;
        canonical
            .into_os_string()
            .into_string()
            .map_err(|p| format!("{} is not UTF-8", p.display()))
    }
    fn list(&mut self, dir: &str) -> Result<Vec<String>, String> {
        /// The files below `dir`, each as `below` and its path under `dir`; a symlink is listed as
        /// a file, never followed.
        fn walk(dir: &std::path::Path, below: &str, out: &mut Vec<String>) -> std::io::Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let name = entry.file_name().into_string().map_err(|name| {
                    std::io::Error::other(format!("{} is not UTF-8", name.display()))
                })?;
                let path = if below.is_empty() {
                    name
                } else {
                    format!("{below}/{name}")
                };
                if entry.file_type()?.is_dir() {
                    walk(&entry.path(), &path, out)?;
                } else {
                    out.push(path);
                }
            }
            Ok(())
        }
        let mut out = Vec::new();
        match walk(std::path::Path::new(dir), "", &mut out) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(format!("{dir}: {e}")),
        }
        out.sort();
        Ok(out)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let mut args = std::env::args().skip(1);
    let (mut socket, mut workspace) = (None, None);
    while let Some(arg) = args.next() {
        if arg == "--control" {
            socket = args.next();
            if socket.is_none() {
                eprintln!("--control needs the path of a socket");
                std::process::exit(2);
            }
        } else {
            workspace = Some(arg);
        }
    }
    let mut app = App::new(Box::new(NativeFiles));
    if let Some(dir) = workspace {
        // A directory that cannot be opened leaves the empty presentation, and the status line
        // says why. One that opens is the workspace Save writes.
        app.open_workspace(dir);
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
