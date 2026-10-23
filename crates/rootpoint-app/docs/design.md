# Design of rootpoint-app

How rootpoint-app is built, and why: one level-three heading per decision, under level-two
subjects. Design that is decided and not built is in a plan document, not here. What rootpoint-app
is for is in `register@rootpoint-app@goal`.

## The host

### The window is an eframe app that shows the renderer's image as a texture and draws no slide content itself `##gui-toolkit`

eframe hosts the editor: its panels, the slide list, the side panel and the slide view, and
presenter mode. The slide view and presenter mode show the image `render_slide` returns, rendered
with `Precision::Native` at the view's size in physical pixels and uploaded with no conversion,
since the render is premultiplied RGBA8 in sRGB, the format of
`ColorImage::from_rgba_premultiplied`: `Context::load_texture` once, then `TextureHandle::set`. A
side is at most 8192 pixels, both sides scaled by one factor beyond it, because egui-wgpu 0.37.0
asks the GPU for textures of at most 8192 pixels a side (`path@elsewhere@egui-wgpu/src/setup.rs`),
below the render crate's own limit.

It depends on `design@rootpoint-render@slide-renderer`: under the project's own renderer the host
needs only panels, a list, a texture view, input forwarding, a browser build, which
`goal@rootpoint-app@browser-window` asks, and a way for another thread to wake it, which
`goal@rootpoint-app@live-agent-edits` needs: eframe sets egui's repaint callback, natively and on
the web, so `Context::request_repaint` called from another thread wakes the window. Its rivals
lost on the toolkit survey of the first prototype, `rejected@rootpoint-app@other-gui-hosts`; if
the renderer decision is reopened toward the toolkit's own painter, iced is the host to weigh
again. The texture path is fast enough for a drag: a divider drag in a 1600×900 view took at most
5.9 ms a frame in one run and 6.05 ms in another, release build, on the owner's Intel Core Ultra 7
165H, the frame timed around `Context::run_ui`, by
`cargo test --release -p rootpoint-app --test presenter -- --ignored --nocapture`; the GPU's
upload of the texture is not in it.

### Every action of the editor is a protocol command, run on the one session `##actions-are-commands`

Each action, a click, a key, a drag, a button, issues the `Command`s of the protocol that do what
it does, through `App::run`, which runs them on the app's one `rootpoint_protocol::Session`; the
app changes the document and the view nowhere else. Each action is a function of
`path@rootpoint-app@src/actions.rs`, which issues its commands, and
`path@rootpoint-app@tests/actions.rs` checks each against the same commands on a second session,
the documents, the views and the files they write, and again after one `undo`; the take-over,
`App::take_control`, whose command needs a control socket's open group, is checked by
`path@rootpoint-app@tests/control.rs`.

So every operation the GUI offers runs headless, as `goal@rootpoint@headless-core` asks, and a
script of the same commands reproduces what the author did. The rival, the GUI changing the
document itself, `rejected@rootpoint-model@gui-mutation`, implements every edit twice.

### Each gesture of the window is one undo group, closed at its end, and a drag's also when the window loses focus `##gesture-groups`

A divider drag, a grip drag and a stay in editing a text each open a group with `begin` when they
start, after closing any group still open, so that within one frame a group closes before another
opens. The release closes a drag's group with `end`, and Esc with `cancel`; the end of a stay, by
Esc, a click outside its area, an undo, a redo or any other action, closes its group, the stay
ending first as every action ends a gesture, as the protocol's `undo` inside a group closes it
first. While a drag runs, no key but Esc acts, since a key's edit would join the drag's step. The
groups are the protocol's, `design@rootpoint-protocol@gesture-undo`, so a gesture is one undo step
in a script as in the window.

The window's loss of focus closes a drag's group, so a release never delivered leaves no group
open, and leaves a stay and its group as they are: a stay's end is always delivered, and a window
manager can take the keyboard while it resizes the window, which winit 0.30.13 reports as a loss
of focus (its X11 `xinput2_unfocused` filters no FocusOut mode); that a resize does so on the
owner's desktop is read from that source, not measured there. Closing the stay's group there
would split one stay into two undo steps. The group is open while the window is away, and no
command of the control socket joins it, since the socket's `control` ends the window's gestures
first, `design@rootpoint-app@one-controller-at-a-time`.

