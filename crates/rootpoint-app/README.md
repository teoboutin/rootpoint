# rootpoint-app

The editor of rootpoint, in a window: the binary `rootpoint`. `rootpoint [<dir> | <file>.rpk]`
opens the directory given as a workspace, its `presentation.ron` and its assets' files, or the pack
given in memory, or an empty presentation in memory. A workspace is held by one process at a time:
another window, or `rootpoint-headless`, that opens it is refused, naming the process that holds
it, and the lock goes when the window closes or opens another store. On Unix the window listens,
from its start, on a socket for the commands of the text protocol, which `rootpoint-ctl` of the
protocol crate sends: in `$XDG_RUNTIME_DIR/rootpoint/`, named after the workspace, or
`memory-<pid>.sock` in memory, the socket following the store; a workspace's `.rootpoint/socket`
names it, so `rootpoint-ctl <dir>` reaches the window. Where `$XDG_RUNTIME_DIR` is unset, the
sockets are in `$TMPDIR/rootpoint/`, then `/tmp/rootpoint-<uid>/`, a directory of the user's alone.
A socket left by a run that was killed is replaced, a path that holds another kind of file is
refused and kept, and the window removes its socket when it closes; a window whose socket cannot
be bound runs without one, the status line saying why. The toolbar ends with the state label:
"In memory, not saved", "In memory, saved to `<file>`", with the changes not saved yet, or
"Workspace `<path>`, saved", and "not saved", with the reason, when a write of the workspace did
not happen. A workspace saves itself after every change, on a thread of its own: when
`presentation.ron` was changed by another program, the window writes nothing over it, Ctrl+S
included, and a banner offers "Load the file", which makes it the presentation, one step undo
takes back, or "Keep mine", which writes the window's. Closing a window whose changes no file
holds, in memory or in a workspace whose writes stopped or failed, asks first. The window reads an image or a font again when its file changes on disk. The window holds the
authority to edit at start; `control` gives it to the socket, `release` gives it back, and the
queries `tree`, `view`, `store`, `styles`, `assets` and `render` need none. While the socket holds control, a banner
over the toolbar says so beside a button "Take control", which takes it back; the window shows each
change as it is made, and of the actions below it keeps only those that change the view or the
window's own state: selecting by a click, Alt and the arrows, the outline or the node menu's
"Select", showing a slide of the list, presenting, the button "Preview", switching the side panel's
tab, and selecting a style in the Styles tab's list. It draws no grip, no divider highlight and no
line of a split at the pointer, and a click on the selected text area only selects it. The toolbar
opens a directory as a workspace (Ctrl+O), or a pack, a `.rpk` file, in memory ("Open pack…");
saves (Ctrl+S) a workspace to its `presentation.ron`, and a presentation held in memory to the pack
it was opened from or last written to, asking for one when there is none; writes a presentation held
in memory as a pack ("Save as pack…") or as a workspace in an empty directory ("Save as workspace…",
"Unpack to…" once a pack holds it), and a workspace as a pack ("Pack to…"); opens the built-in
sample in memory, previews, presents, and undoes (Ctrl+Z) and redoes (Ctrl+Shift+Z or Ctrl+Y). The
window shows the shown slide, scaled uniformly to the largest size that fits and
letterboxed, and, under it, what the slide's render could not draw as the document asks: missing
images, fonts not found, characters no font has, styles the document lacks, and text not drawn.

- A click on an area selects it. The side panel's outline shows the slide's whole tree, one row per
  node, indented by its depth, with the selection's row highlighted; a click on a row selects its
  node, and the node of the row under the pointer is outlined on the slide. Alt+↑ and Alt+↓ select
  the sibling above and below, Alt+← the parent and Alt+→ a child, the one selected last in it, as
  the outline reads. The side panel keeps the width its edge is dragged to, whatever it shows.
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
- The side panel's Selection tab sets the selected area's content, text or an image, the text's
  named style or "custom", its size, weight, slant, colour, alignment and font, and the image's
  fit, and adds an image asset: a file below a workspace's root is added at its path there, and any
  other file is copied into the presentation, as the protocol's `import` does. A change of a value of a text that names a style makes the text
  custom: the Selection tab changes no named style.
- A right-click on an area, or on a row of the outline, selects that node and opens the node menu
  on it: "Select" selects the node or one of its ancestors, "Content" makes an area a text or an
  image, "Style" names a text's style or makes it custom, and "Remove" removes the node, as the side
  panel's controls and Delete do. A right-click while editing a text leaves editing first. Esc, or
  any click, on an entry, elsewhere in the menu or outside it, closes it.
- The side panel's Styles tab lists the named styles, each with the number of areas that name it,
  and changes the one selected in its list: its name, its values, a new style, a copy, or its
  removal, which asks what the areas naming it take instead, a custom copy first.
- The side panel's Assets tab shows the assets as a tree of their folders, with a search and a
  filter, all, unused, missing or untracked, and the total size. The selected asset shows its size,
  its thumbnail and its uses, each a button to its node or its style; its path moves it when typed,
  and a drag of its row onto a folder's moves it there; Replace… gives its file other bytes, Remove
  asset removes it while nothing names it, a removal into another asset makes its uses name that
  one, and a missing asset is relinked to a file the presentation does not name. Import… and a file
  dropped on the window add an asset; Remove unused removes every asset nothing names. Each change
  is one undo step.
- A text is edited in place, on the slide: a click on the selected text area, or a double-click on
  any text area, puts the caret where it clicked. Typing, Backspace and Delete, the arrows with
  Ctrl for words and with Shift to select, Home and End, a drag or a Shift+click to select, Ctrl+A,
  copy, cut and paste, and IME composition and dead keys edit the text; Esc leaves editing, the
  area still selected, and a click outside the area leaves editing and acts as it does outside
  it. The edits of one stay in editing are one undo step.
- Preview, in the toolbar, shows the shown slide as it will be presented, inside the editor: no
  mark is drawn over it, no area is selected, and a press on it starts no gesture. A click on an
  area selects it and leaves the preview, a double-click acting as that one click, and Esc or the
  button leaves it too.
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

- Open (Ctrl+O) asks for a directory, with the browser's directory picker, and opens a copy of its
  `presentation.ron` in memory, its images and fonts copied from the directory's files. The browser
  reads every file of the directory, at any depth, when it is picked.
- It saves nothing: it has no Save button, and Ctrl+S says so in the status line. Its changes are
  lost when the page is closed or reloaded; F5 presents and does not reload.
- Add an image asks for a PNG or JPEG file, which is copied into the presentation under its name,
  or the name with a number when another file with other bytes has it.
- Present asks the browser for full screen; when the browser leaves full screen, as it does on
  Esc, presenter mode ends.
- It has no control socket.

## Its library

Its library root holds the editor's state, `App`, whose `run` takes a protocol `Command`: every
action of the editor is such a command, run on one `rootpoint_protocol::Session`. `App::frame`
shows the editor for one frame in an `egui::Ui`, so a test drives it with `Context::run_ui` and
no window. `App::in_memory` makes an editor whose files are held in memory, as the browser build's
are, which `App::open_directory` and `App::add_picked_image` fill, and from which `open-copy` and
`import` copy into the session.
