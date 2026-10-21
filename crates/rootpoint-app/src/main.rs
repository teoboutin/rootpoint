//! The editor of rootpoint, in a window. `rootpoint [presentation.ron]` opens the presentation
//! given, or an empty one. Paths are relative to the working directory, as the headless runner's
//! are.

use rootpoint_app::App;
use rootpoint_protocol::{Command, Files};

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
    let mut app = App::new(Box::new(NativeFiles));
    if let Some(path) = std::env::args().nth(1) {
        // A file that cannot be loaded leaves the empty presentation, and the status line says
        // why.
        app.run(Command::Load { path });
    }
    eframe::run_native(
        "rootpoint",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(app))),
    )
}