### The editor's marks are drawn over the slide's texture with egui, never into the slide's image `##editor-overlay`

The divider lines, the highlight of the divider under the pointer, the selection's outline and its
grip, the preview line of a split, the hint of a drop, and the caret, the selection and the
composition's underline of a text edited in place are drawn with egui's painter over the texture,
in `App::overlay`; the marks of a gesture are not drawn while the control socket holds the
authority to edit, `design@rootpoint-app@one-controller-at-a-time`. The banner of the control
socket and its button "Take control", `design@rootpoint-app@take-over-at-once`, are egui widgets, in
presenter mode over the texture and never in its image. `render_slide` draws only the slide, so the
image presenter mode shows
is the editor's, pixel for pixel at one size, which `path@rootpoint-app@tests/presenter.rs`
compares, and which `goal@rootpoint-app@faithful-slide-view` asks. The marks are lines, rectangles
and the hint's label, editor text that is no slide content, so they are not the slide text that
`rejected@rootpoint-render@toolkit-painter` keeps out of the toolkit.

## Selecting

### A click on the slide selects an area, and a split is reached by other ways than a click on the slide `##node-selection`

A click on an area sends `select`. A click lands in an area, so a click on the slide reaches no
split, and a split is what a spanning split needs, as `goal@rootpoint@free-slide-division` asks.
So the window has other ways to reach a split than a click on the slide: today a click on the node's
row of the outline, `design@rootpoint-app@tree-outline`, the slide's own row selecting none, Alt
with an arrow, `design@rootpoint-app@selection-keys`, and the "Select" of the node menu that a
secondary click on the slide opens, `design@rootpoint-app@node-menu`. A click on a divider grabs it rather than
selecting its split, since a drag starts there. The selection is the session's view, per
`design@rootpoint-protocol@view-state`.

### The side panel shows the shown slide's whole tree as an outline, one row per node `##tree-outline`

Under a row for the slide, each node has a row in depth-first order, indented by its depth: a split
with its axis, a text with its style's name and the start of its text, an image with its file's
name. The selection's row is highlighted, and the overlay outlines the node of the row under the
pointer. An outline grows downward, one row per node; the nearest rival, a tree drawn top-down like
an organization chart, needs a column per area, and a slide of 10 areas does not fit a panel of 260
points. The keys move as the outline reads, `design@rootpoint-app@selection-keys`, so a change of
the drawing reopens them. On the owner's word: the owner asked for the whole tree with the selection
highlighted, and maybe the splits told from the areas.

### A secondary click selects the node under it, then opens the node menu on that node `##node-menu`

A secondary click on an area of the slide, on a divider's band or the grip too, or on a row of the
outline, sends `select` for that node, then opens the node menu at the click: "Select", with the
node's ancestors from the slide's root down and the node itself; on an area, "Content", text, image
while the document holds an image asset, and on an image area each image asset; on a text area, "Style", the table's styles and
custom; and "Remove", except on the slide's root. Each entry runs the action of the side panel's
equivalent or of Delete, `design@rootpoint-app@actions-are-commands`, and the menu closes as soon as
the selection is no longer its node, so an entry acts on the node the menu names. A secondary click
on the letterbox, or on the slide's row of the outline, names no node and opens no menu; during a
drag of a divider, of the grip or of a text selection it does nothing. The menu is the window's own state, not a view of the
session, `goal@rootpoint-protocol@commands-drive-everything`.

The click selects first, so the menu and the side panel describe one node. The nearest rival, a menu
that leaves the selection where it was, would describe another node than the side panel does, and
its entries would act on a node that is not the selection, which no action of the window does. The
menu has no entry that splits, since the side panel's buttons and Shift with an arrow do that. The
menu works in the browser build: eframe 0.37.0 cancels the browser's own context menu on the canvas,
in `install_event_handlers` of its `path@elsewhere@src/web/events.rs`, which passes "contextmenu" to
`prevent_default_and_stop_propagation`. That is read in eframe's source, and no test checks it: the
harness of `path@rootpoint-app@tests/web.rs` runs no `WebRunner`. The browser build has the
window's actions, `goal@rootpoint-app@browser-window`, so a menu the browser's own covered would
fail it.

On the owner's word: the owner asked for a menu on a right-click of a node that selects among its
parents, removes the node, and changes its content kind and its style.

