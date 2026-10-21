# rootpoint-app

The editor of rootpoint, in a window: the binary `rootpoint`. `rootpoint [presentation.ron]` opens
the presentation given, or an empty one; the button "Open…" opens another. The window shows the
shown slide, scaled uniformly to the largest size that fits and letterboxed, and, under it, what
the slide's render could not draw as the document asks: missing images, fonts not found,
characters no font has, and text not drawn.

Its library root holds the editor's state, `App`, whose `run` takes a protocol `Command`: every
action of the editor is such a command, run on one `rootpoint_protocol::Session`. `App::frame`
shows the editor for one frame in an `egui::Ui`, so a test drives it with `Context::run_ui` and
no window.
