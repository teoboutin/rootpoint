# rootpoint-app

The editor of rootpoint, in a window: the binary `rootpoint`. `rootpoint [--control <socket>]
[presentation.ron]` opens the presentation given, or an empty one. With `--control`, on Unix, the
window listens on a socket at that path for the commands of the text protocol, which
`rootpoint-ctl` of the protocol crate sends; the window exits with an error when another window
listens there, refuses a path that holds another kind of file, and removes the socket when it
closes; a socket left by a run that was killed is replaced at the next start. The window holds the
authority to edit at start; `control` gives it to the socket, `release` gives it back, and the
queries `tree`, `view` and `render` need none. While the socket holds control, a banner over the
toolbar says so beside a button "Take control", which takes it back; the window shows each change
as it is made, and of the actions below it keeps only those that change the view: selecting by a
click, Alt and the arrows or the breadcrumb, showing a slide of the list, and presenting. It
draws no grip, no divider highlight and no preview line, and a click on the selected text area
only selects it. The toolbar opens a presentation (Ctrl+O), saves it to its
file (Ctrl+S) or to another, opens the built-in sample, presents, and undoes (Ctrl+Z) and redoes
(Ctrl+Shift+Z or Ctrl+Y). A save copies beside the saved file every asset outside its directory,
and every embedded one, as the protocol's `save` does. The window shows the
shown slide, scaled uniformly to the largest size that fits and letterboxed, and, under it, what
the slide's render could not draw as the document asks: missing images, fonts not found,
characters no font has, and text not drawn.

- A click on an area selects it, and the side panel shows the path from the slide's root to the
  selection, each entry a link that selects it. Alt+↑ selects the parent, Alt+↓ a child, the one
  selected last in it, and Alt+← and Alt+→ the siblings.
- A divider is grabbed within 4 points of it, and dragged; each child keeps 40 slide units or
  more along the split's axis. The whole drag is one undo step, and Esc cancels it. A
  double-click sets a divider halfway between its neighbours.
- The side panel's buttons, or Shift and an arrow, split the selection at its middle, the new
  area on that side; Delete removes it. With Ctrl held over the selection a vertical line follows
  the pointer, with Shift a horizontal one, and a click splits there, the new area, an empty
  text, taking the smaller part; Alt snaps the line, and a divider drag, to the nearest twelfth or
  to a divider within 8 points. Esc, or releasing the key, removes the line.
- The selected area shows a grip in its top-left corner, except while its text is edited. Dragged onto the middle third of another
  area it swaps the two; dragged onto the rest it moves the area to the nearest edge of the
  target. A tint and a label show what the drop will do; the drop is one undo step.
- The side panel sets the selected area's content, text or an image, the text's size,
  weight, slant, colour, alignment and font, and the image's fit, and adds an image asset.
- A text is edited in place, on the slide: a click on the selected text area, or a double-click on
  any text area, puts the caret where it clicked. Typing, Backspace and Delete, the arrows with
  Ctrl for words and with Shift to select, Home and End, a drag or a Shift+click to select, Ctrl+A,
  copy, cut and paste, and IME composition and dead keys edit the text; Esc leaves editing, the
  area still selected, and a click outside the area leaves editing and acts as it does outside
  it. The edits of one stay in editing are one undo step.
- Present (F5) shows the shown slide full screen, alone, as the editor's slide view renders it
  with no mark over it, but for a frame in a corner, with the label "The text protocol has
  control" and the button "Take control", while the control socket holds control. →, Space and Page Down show the next slide, ← and Page Up the previous,
  and Esc returns to the editor.
- The slide list shows each slide: a click shows it, a drag onto another row moves it there, and
  its buttons add a slide after the shown one, and remove or duplicate the shown one.

## In a web browser

`cargo x web` builds the browser build into `web/` under cargo's target directory, `target/web/`
unless cargo is told another: a page, the editor built for the wasm
target and its loader, which any static file server serves, such as
`python3 -m http.server -d target/web`. It opens the built-in sample, and has the editor's actions
above, with these differences:

- Open (Ctrl+O) asks for a directory, with the browser's directory picker, and opens the one
  `.ron` file directly in it, its images and fonts read from the directory's files. The browser
  reads every file of the directory, at any depth, when it is picked.
- It saves nothing: it has no Save button, and Ctrl+S says so in the status line. Its changes are
  lost when the page is closed or reloaded; F5 presents and does not reload.
- Add an image asks for a PNG or JPEG file, which joins the presentation's directory in memory
  under its name, or the name with a number when another file with other bytes has it.
- Present asks the browser for full screen; when the browser leaves full screen, as it does on
  Esc, presenter mode ends.
- It has no control socket.

## Its library

Its library root holds the editor's state, `App`, whose `run` takes a protocol `Command`: every
action of the editor is such a command, run on one `rootpoint_protocol::Session`. `App::frame`
shows the editor for one frame in an `egui::Ui`, so a test drives it with `Context::run_ui` and
no window. `App::in_memory` makes an editor whose files are held in memory, as the browser build's
are, which `App::open_directory` and `App::add_picked_image` fill.