### Alt with ↑ and ↓ moves at one depth, and Alt with ← and → changes depth `##selection-keys`

Alt with ↑ and ↓ selects the sibling above and below, and nothing at the first or the last; Alt with
← the parent, and nothing at the root; Alt with → the child of the selection selected last, or its
first child, and nothing on an area. The app keeps for each split the child last selected in it,
which only chooses the node a `select` names. In the outline, `design@rootpoint-app@tree-outline`,
siblings are the rows above and below at one depth, and the parent and the children a step left and
right. On the slide, Alt with ↓ moves down in a stacked split and right in a beside split. Shipped
tools move the same way in the survey of split editing,
`path@rootpoint@docs/grounding/split-editing.md`: i3's `focus parent`, `focus child` to the child
last focused and its sibling steps, and Figma's Shift+Enter and Enter. On the owner's desktop every
one of these keys reaches the app, by the owner's report on the built editor.

The nearest rival, Alt with ↑ and ↓ moving one row of the outline,
`rejected@rootpoint-app@outline-row-keys`, makes Alt with → redundant and drops the move at one
depth. Spatial keys, `rejected@rootpoint-app@spatial-keys`, move on the slide rather than in the
tree. On the owner's word: the owner had a hard time deciding, and thought these keys could be kept
for now, as relatively consistent with the outline, though the row moves might be a bit more
natural; the owner judged them not very hard to change. Whether the keys still confuse is watched by
`tripwire@rootpoint-app@keys-confuse-with-the-outline`.

### The side panel keeps the width its edge is dragged to, whatever it shows `##sidebar-width`

The side panel starts at 260 points and is resizable by a drag of its edge; no content widens or
narrows it, so a click that changes the selection does not resize the slide view. egui 0.37.0 starts
each frame of a panel from the size its content took in the last one, wider or narrower, read in
`Panel::show_inside_dyn` of egui's `path@elsewhere@src/containers/panel.rs`, where `default_size` is
only "the initial wrapping width". So the rows wrap or cut their text to fit, since a row laid out
in `ui.horizontal` takes its whole width, as the style rows of a text area would, wider than 260
points. And the content is a scroll area the size of the panel, which neither shrinks to its content
nor grows with it, so that a row too wide to wrap scrolls instead of widening the panel.
`path@rootpoint-app@tests/panel.rs` checks the width in each state of the content, and that the
content fits. The rival, a fixed width with no drag, would give a deep tree or a long style name no
room. On the owner's word: the owner asked for a side panel of a constant width, since a width that
changes at a click resizes the slide view.

### Only the Styles tab changes a named style, and a change from the selection makes the text custom `##styles-tab-edits-styles`

The Selection tab's style row names a style of the table, or makes the text's values its own; its
other controls change only the text's own values, so a change of a value of a text naming a style
makes that text custom, with the named values and that change, `design@rootpoint-model@named-styles`.
The Styles tab is the one part of the window that sends `style set`, `style rename`, `style add` and
`style remove`, and it lists, for each style, the number of areas that name it. A change to a style
that slides out of view use is then made only from the view that lists those uses, which answers the
cost `rejected@rootpoint-model@content-by-reference` names, that editing a shared content changes
every slide that shows it. The rival, controls of the selection that change the named style the
text names, would change slides out of view with a click meant for one text. It is a reason
respected by every control of both tabs, and the Styles tab, a tab of the side panel, keeps the
panel's width, `design@rootpoint-app@sidebar-width`.

### A subtree is selected for a spanning split by moving up to its node, and no rectangle selects `##span-selection`

To split across several areas, the author selects the split that holds them, with Alt+← or its row
of the outline, and splits it. No rectangle drawn across areas selects anything. A rectangle
selection is a second way to select, and its result surprises: the smallest node holding the
rectangle can be much larger than it, and two of three children of a split are no node of their own,
so selecting them would build the grouping that `tripwire@rootpoint-model@grouping-lost-to-merge`
watches.

## Splitting and resizing

### A split is started at the selection's middle by buttons and keys, or at the pointer with Ctrl or Shift `##selection-actions`

Four buttons of the side panel, and Shift with an arrow, split the selection at its middle, the
new area left, right, above or below: `split <node> beside|stacked 0.5 before|after`. With Ctrl
held over the selection, a click splits it beside at the pointer, and with Shift, stacked; the new
area, an empty text, `design@rootpoint-model@no-empty-kind`, takes the smaller part, `after` when
the two are equal. Delete removes the selection. None of these acts while the control socket
holds the authority to edit, `design@rootpoint-app@one-controller-at-a-time`.

