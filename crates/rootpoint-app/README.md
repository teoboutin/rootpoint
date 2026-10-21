# rootpoint-app

The editor of rootpoint, in a window: the binary `rootpoint`. `rootpoint [presentation.ron]` opens
the presentation given, or an empty one; the button "Open…" opens another. The window shows the
shown slide, scaled uniformly to the largest size that fits and letterboxed, and, under it, what
the slide's render could not draw as the document asks: missing images, fonts not found,
characters no font has, and text not drawn.

- A click on an area selects it, and the side panel shows the path from the slide's root to the
  selection, each entry a link that selects it. Alt+↑ selects the parent, Alt+↓ a child, the one
  selected last in it, and Alt+← and Alt+→ the siblings.
- A divider is grabbed within 4 points of it, and dragged; each child keeps 40 slide units or
  more along the split's axis. The whole drag is one undo step, and Esc cancels it. A
  double-click sets a divider halfway between its neighbours.
- The slide list shows each slide: a click shows it, a drag onto another row moves it there, and
  its buttons add a slide after the shown one, and remove or duplicate the shown one.

Its library root holds the editor's state, `App`, whose `run` takes a protocol `Command`: every
action of the editor is such a command, run on one `rootpoint_protocol::Session`. `App::frame`
shows the editor for one frame in an `egui::Ui`, so a test drives it with `Context::run_ui` and
no window.
