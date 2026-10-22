//! The editor of rootpoint, in a window. `rootpoint [--control <socket>] [presentation.ron]` opens
//! the presentation given, or an empty one. With `--control`, the window listens on a Unix domain
//! socket at that path, and runs the commands `rootpoint-ctl` sends it; the window exits with an
//! error when another window listens there. Paths are relative to the working directory, as the
//! headless runner's are.

use rootpoint_app::App;
use rootpoint_app::control::Listener;
use rootpoint_protocol::Files;

// Ties this binary's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

/// The native file system, paths relative to the working directory.
struct NativeFiles;

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