The pointer split saves a drag per split, and `split` takes `at` as a fraction of the node, so the
pointer maps straight onto it; both ways serve `goal@rootpoint@free-slide-division` and
`goal@rootpoint-app@direct-manipulation`. On the owner's word: the owner chose both ways over
either alone. The owner's first idea was the pointer split alone; in the owner's opinion the
design is better with the buttons added, which look easy to use, while the pointer split allows
faster actions once its keys are mastered.

### A split at the pointer shows its line before it commits, and Alt snaps the line `##split-preview`

While Ctrl or Shift is held over the selection, a line follows the pointer where the split would
go; releasing the key, or Esc, removes it and issues nothing. With Alt also held, the line snaps to
the nearest twelfth of the node, or to a divider across the same axis elsewhere on the slide within
8 points, the nearer of the two; a divider drag snaps the same way. Snapping to other dividers lines
up rows across columns, the need `issue@rootpoint-model@linked-dividers` defers, without linking
them, and the twelfths give the usual fractions in one gesture, as Blender, in the survey of split
editing, snaps a pointer split to twelfths. On the owner's word: the owner chose the preview with
the snapping over the preview alone.

### A divider's grab band and a line's snap band are measured in points `##divider-drag`

A press within 4 points of a divider grabs it, the nearer of two within 4 points, and the cursor
is a resize cursor there; a line held with Alt snaps to a divider within 8 points. The drag keeps
the press's offset from the line and sends nothing until the pointer moves, so a click leaves no
step. Both bands are in points, so they are the same under the pointer at every size of the view:
a band in slide units would narrow with the view, and at a quarter of the slide's size a band of 4
slide units is one point. While the control socket holds the authority to edit, a press grabs no
divider and the cursor stays as it is, `design@rootpoint-app@one-controller-at-a-time`.

The survey of split editing found shipped grab bands from 4 to 10 pixels wide in all: VS Code 4,
Dear ImGui a 2-pixel separator with 4 of padding, egui_tiles 6, iced's example 10. The band here,
4 points on each side of the line, is 8 points wide, inside that range; within it the value is a
default that no fact decides.

### Every split, divider drag and move keeps each part of a split at 40 slide units or more `##min-part`

A divider drag stops where a child of the split would be under 40 slide units along its axis; a
split is refused, its button disabled and its click ignored, where a part would be under 40; a
grip drop on a move zone whose split would leave a part under 40 takes no target. So every divider
keeps room to move, while a part under the bound would leave a divider that no drag can move,
since the drag's stops would cross. The value is a default: the survey of split editing found
minimum sizes in Blender, 29 pixels times the interface scale, and egui_tiles, 32.

## Removing

### No action of the window that discards content asks a confirmation, since undo restores what it took `##no-discard-confirmation`

Delete, the Selection tab's Remove, a slide's removal and a change of a text to an image act at
once.
Each is an operation whose inverse restores what it discarded, `Op::Remove` with a snapshot of the
slide's former tree, per `design@rootpoint-model@edit-operation-core`, so one `undo` brings the
content back. The rival, a dialog before an action that discards content,
`rejected@rootpoint-app@removal-guard`, costs a click on every removal the author meant. On the
owner's word: the owner chose undo over a dialog to keep content from being lost by mistake. Undo
does not reach content whose history is gone, after a closed window or a reloaded page,
`issue@rootpoint-app@page-close-loses-edits`, nor a step a later edit cleared from the redo stack,
nor a step the author cannot find, `issue@rootpoint-protocol@history-browsing`.

## Moving content

### An area is swapped with another, or moved to its edge, by dragging its grip, with a hint of the drop's effect `##swap-gesture`

The selected area shows a grip, a square of 12 points set 8 points inside its top-left corner, so
no divider's band reaches it. The area whose text is edited shows no grip and its corner takes no
grip press, since the grip covers the first characters of its text, where a press while editing
places the caret and a drag selects, as the next paragraph keeps; Esc leaves editing and gives the
grip back. While the control socket holds the authority to edit, no area shows a grip,
`design@rootpoint-app@one-controller-at-a-time`. A drag of the grip runs in a group, per
`design@rootpoint-app@gesture-groups`. Over
another area, its middle third, by width and by height, swaps: the drop sends `swap`. The rest
moves the dragged area to the edge nearest the pointer, the distances taken as fractions of the
target's width and height, left and right winning a tie: the drop splits the target on that edge at
its middle, swaps the dragged area with the new one and removes the dragged one. The zone under the
pointer is tinted, with a label, "swap" or "move left", "move right", "move up", "move down". A
drop on the dragged area itself, or outside any area, cancels.

A plain drag inside an area is not the gesture, since a drag inside a text area is kept for
selecting its text, which editing text in place, `design@rootpoint-app@edit-in-place`, needs. A
swap button followed by a click on the target, which no fact defeats, adds a mode and costs the
same two actions. The edge move replaces a removal, a split and a new edit of the content with one
gesture, as `goal@rootpoint-app@direct-manipulation` asks. In the survey of split editing, iced
swaps on a centre drop and splits on an edge drop; i3 moves on a centre drop, swaps with a modifier,
and inserts on an edge drop; Blender swaps by a drag from a corner, and docks by an edge drop; and
the tools highlight the target. On the owner's word: the owner accepted the edge moves on the
condition that
a preview or a hint shows what a drop will do, judging that a preview of the exact result might not
be meaningful here.

## Editing text

### An area's text is edited on the slide, with Parley's `Selection` on the layout the render draws `##edit-in-place`

The caret and the selection of a text being edited are Parley's `Selection`, placed and moved by
its methods on the layout `rootpoint_render::area_text` gives,
`design@rootpoint-render@area-text-layout`,
which is the layout `render_slide` draws. Every change of the text, a typed string, a deletion, a
cut, a paste, an IME composition and its commit, is one `text` command with the area's whole text,
per `design@rootpoint-app@actions-are-commands`; the render lays the new text out, and the caret is
placed again on it by byte index. The composing text
stands in the area's text, by commands, so the renderer draws it where it will stand; the overlay
draws the caret, the selection and the composition's underline, per
`design@rootpoint-app@editor-overlay`. While editing, the app sets `PlatformOutput::ime` every
frame, since egui-winit 0.37.0 allows IME input only in a frame that sets it.

So the caret sits on the drawn glyphs at every size of the view, as
`goal@rootpoint-render@same-slide-everywhere` asks of every display: a test over every character
of the sample's area n3 puts it on the drawn glyph at three window sizes, within half a pixel, a
tolerance for rounding that the session chose, by `cargo test -p rootpoint-app --test editing`. A
toolkit text field over the slide would break lines differently and let the caret drift, as
`rejected@rootpoint-render@toolkit-painter` says, and Parley's `PlainEditor` lays its own copy of
the text out, `rejected@rootpoint-app@plain-editor`.

### A first click on a text area selects it, and a click on the selected text area edits it `##edit-entry`

A click on an area selects it. A click on the selected text area, so a double-click on any text
area, enters editing with the caret at the click, wherever no other gesture takes the press: a
press on the grip, `design@rootpoint-app@swap-gesture`, or on a divider's band keeps its own
gesture and never enters editing; Esc leaves it, the area still selected, and a press outside the
area leaves it and acts as it does outside editing: a click on another area selects that area.
Outside editing, the node keys, Shift with an arrow, Delete and Alt with an arrow, act on a
selected text area, per `design@rootpoint-app@selection-actions` and
`design@rootpoint-app@selection-keys`; while editing, the keys go to the text, where Shift with an
arrow selects. While the control socket holds the authority to edit, a click on the selected text
area only selects it, `design@rootpoint-app@one-controller-at-a-time`. Every area that holds no
image holds a text, a new area an empty one, per `design@rootpoint-model@no-empty-kind`, so a
second click enters editing on a new area too, and typing into it needs no step that makes it a
text first.

On the owner's word: the owner chose this shape over a single click that selects a text area and
edits it at once, which saves a click before typing in an area not yet selected, but leaves the
node keys to act on a text area only after Esc and makes every click on a text area place a caret;
the owner thinks it is quite intuitive to use. Also on the owner's word: a click enters editing
only at a place that serves no other gesture, so the grip and the dividers stay usable. And: a
press outside the edited area acts as it does outside editing, over a press that only leaves
editing with the edited area still selected, which would cost a second click to select another
area.

### The caret and the selection of a text being edited are the editor's state, and no command places or reads them `##caret-editor-state`

While a text is edited, its caret, its selection and its composing range live in the app, as a
drag's position does, and the protocol has no command that places or reads them; every change of
the text they lead to is a `text` command, per `design@rootpoint-app@edit-in-place`. On the
owner's word, they are not among the views `goal@rootpoint-protocol@commands-drive-everything`
makes commands of: the owner chose editor state alone over a protocol command for the caret and
the selection, `rejected@rootpoint-app@caret-command`, and approved the same for the composing
range.

### An empty preedit with no composition open changes nothing `##empty-preedit`

While a text is edited, an IME preedit with an empty text, when no composition is open, leaves the
text and the selection as they are. winit 0.30.13's X11 backend sends such a preedit when a
composition starts, read from its source,
`path@elsewhere@src/platform_impl/linux/x11/event_processor.rs`, not measured on the owner's
desktop; applied as a composition, it would replace the selection with nothing and so delete a
selected text before the first composed character. An empty preedit while a composition is open
ends it, removing the composing text.

## The control socket

### The window runs the lines of its control socket on its own thread, at the start of each frame `##control-socket`

With `--control <path>`, the window listens on a Unix domain socket at that path,
`rootpoint_app::control::Listener`. Each connection has a thread of its own, which reads one line,
queues it on a channel to the GUI's thread, calls the window's wake-up, and waits for the reply
before it reads the next; `App::frame` takes the lines queued when it starts, at most one per
connection, so a batch runs one line a frame, and only the GUI's thread touches the session. Every
command runs through `App::run` as the window's own do, `design@rootpoint-app@actions-are-commands`,
but the queries, which change nothing. The wake-up is `Context::request_repaint`, which eframe lets
another thread call, `design@rootpoint-app@gui-toolkit`, so an idle window draws a frame for a
line. The lines are the protocol's, `design@rootpoint-protocol@protocol-transport`, and its rule
for one line, `read_line`, is the one `rootpoint-headless` follows; a line with no command gets no
reply. A connection carries any number of lines, a batch, `design@rootpoint-app@socket-batch`. The
listener is built on Unix only: the browser build has no socket, and its window holds the
authority to edit always.

So an agent drives the running window as `goal@rootpoint-app@live-agent-edits` asks, and the
session stays the one state, as `goal@rootpoint@headless-core` asks of every operation. The lines
go to the session and not the session to the lines, since eframe gives the app, which owns the
session, to the GUI's thread alone, as `&mut self` of `eframe::App::ui`.

### The window replaces only a socket nothing answers, and refuses any other file at its path `##only-a-stale-socket-is-replaced`

At start, a socket at the path that accepts a connection is another window's, and the window
refuses to start; a socket that refuses connections is left by a run that ended, as a run killed
by a signal leaves it, and is removed before the bind; a socket that fails to connect for another
reason, and any file that is no socket, is refused and kept. The window removes the socket's file
when it closes, if it is still the one it bound, by its inode. The rival, removing whatever refuses
a connection, deleted the presentation an author gave as the socket's path, since a regular file
refuses a connection too. The window's bind is not atomic against another window that starts at
the same moment on the same path, `issue@rootpoint-app@two-windows-bind-one-path`.

### The window and its control socket are two controllers, and one holds the authority to edit at a time `##one-controller-at-a-time`

The window holds the authority to edit at start, so a window just opened is the author's,
`rejected@rootpoint-app@socket-holds-control-at-start`; the socket asks for it with `control`,
`design@rootpoint-app@control-passing`, and the window takes it back by its button,
`design@rootpoint-app@take-over-at-once`. The socket counts as one controller, every connection
together: an authority held by a connection would end with it, at every command a client sends on a
connection of its own, as `rootpoint-ctl <socket> <words>…` does, and between two batches,
`rejected@rootpoint-app@control-per-connection`. While the socket holds it, every action of
`path@rootpoint-app@src/actions.rs` that changes the document returns at once, `App::edit_at` with
them, the side panel disables its controls that send one, and the window draws no mark of a gesture:
no divider highlight, no resize cursor, no grip, no preview line, no drop line in the slide list.
While the window holds it, every command of the socket but the queries, `control` and `release`
replies `err the window has control`, or the refusal of a take-over.

Two exceptions keep looking free. The socket's queries, `tree`, `view`, `styles` and `render`, need
no authority, since they apply no operation and join no group. The window's inputs that change only
the view need none either: a click selects, and never enters editing; Alt with an arrow, a row of
the outline and the outline of its node under the pointer, the node menu's "Select", whose other
entries are disabled, a row of the slide list, presenting and presenter mode's keys; and the
window's own state, the side panel's tab, a style selected in the list of the Styles tab and the
node menu itself. The view stays the session's one view,
`design@rootpoint-protocol@view-state`: an author inspects any slide while the agent edits another,
since an editing command names its nodes by id and reads no view,
`design@rootpoint-protocol@protocol-addressing`. The socket's view commands still need the
authority, since a `select` of another node ends a stay of the window, its area no longer the
selection.

When the authority passes, the side that loses it has its gestures or its group closed, so no
command of one controller joins the other's undo group, and
`design@rootpoint-protocol@gesture-undo` stands as it is; one lock over the whole document needs
no reconciling of two edits at one place, which `goal@rootpoint@minimal-content-set` keeps out of
scope. The rival, a group per client in the session, `rejected@rootpoint-protocol@group-per-client`,
reopens that head and orders the undo stack out of the order of the edits.

On the owner's word: the owner proposed the two controllers, one with the authority to edit at a
time, each able to ask the other for it, for an agent that makes the changes the author asks for
while the author watches and can interrupt, which `goal@rootpoint-app@live-agent-edits` states; and
asked whether the window's inputs that only change the view could stay usable, so the author
inspects the slides while the agent edits, which the owner approved.

### The window takes the authority back at once, by a button, closing the socket's open group with `end` `##take-over-at-once`

The button "Take control" takes the authority back at once: an interrupt that needed the agent's
consent would not be one, and `goal@rootpoint-app@live-agent-edits` asks that the author interrupt
the agent at any moment, by hand, and take the work over with no typed command. It closes with
`end` a group the socket left open, so its edits are one undo step that one undo reverts, where
`cancel` would erase what the author watched happen. It sits in a banner over the toolbar, and in
presenter mode in a corner, over the slide's texture and never in its image,
`design@rootpoint-app@editor-overlay`. Esc takes no authority: in presenter mode on the owner's
word, below; in the editor it keeps its one meaning there too, leaving.
After a take-over, each edit of the socket replies
`err the author took control: stop, and wait for the author's instructions`, until its next
`control`, and the refusal ends its connection, `design@rootpoint-app@socket-batch`.

On the owner's word: the owner ruled out Esc taking control in presenter mode and proposed the
corner button in its place, since the owner might want to leave presenter mode without taking
control, and judged that a real talk would not happen
under an agent's control, so the button costs a talk nothing.

### The socket's request for the authority is granted at once, also right after a take-over `##control-passing`

The socket's `control` is granted at once: it ends the window's gestures as any edit of the
window does, a stay with its edits one undo step, a divider drag as at its release, a grip drag
cancelled, since its edits are made only at its drop, the dialog of a style's removal closed,
removing nothing, and the node menu closed. Right after a take-over it is granted too,
and the refusal of the agent's edits, `design@rootpoint-app@take-over-at-once`, is what tells a
well-behaved agent to stop and wait. `release` gives the authority back as
`design@rootpoint-protocol@control-commands` says, while the socket holds it; while the socket
holds none, it replies `ok` and changes nothing, so the refusal after a take-over lasts, and a
group the window holds open is no concern of the socket's.

On the owner's word: the owner chose the request granted at once, judging it fits the owner's
workflow best, over a request refused until the author gives the authority back,
`rejected@rootpoint-app@request-refused-until-handed-back`, and over one granted only while the
window has no gesture open, `rejected@rootpoint-app@control-when-idle`. The owner expects to take
control to interrupt the agent, to ask a question, change the instructions or correct a mistake
early, and to adjust by hand after the agent's `release`, so a request mostly comes when the
author is in no gesture. For a request right after a take-over, the owner chose the refusal that
tells the agent to stop and wait over a button "Give control" the author would click to hand back,
`rejected@rootpoint-app@give-control-button`, judging that in principle it should be enough, since
in the owner's view agents seldom act otherwise now, and choosing to try it first.
`tripwire@rootpoint-app@control-ends-a-gesture` and `tripwire@rootpoint-app@agent-takes-control-back`
watch the two choices.

### A connection to the control socket carries a batch, which a take-over ends with one refusal `##socket-batch`

A client sends a task's edits over one connection, and the window runs them one a frame, so the
author sees each edit as it is made, as `goal@rootpoint-app@live-agent-edits` asks, and a take-over
falls between two. The refusal of a take-over, and it alone, is written once and ends the
connection, so the batch's other lines are dropped, never run; a query's own error ends nothing.
`rootpoint-ctl <socket>` with no words is such a client,
`design@rootpoint-protocol@ctl-sends-a-line-or-a-batch`.

The rival, one connection per command, `rejected@rootpoint-app@connection-per-command`, costs an
agent a round trip for each edit and answers a take-over with as many refusals as the agent's
script has lines left. On the owner's word: the owner judged that restricting an agent to one edit
at a time seems a less efficient method.

## The browser build

### The browser build opens a directory the author picks and the built-in sample, and saves nothing `##browser-load-only`

Built for the wasm target, the editor opens the built-in sample at start, and Open asks for a
directory with the browser's directory picker and loads the one `.ron` file directly in it. Its
edits stay in memory: it shows no Save button, Ctrl+S says that it saves nothing, and its files,
`MemoryFiles`, refuse every write. Saving in the browser comes with packing, which writes a
presentation into one file, `design@rootpoint-model@image-storage`; until then
`goal@rootpoint@runs-in-browser` is met but for its packed file.

On the owner's word: for the prototype, the owner chose a browser build that loads only over one
that opens a presentation through the file picker and saves it by a download, judging that it
reduces the work without losing much of the features; and proposed opening a whole directory
rather than file by file.

### A picked directory's files are held in memory by their paths, and the document keeps its `Path` assets `##picked-files-by-path`

`App::open_directory` holds the picked files in `MemoryFiles`, the session's `Files`, under their
paths below the picked directory, and loads the presentation at the root of those paths;
`App::add_picked_image` holds an image under its name in the presentation's directory. The session
reads each `Path` asset from memory as it reads one from disk, per
`design@rootpoint-protocol@session-files`, so the document is the one its file holds and a script of
the same commands reproduces it, as `goal@rootpoint@headless-core` asks of every operation. Every
picked presentation has the same directory, so a new directory's assets are decoded again rather
than taken from the cache, which is keyed on the directory and the assets' paths. The rival, each
picked image converted to `AssetSource::Embedded`, would apply an operation per asset at every open,
an undo step the author never made, and the document would no longer be its file.

### In the browser, presenter mode asks the page for full screen, and ends when the page leaves it `##page-full-screen`

eframe 0.37.0's web backend handles no `ViewportCommand::Fullscreen`: `handle_viewport_commands`
of `path@elsewhere@src/web/app_runner.rs` logs it as not implemented. So, on the wasm target,
`App::frame` asks the page's document for full screen when presenter mode starts, and leaves it
when it ends. The browser takes Esc to leave full screen, and the page may not receive the key, so
presenter mode ends when the page leaves full screen while presenting, as Esc ends it natively;
`goal@rootpoint-app@presents-full-screen` asks presenter mode to be full screen.

### In the browser, no key of the editor reaches the browser's own action `##editor-keys-kept-from-browser`

A key's default action in the browser, a reload or a step back in the history, leaves the page and
loses every edit held in memory, `design@rootpoint-app@browser-load-only`. eframe 0.37.0 prevents
the default of some keys itself, in `should_prevent_default_for_key` of
`path@elsewhere@src/web/events.rs`: every arrow key, so Alt with an arrow,
`design@rootpoint-app@selection-keys`, never goes back in the history, and Ctrl with O or S. It
never prevents F5's, which starts presenter mode, and its listener on the canvas stops the
propagation of every key, `WebOptions::should_stop_propagation` being true by default. So a
listener of the window in the capture phase prevents F5's default before the canvas has the key,
and the canvas still receives it. A listener in the bubble phase never sees the key while the
canvas has focus, `rejected@rootpoint-app@bubble-phase-f5-listener`. The test
`f5_is_kept_from_the_browser_when_the_canvas_stops_its_propagation` of
`path@rootpoint-app@tests/web.rs` checks it in a headless browser.
