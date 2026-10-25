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
Esc, a click outside its area, an undo, a redo or any other action that does not act on the edited
text, closes its group, the stay ending first as every such action ends a gesture, as the
protocol's `undo` inside a group closes it first. An action of the side panel that acts on the
edited text, a style control, a span of the ancestor row or its "Split here",
`design@rootpoint-app@controls-act-at-the-caret`, joins the stay instead, its commands in the
stay's group, as a key of editing's are. Ending the stay first would leave no caret for it to act
at, and an undo step of its own would cut the stay, which this head makes one undo step, into
several. While a drag runs, no key but Esc acts, since a key's edit would join the drag's step. The
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
grip, the line of a split at the pointer, the hint of a drop, and the caret, the selection, the
composition's underline and the frames of the spans that hold the caret, of a text edited in place,
are drawn with egui's painter over the texture, in `App::overlay`; the marks of a gesture are not drawn while the control socket holds the authority to
edit, `design@rootpoint-app@one-controller-at-a-time`, and no mark is drawn in the final view, the
session's preview, `design@rootpoint-protocol@view-state`. The banner of the control socket and its
button "Take control", `design@rootpoint-app@take-over-at-once`, are egui widgets, in presenter mode
over the texture and never in its image. `render_slide` draws only the slide, so the image presenter
mode shows is the editor's, pixel for pixel at one size, in the final view with nothing painted over
it, which `path@rootpoint-app@tests/presenter.rs` compares, and which
`goal@rootpoint-app@faithful-slide-view` asks. The marks are lines, rectangles and the hint's label,
editor text that is no slide content, so they are not the slide text that
`rejected@rootpoint-render@toolkit-painter` keeps out of the toolkit.

The frames take no space in the slide, so the text an author edits stands where it is presented:
one thin rectangle per line a span covers, each outer frame lighter and further out, and around
the caret for an empty span, which covers no glyph. The caret is drawn in the colour and at the
size it types with. So the two caret positions at a span's edge, which stand at one byte index,
show different marks, a frame at one and not the other, which the test
`the_two_positions_at_an_edge_show_different_marks` of `path@rootpoint-app@tests/gestures.rs`
checks over the spans of the sample's styled text. LyX's frames reserve their space,
`rejected@rootpoint-app@reserved-frame-space`. Whether frames of no space let the author tell the
two positions of an edge apart in real use is watched by `tripwire@rootpoint-app@frames-mislead`.
On the owner's word: the owner expects to revise the frames' form once seen live, and might
consider relaxing `goal@rootpoint-app@faithful-slide-view` if a pure overlay does not work, while
believing it has good chances of working out.

### The window always shows where its presentation is and whether it is saved, and names where each writing action writes `##visible-mode`

The state label, `App::state_label`, at the right end of the toolbar, reads "In memory, not saved";
"In memory, saved to `<file>`", with ", n changes since" once there are; or, in a workspace, which
saves itself, `design@rootpoint-app@workspace-autosave`, "Workspace `<path>`, saved", "Workspace
`<path>`, not saved: the file changed outside rootpoint" while the guard of
`design@rootpoint-app@ron-overwrite-guard` stops the writes, and "Workspace `<path>`, not saved:
`<reason>`" when the last write failed. A window with no autosave, as `App::new` in the tests, reads
"n changes since the save" in place of "saved" once there are. Each action that writes is named by
what it writes: "Save as pack…", "Save as workspace…", "Unpack to…", "Pack to…"; Save writes to
the store the label names. The label shows the store `store` replies,
`design@rootpoint-protocol@store-query`. On the owner's
word: the owner set that the window conveys which state it is in, and what each of its actions
does.

## Selecting

### In the final view a press starts no gesture `##final-view-presses`

In the final view, the session's preview, a press on the slide starts no gesture: no divider or
grip drag, no split at the pointer, no editing; Ctrl or Shift shows no line of a split. A click on
an area only sends its `select`, which ends the final view, by the rule of
`design@rootpoint-protocol@final-preview`, and a secondary click then opens the node menu,
`design@rootpoint-app@node-menu`. The author did not see where a gesture would act, since no mark
is drawn there, `design@rootpoint-app@editor-overlay`. The nearest rival, a final view that hides
the marks and keeps the gestures, would split on a Ctrl-click with no line shown, and leave the
selection invisible.

### A click on the slide selects an area, and a split is reached by other ways than a click on the slide `##node-selection`

A click on an area sends `select`. A click lands in an area, so a click on the slide reaches no
split, and a split is what a spanning split needs, as `goal@rootpoint@free-slide-division` asks. So
the window has other ways to reach a split than a click on the slide: today a click on the node's
row of the outline, `design@rootpoint-app@tree-outline`, the slide's own row selecting none, Alt
with an arrow, `design@rootpoint-app@selection-keys`, and the "Select" of the node menu that a
secondary click on the slide opens, `design@rootpoint-app@node-menu`. A click on a divider grabs it
rather than selecting its split, since a drag starts there. The selection is the session's view, per
`design@rootpoint-protocol@view-state`.

### The side panel shows the shown slide's whole tree as an outline, one row per node `##tree-outline`

Under a row for the slide, each node has a row in depth-first order, indented by its depth: a split
with its axis, a text with its style's name and the start of its text, an image with its asset's
path, an equation with its number and the start of its LaTeX. The selection's row is highlighted,
and the overlay outlines the node of the row under the pointer. An outline grows downward, one row
per node; the nearest rival, a tree drawn top-down like an organization chart, needs a column per
area, and a slide of 10 areas does not fit a panel of 260 points. The keys move as the outline
reads, `design@rootpoint-app@selection-keys`, so a change of the drawing reopens them. On the
owner's word: the owner asked for the whole tree with the selection highlighted, and maybe the
splits told from the areas.

### A secondary click selects the node under it, then opens the node menu on that node `##node-menu`

A secondary click on an area of the slide, on a divider's band or the grip too, or on a row of the
outline, sends `select` for that node, then opens the node menu at the click: "Select", with the
node's ancestors from the slide's root down and the node itself; on an area, "Content", text, image
while the document holds an image asset, and on an image area each image asset, then equation; on a
text area, "Style", the table's styles and custom; and "Remove", except on the slide's root. Each
entry runs the action of the side panel's equivalent or of Delete,
`design@rootpoint-app@actions-are-commands`, and the menu closes as soon as the selection is no
longer its node, so an entry acts on the node the menu names. A secondary click on the letterbox, or
on the slide's row of the outline, names no node and opens no menu; during a drag of a divider, of
the grip or of a text selection it does nothing. The menu is the window's own state, not a view of
the session, `goal@rootpoint-protocol@commands-drive-everything`.

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

### Only the Styles tab changes a named style, and a change from the selection makes the text or the span custom `##styles-tab-edits-styles`

The Selection tab's style row names a style of the table, or makes the text's values its own; its
other controls change only the text's own values, so a change of a value of a text naming a style
makes that text custom, with the named values and that change,
`design@rootpoint-model@named-styles`. While a text is edited, its controls name a span style or set
a span's own values, `design@rootpoint-app@controls-act-at-the-caret`, so a value set on a span
naming a span style makes that span custom, `design@rootpoint-model@named-span-styles`. The Styles
tab is the one part of the window that sends `style set`, `style rename`, `style add` and
`style remove`, and their `span-style` forms, and it lists, for each style, the number of areas
that name it, and for each span style, in a section of its own, the number of spans; each value of
a span style may be set or not, `<key>=inherit`, since each is optional. A change to a style that slides out of view use is then made only from
the view that lists those uses, which answers the cost
`rejected@rootpoint-model@content-by-reference` names, that editing a shared content changes every
slide that shows it. The rival, controls of the selection that change the named style the text
names, would change slides out of view with a click meant for one text. It is a reason respected by
every control of the Selection and Styles tabs, and the Styles tab, a tab of the side panel, keeps the panel's width,
`design@rootpoint-app@sidebar-width`.

### Only the Assets tab changes an asset, beside the uses it lists `##assets-tab`

The Assets tab, a tab of the side panel, `path@rootpoint-app@src/asset_tab.rs`, is the one part of
the window that moves, relinks, replaces and removes an asset, by the commands of
`design@rootpoint-protocol@asset-commands`, each an action of
`design@rootpoint-app@actions-are-commands`; the Selection tab and the node menu choose which asset
an image shows, and change none. It shows the tree of the asset paths, its folders the prefixes the
paths have, `design@rootpoint-model@folders-from-paths`, with a search and the filters of
`assets`, and, for the selected asset, its size, its thumbnail and its uses, each a button to its
node, its style or its span style. An asset an image shows on several slides is then changed only from the view that
lists those slides, for the reason `design@rootpoint-app@styles-tab-edits-styles` gives for a
style: a change meant for one image would otherwise change slides out of view. It keeps the panel's
width, `design@rootpoint-app@sidebar-width`, its rows cut
to it. It adds no kind of slide content, so `goal@rootpoint@minimal-content-set`, which counts the
features of slide content, does not count it. On the owner's word: the owner proposed an asset
manager screen that edits the file tree, searches, renames and filters the unused assets, and asked
for any other meaningful feature, which the session listed; the rule that only the tab changes an
asset is the argument above.

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

### No action of the window that undo can reverse asks a confirmation `##no-discard-confirmation`

Delete, the Selection tab's Remove, a slide's removal and a change of a text to an image act at
once.
Each is an operation whose inverse restores what it discarded, `Op::Remove` with a snapshot of the
slide's former tree, per `design@rootpoint-model@edit-operation-core`, so one `undo` brings the
content back. The rival, a dialog before an action that discards content,
`rejected@rootpoint-app@removal-guard`, costs a click on every removal the author meant. On the
owner's word: the owner chose undo over a dialog to keep content from being lost by mistake. Undo
does not reach content whose history is gone, after a closed window or a reloaded page,
`issue@rootpoint-app@browser-autosave`, nor a step a later edit cleared from the redo stack,
nor a step the author cannot find, `issue@rootpoint-protocol@history-browsing`.

The close of a window whose edits are in no file is the one action that asks before it acts, since
no undo reaches a closed window: the window cancels the close by `ViewportCommand::CancelClose` and
asks "Close without saving?", with "Close" and "Keep open". Its edits are in no file when it holds
its presentation in memory with an undo step since the save, or an edit of a group still open, as
a text typed in place; and, in a workspace, while the guard of
`design@rootpoint-app@ron-overwrite-guard` stops the autosave's writes or its last write failed.
The page of the browser build asks the same of its close or its reload, by its `beforeunload`, the
browser's own question, `design@rootpoint-app@browser-pack-save`. A workspace whose autosave wrote
every undo step does not ask, `design@rootpoint-app@workspace-autosave`, and nothing unsaved asks
nothing. An open, an `open-pack` or a `sample` that replaces a presentation held in memory does
not ask either, on the owner's word: the owner chose an open that does not ask.

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

### An area's text is edited on the slide, its caret placed by Parley's `Cursor` on the layout the render draws `##edit-in-place`

The caret and the selection's anchor of a text being edited are caret positions of the text's tree,
`design@rootpoint-app@boundary-caret-positions`, which the `TextMap` of
`rootpoint_render::area_text` maps to the byte index of the layout `render_slide` draws,
`design@rootpoint-render@area-text-layout`; Parley's `Cursor` and `Selection` place, draw and move
the caret there, from that byte index and the affinity of the last move or click. Every change of
the text, a typed string, a deletion, a cut, a paste, an IME composition and its commit, is an
`insert` or a `delete` in one text node, or an `unwrap` of a span,
`design@rootpoint-app@empty-span-unwrapped`, the small commands of
`design@rootpoint-protocol@text-edit-commands`, per `design@rootpoint-app@actions-are-commands`, so
every span keeps its id; the render lays the new text out, and the caret keeps its caret position. A
move past a character passes one cluster as Parley 0.12.0 cuts the layout: a character with the
marks drawn on it is one cluster, so the caret stays on a glyph's boundary; each character of a
ligature is a cluster too, with an even share of its advance by `Cluster::advance`, where Parley's
own moves stop as well. The composing text stands in the text node it was typed into, by commands,
so the renderer draws it where it will stand; the overlay draws the caret, the selection and the
composition's underline, per `design@rootpoint-app@editor-overlay`. While editing, the app sets
`PlatformOutput::ime` every frame, since egui-winit 0.37.0 allows IME input only in a frame that
sets it.

So the caret sits on the drawn glyphs at every size of the view, as
`goal@rootpoint-render@same-slide-everywhere` asks of every display: a test over every character
of the sample's area n3 puts it on the drawn glyph at three window sizes, within half a pixel, a
tolerance for rounding that the session chose, and a property test walks the caret through
generated trees of spans at the same sizes, by `cargo test -p rootpoint-app --test editing`. A
toolkit text field over the slide would break lines differently and let the caret drift, as
`rejected@rootpoint-render@toolkit-painter` says, and Parley's `PlainEditor` lays its own copy of
the text out, `rejected@rootpoint-app@plain-editor`.

### A span's edge is two caret positions, and typing goes into the caret's text node `##boundary-caret-positions`

A caret position is a text node of the text's tree, its root or a span, and an offset in its
sequence, a span counting as one item, `design@rootpoint-model@text-node-tree`. Right at an offset
before a span enters it at offset 0, and at its last offset leaves it, after it in its parent; Left
is the mirror; both positions of an edge stand at one byte index of the layout. Typing inserts into
the caret's text node, so the author chooses, by an arrow, whether a typed letter joins a span. A
byte index goes back to a caret position, after a click or a move by Parley, in the deepest text
node that owns the glyph on the side the cursor's affinity names, the glyph after the index for
`Downstream` and before it for `Upstream`, which is the side of the glyph a click took, as Parley
0.12.0's `Cursor::from_point` sets it; with no glyph on that side, at the text's start or end, the
other side is taken. However many edges meet at an index, one glyph owns each side, so the rule
never has to choose among them.

The rival, one caret position per edge and a rule that guesses the typing style, is how LyX types
into its flat runs of fonts, by `Cursor::setCurrentFont`, in the survey of LyX,
`path@rootpoint@docs/grounding/lyx-nested-text.md`: the two positions the author asks for exist only
where the span is an item of the sequence, `rejected@rootpoint-app@one-position-per-edge`. On the
owner's word: the owner proposed the edge as an invisible character the caret enters, and judged
the explicit choice the key feature, and that in practice no place has many edges meeting, since
these trees rarely go deep, maths aside, where LaTeX has the same problem.

### Backspace and Delete act in the caret's text node, selecting a span before deleting it `##delete-in-caret-node`

With no selection, Backspace and Delete act on the item before or after the caret in its text node.
A character is deleted, by the logical clusters `App::delete` reads. A span with items is selected
whole by a first press and deleted by a second; an empty span goes at once. At a span's own start
for Backspace, or its end for Delete, the span is unwrapped, `unwrap`, its content kept in its
parent and the caret where it was drawn. At the root's start or end the keys do nothing.

So every edit of the keys stays in the caret's text node, where typing goes,
`design@rootpoint-app@boundary-caret-positions`, and a span's style is removed from the keyboard,
with no style action. The first press asks nothing: it selects what the second deletes, as Shift
with an arrow selects it, and the overlay draws the selection, so the author sees the whole span
before deleting it; any other key then acts on that selection. It is no confirmation of
`design@rootpoint-app@no-discard-confirmation`, whose rival is a question the author must answer
before an action. LyX acts the same, read in its source, the Codeberg mirror `LyX-org/lyx` at
commit `64141cf0e7a073e1226ec168d317618537ba0fab`: `site.bind` binds both keys with `confirm`;
`Text::dispatch` of `path@elsewhere@src/Text.cpp` selects a non-empty text inset before deleting
it, `InsetText::confirmDeletion` being `!text().empty()`; and `Text::backspace` at an inset's start
calls `dissolveInset`. The rival, deleting the character drawn before or after the caret in
whatever text node holds it, never a span whole, edits a node the caret is not in, which the
author does not see, and leaves an emptied span behind until the stay ends. The two shapes meet
every binding constraint. On the owner's word: the owner chose LyX's, finding it very convenient.

### A style action opens an empty span at the caret with no selection, and wraps a selection `##style-action-target`

A style action, Ctrl+B or Ctrl+I, or a control of the Selection tab while a text is edited,
`design@rootpoint-app@controls-act-at-the-caret`, sends with no selection `wrap` of the empty
range at the caret, the caret then put in the new span, so the next letters typed take its style;
with a selection, `wrap` of it, the new span then selected whole; and on a selection that is one
whole span, `span`, which changes that span's style, so a second action on a span just made
changes it rather than wrapping it again. Ctrl+B sets bold and Ctrl+I italic; colour, size, font
and a span style have no key, and are set from the Selection tab. A style action inside a span
already in that style opens a span all the same, since a span that sets a value equal to its
parent's is kept, `design@rootpoint-model@text-tree-normal-form`. The empty span stays while it is held,
`design@rootpoint-app@empty-span-unwrapped`.

So the author styles the letters about to be typed as well as those already typed, with no
markup, as `goal@rootpoint@no-language-no-build` asks of the editor. The rival, an action that
needs a selection, styles only what exists, and the author types first and styles after. On the
owner's word: asked whether opening a new span means creating an empty one at the caret, styling a
selection, or both, the owner chose both.

### While a text is edited, the Selection tab's controls are the style actions, and the ancestor row selects a span `##controls-act-at-the-caret`

While a text is edited, the Selection tab shows, in place of the area's content controls, the
ancestor row and the controls at the caret. The row lists "text", then each span that holds the
caret, outer first, with its style's name or "custom", or, while one span is selected whole, that
span and those that hold it; a click on a span selects it whole, and a click on "text" leaves
editing, as Esc does, the area still selected, where the controls change the area's style as
outside editing. "Split here" is the row's button, `design@rootpoint-app@split-at-caret`. The
controls show the values the caret types with, or those of a span selected whole, and each change
is a style action, `design@rootpoint-app@style-action-target`: the style menu lists the span styles
a span may name, and the values set a span's own; an alignment, which no span has, changes the
area's style by `text` with no tree. Every command joins the stay's group,
`design@rootpoint-app@gesture-groups`.

Each control then has one meaning at a time, and the window holds no state of its own for it; the
cost is that changing the area's size or style name while editing takes one click out of the stay.
The row's listing, which names each span with its style, comes from the frames' design: the
frames over the slide show where a span is, not its style. On the owner's word: the owner chose
what the controls and the row's clicks do over two other shapes, a target the row chooses for the
controls, the caret's, a span's or the area's, which never leaves editing but makes the author
read the target before each click, held by the window with a rule for when it returns to the
caret at each move of the caret; and the area's controls shown beside those at the caret, which
reach the area's style in the stay at the cost of about ten rows of the side panel in place of
five.

### "Split here" cuts the span that holds the caret at the caret, on Ctrl+Space, and leaves it at its edge `##split-at-caret`

Ctrl+Space, or "Split here" of the ancestor row, sends `split-span` of the span that holds the caret
at the caret, the caret then put between the two halves, in their parent; pressed again it cuts the
next span out. At the span's start or end it sends nothing, since `Op::SplitSpan` refuses there,
where one half would be empty, and puts the caret before or after the span; in the
root, or with a selection, whose caret's end is no place to cut at, it does nothing. So the author
leaves a style anywhere, or splits it, where the caret's positions alone leave it only at an edge.
The rival, a key per style that toggles it, Ctrl+B pressed in a bold span splitting it and going on
unbolded, has no meaning for a colour or a named span, where "Split here" acts the same for every
style.

Ctrl+Space is a key no other action of editing takes: Esc leaves editing,
`design@rootpoint-app@edit-entry`, the arrows move the caret and Shift with an arrow selects, and
Alt with an arrow is the node keys outside editing. It reaches the editor on the owner's desktop:
`gsettings get org.freedesktop.ibus.general.hotkey triggers` and `gsettings get
org.gnome.desktop.wm.keybindings switch-input-source` both name Super+Space; and egui-winit 0.37.0
sends no text for a key pressed with Ctrl, `on_keyboard_input`, so no space is typed. In the browser
it is kept from the page, `design@rootpoint-app@editor-keys-kept-from-browser`. On the owner's word:
the owner asked to be able to leave a style at any place, or split it.

### A selection is a range of one text node, a span inside it taken whole `##selection-one-level`

A selection's anchor and caret are lifted to their nearest common text node: an end inside a span of
that node takes the span whole, so a selection begun in a span and extended past its end rises to
the span's parent. Shift with an arrow at a span's edge passes the span whole. The anchor is kept
where it was set and lifted when the selection is read, as LyX's `CursorData::normalAnchor` lifts
its anchor, in the survey of LyX, so a selection dragged back inside the span selects in it again. A
cut and typing over a selection are then each one `delete` over one node's range, never a cut
through a span, `design@rootpoint-protocol@text-edit-commands`. The rival, a free selection across
nodes, makes each of them a cut of several nodes at once. On the owner's word: the owner counted the
rule, in the owner's opinion, a feature more than a cost, found LyX's selection model convenient,
and judged that cutting a span in half is practically never needed.

### The editor unwraps an empty span once neither the caret nor a selection of the span holds it, and every empty span when the stay ends `##empty-span-unwrapped`

The model accepts an empty span, `design@rootpoint-model@text-tree-normal-form`, so that a style
chosen with nothing selected opens one at the caret, `design@rootpoint-app@style-action-target`,
which stays while the caret is in it, or while a selection whose ends stand in its parent is that
span whole, as a click on the ancestor row makes one,
`design@rootpoint-app@controls-act-at-the-caret`, so a second control changes its style; a
selection lifted from an anchor left inside the span, as Shift with an arrow out of it leaves one,
does not hold it. When the caret and the selection leave an empty span, the editor sends `unwrap`
for it, and when a stay ends, for every empty span of the text, the inner first, inside the stay's
group, so the stay is still one undo step. A position inside the unwrapped span, an anchor left
there by Shift with an arrow, stands at the span's place in its parent. An empty span the caret has left draws nothing, and would remain an item the
arrows step into and a file keeps, with nothing to show the author where it is.

### A first click on a text area selects it, and a click on the selected text area edits it `##edit-entry`

A click on an area selects it. A click on the selected text area, so a double-click on any text
area, enters editing with the caret at the click, wherever no other gesture takes the press: a press
on the grip, `design@rootpoint-app@swap-gesture`, or on a divider's band keeps its own gesture and
never enters editing; Esc leaves it, the area still selected, and a press outside the area leaves it
and acts as it does outside editing: a click on another area selects that area. Outside editing, the
node keys, Shift with an arrow, Delete and Alt with an arrow, act on a selected text area, per
`design@rootpoint-app@selection-actions` and `design@rootpoint-app@selection-keys`; while editing,
the keys go to the text, where Shift with an arrow selects. While the control socket holds the
authority to edit, a click on the selected text area only selects it,
`design@rootpoint-app@one-controller-at-a-time`. In the final view a click selects and enters no
editing, `design@rootpoint-app@final-view-presses`, and a double-click begun there acts as that one
click. A secondary click in the slide view or on a row of the outline, inside the edited area too,
leaves editing before it opens the node menu, `design@rootpoint-app@node-menu`. A new area holds an
empty text, per `design@rootpoint-model@no-empty-kind`, so a second click enters editing on a new
area too, and typing into it needs no step that makes it a text first; a click on an image area or
an equation area, which hold no text, only selects it.

On the owner's word: the owner chose this shape over a single click that selects a text area and
edits it at once, which saves a click before typing in an area not yet selected, but leaves the node
keys to act on a text area only after Esc and makes every click on a text area place a caret; the
owner thinks it is quite intuitive to use. Also on the owner's word: a click enters editing only at
a place that serves no other gesture, so the grip and the dividers stay usable. And: a press outside
the edited area acts as it does outside editing, over a press that only leaves editing with the
edited area still selected, which would cost a second click to select another area. And: the owner
chose that the secondary click leaves editing before the menu opens, over a stay kept open under the
menu, and that a double-click begun in the final view acts as its one click, over its second press
acting as outside the final view, entering editing or setting a divider halfway. With the stay open, Delete and the node keys would act on the text while the menu
names the area; and the double-click's second press would act where the author, with no mark in
view, did not see it would, as `design@rootpoint-app@final-view-presses` argues of any press in the
final view.

### The caret and the selection of a text being edited are the editor's state, and no command places or reads them `##caret-editor-state`

While a text is edited, its caret, its selection and its composing range live in the app, as a
drag's position does, and the protocol has no command that places or reads them; every change of
the text they lead to is a command of `design@rootpoint-protocol@text-edit-commands`, per
`design@rootpoint-app@edit-in-place`. The caret is a caret position, a text node and an offset,
`design@rootpoint-app@boundary-caret-positions`, so its text node is part of the caret, which the
owner's ruling below covers as it stands. On the
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

The native window listens on a Unix domain socket, `rootpoint_app::control::Listener`, whose place
is `design@rootpoint-app@socket-place`. Each connection has a thread of its own, which reads one
line, queues it on a channel to the GUI's thread, calls the window's wake-up, and waits for the
reply before it reads the next; `App::frame` takes the lines queued when it starts, at most one per
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

### The control socket starts with the window, in the runtime directory, named by its store, and follows the store `##socket-place`

The socket is bound when the window starts, in the runtime directory,
`rootpoint_protocol::workspace::runtime_dir`: `$XDG_RUNTIME_DIR/rootpoint/`, or
`$TMPDIR/rootpoint/`, then `/tmp/rootpoint-<uid>/` where the variable is unset or relative, a
directory of the user's alone with mode 0700, `make_runtime_dir`, which refuses one another user
made first. Its name follows the store, `socket_name`: `<name>-<hash>.sock` in a workspace, the
root's name cut to 40 bytes, or less where the path would pass 103 bytes, the 104 of macOS's
`sun_path` less its NUL and below Linux's 108, and the first 8 hexadecimal digits of the 64-bit
FNV-1a hash of the canonical root; `memory-<pid>.sock` in memory. A workspace's
`.rootpoint/socket` holds the socket's path and nothing else, so a client finds the window from the
workspace, `design@rootpoint-protocol@ctl-sends-a-line-or-a-batch`. When a command changes the
store, the window's own or a line of the socket, the window renames its socket's file to the new
name, `Listener::rename_to`, removes the old workspace's pointer file and writes the new one's; a
socket keeps its connections when its file is renamed, which the test
`a_renamed_socket_answers_at_its_new_path` checks. A socket that cannot be bound leaves the window
without one, and the status line says why.

An agent reaches the window from the moment it opens, a new presentation in memory included, as
`goal@rootpoint-app@live-agent-edits` asks; the rival, a socket only where a flag names its path,
`rejected@rootpoint-app@socket-at-a-given-path`, leaves a window opened plainly unreachable. The hash
tells two workspaces of one name apart, where the name alone would make the second window's socket
refused, since the first answers there. The pointer file holds no flag of state: a flag goes stale
when the window crashes, and a client learns whether the window is live by connecting. The socket
needs no lock of its own: `App::run` moves the socket to a workspace's name only after
`workspace::run` took that workspace's lock, `design@rootpoint-protocol@workspace-lock`, so only the
holder of the lock binds its socket. The socket lies outside the workspace,
`rejected@rootpoint-app@socket-in-the-workspace`, and is a file of a directory no other user
reaches, against `rejected@rootpoint-app@abstract-namespace-socket` and
`rejected@rootpoint-app@tcp-socket`, and is one socket for every client, against
`rejected@rootpoint-app@named-pipes`. On the owner's word: the owner proposed the socket in the
runtime directory named by the workspace.

### The window replaces only a socket nothing answers, and refuses any other file at its path `##only-a-stale-socket-is-replaced`

At a bind or a rename, a socket at the path that accepts a connection is another window's, and is
refused; a socket that refuses connections is left by a run that ended, as a run killed by a signal
leaves it, and is removed first; a socket that fails to connect for another reason, and any file
that is no socket, is refused and kept. The window removes the socket's file when it closes, if it
is still the one it bound, by its inode. The rival, removing whatever refuses a connection, deleted
the presentation an author gave as the socket's path, since a regular file refuses a connection
too. Two windows starting at one moment on one workspace cannot both replace a stale socket and
bind, since only the holder of the workspace's lock binds its socket,
`design@rootpoint-app@socket-place`; the test `two_opens_of_one_workspace_bind_one_socket`
of `path@rootpoint-app@tests/workspace.rs` opens one workspace from two threads, 200 times, and
one open succeeds each time.

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
no divider highlight, no resize cursor, no grip, no line of a split at the pointer, no drop line in
the slide list. While the window holds it, every command of the socket but the queries, `control`
and `release` replies `err the window has control`, or the refusal of a take-over.

Two exceptions keep looking free. The socket's queries, `tree`, `view`, `store`, `styles`,
`span-styles`, `assets` and `render`, need no authority, since they apply no operation and join no group. The window's
inputs that change only
the view need none either: a click selects, and never enters editing; Alt with an arrow, a row of
the outline and the outline of its node under the pointer, the node menu's "Select", whose other
entries are disabled, a row of the slide list, a use's button of the Assets tab, presenting and
presenter mode's keys, and the button "Preview"; and the window's own state, the side panel's tab,
a style selected in the list of the Styles tab, an asset selected in the Assets tab, its search and
its filter, and the node menu itself. The view stays the session's one view,
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

## The workspace on disk

### A workspace saves itself after every undo step, on a thread that writes the newest clone of the document `##workspace-autosave`

The native window holding a workspace starts `rootpoint_app::autosave::Autosave` at the open, and
sends it a clone of the document after each command that changed `Session::unsaved`, the undo
steps; the thread keeps only the newest clone while a write runs, serializes it, and writes
`presentation.ron` through `.rootpoint/presentation.ron.new`, flushed, then renamed, so a reader
never sees half a file. Dropping the window writes the clone still waiting, before the window
releases the workspace's lock. The thread reads no session and applies no change, so it is no
action of `design@rootpoint-app@actions-are-commands`; `rootpoint-headless` has none, and a script
writes by `save`.

The thread is what keeps an undo step within its bound. The test `autosave_gui_cost` of
`path@rootpoint-app@tests/autosave.rs`, run by
`cargo test -p rootpoint-app --release --test autosave -- --ignored --nocapture`, times the
medians of 21 runs at 50, 200 and 1000 slides of the sample, whose document holds no image's
bytes: on telmon, an AMD Ryzen 9 9950X, the GUI thread's share, the clone and its hand-over, is
0.023, 0.084 and 0.524 ms, and `save_ron`, which the rival would run on the GUI thread before its
write, is 0.130, 0.569 and 3.304 ms, linear in the slides. The test asserts a share below 1 ms at
1000 slides, a sixteenth of one frame at 60 Hz, 16.7 ms, where `save_ron` alone takes 3.3 ms, near
a fifth of the frame. The thread also takes the time of a slow `fsync`, as on a USB stick, which no
frame waits for.

With every step written, the files on disk agree with the window but for one step at most, so a
watcher, git or an agent reading the directory sees the tree the window shows, which an explicit
save of the asset changes would leave behind until the save. On the owner's word: the owner chose
a workspace that saves itself, probably on a background thread, as the owner put it, over an
explicit save, with the time of an autosave measured, its scaling included.

### An autosave writes nothing over a `presentation.ron` changed outside rootpoint, and asks `##ron-overwrite-guard`

The window reads `presentation.ron` at the open, as the autosave's baseline, and, before each
write, the thread compares the file with the bytes it last wrote or read. When they differ, it
writes nothing, and the writes stop: a banner over the toolbar says "presentation.ron was changed
outside rootpoint", with "Load the file", which runs `reload`,
`design@rootpoint-protocol@reload-is-an-undo-step`, and "Keep mine", which writes the window's
document; after either, the writes resume, from the bytes read or written. A file that does not
load is named in the banner, which stays. The state label then says the file changed,
`design@rootpoint-app@visible-mode`. The guard knows every write and every read of the
window: a `save` of the window or of its socket is a write of the thread, replying `err` while the
writes are stopped, a `reload` from the banner or a script resumes from the file it read, and an
`open` of the workspace held starts the autosave again.

An autosave makes the window a writer that could overwrite a change nobody saw, after a
`git checkout`, a `git stash` or an agent's edit of the file; the workspace lock does not help,
since those tools do not take it. The banner is not the dialog
`rejected@rootpoint-app@removal-guard` rejects: that one asked before an action undo reverses, and
the banner guards content the window never held. A write between the comparison and the rename, a
window of about one write's time, is not seen: closing it would need a lock git and editors do not
take.
`tripwire@rootpoint-app@guard-shows-with-no-outside-writer` watches a tool that rewrites the bytes
with no change.

### The window reads an asset again when its file changes, watching only the directories of the assets `##asset-watch`

The native window holding a workspace watches, by `notify` 8.2.0, each directory that holds an
asset of the table, not recursively, so the slide view shows the bytes the files hold,
`goal@rootpoint-app@faithful-slide-view`; an event that changes the file of an asset, by its bytes
or its name, drops the decoded assets and renders the slide again, and any other event, a read of a
file, a change of its metadata, or a file that is no asset, `presentation.ron` and `.git/`
included, decodes nothing. After any event the directories are watched again, since a directory
removed and made again, as a checkout does, lost its watch with the old one. A recursive watch of
the root would see every file of `.git/` and of a build tree; the crate's documentation says every
watch counts toward `fs.inotify.max_user_watches`, and a recursive one counts every file and
folder below it; the limit is 65,536 on telmon, read by `cat /proc/sys/fs/inotify/max_user_watches`.
The test `the_watcher_decodes_a_changed_asset_and_nothing_of_git` of
`path@rootpoint-app@tests/autosave.rs` commits to a git repository at the root and decodes nothing.
In memory nothing on disk is the presentation's, so nothing is watched. On the owner's word: the
owner asked for the assets to be watched.

### The window loads a `presentation.ron` changed outside rootpoint only by a command an author or a script sends `##ron-loaded-on-request`

No event of `presentation.ron` loads it: the guard of `design@rootpoint-app@ron-overwrite-guard`
meets a change of it at the next write, and the file is loaded only by a command an author or a
script sends: `open`, or `reload`, which the banner's "Load the file" runs. On the owner's word: the owner saw no reason to watch `presentation.ron`, whose
correct editing, in the owner's opinion, is by the window or the text protocol.

## The browser build

### The browser build opens a pack or a picked directory in memory, and saves by downloading a pack `##browser-pack-save`

Built for the wasm target, the editor opens the built-in sample at start; Open asks for a directory
with the browser's directory picker and opens a copy of its `presentation.ron` in memory, and Open
pack… asks for a `.rpk` file and runs `open-pack` on its bytes. Its edits stay in memory. "Save as
pack…" runs `pack` into its files, `MemoryFiles`, which keep the writes, and hands the bytes to the
page, which downloads them; Ctrl+S, once a pack is the session's target, runs `save` and downloads
it again, and with none packs as "Save as pack…" does, `App::pack_for_download`. It holds no
workspace, memory being its one mode on the owner's word, so `MemoryFiles` moves and removes no
file; a store of the page's origin stays a lead of `issue@rootpoint-app@browser-autosave`. A pack is the one file that carries a
presentation from the browser to any computer, `design@rootpoint-model@workspace-layout`, as
`goal@rootpoint@runs-in-browser` asks; the page asks before it unloads while an edit is in no file,
`design@rootpoint-app@no-discard-confirmation`. `tripwire@rootpoint-app@browser-pack-too-large`
watches a presentation too large for the page's memory.

On the owner's word: the owner set memory as the browser build's one mode, and that the browser
build saves, which the owner counts part of the goal of the browser build,
`goal@rootpoint@runs-in-browser`; and proposed opening a whole directory rather than file by file.

### A picked directory opens as a copy in memory: its files held by their paths, its presentation's assets copied into the session's map `##picked-files-by-path`

`App::open_directory` holds the picked files in `MemoryFiles`, the session's `Files`, under their
paths, the picked directory's name first, and runs `open-copy` on that name, which copies the
directory's `presentation.ron` and the files of its assets into the session's map,
`design@rootpoint-protocol@editing-modes`; `App::add_picked_image` holds an image under its name and
runs `import`, which copies it into the map. The session reads through `Files` as it reads a disk,
per `design@rootpoint-protocol@session-files`, so the document is the one its file holds and a
script of the same commands reproduces it, as `goal@rootpoint@headless-core` asks of every
operation. The rival, `open` of the picked directory as a workspace, would be a workspace whose
files live in the page's memory alone, which `MemoryFiles` neither moves nor removes,
`design@rootpoint-app@browser-pack-save`.

### In the browser, presenter mode asks the page for full screen, and ends when the page leaves it `##page-full-screen`

eframe 0.37.0's web backend handles no `ViewportCommand::Fullscreen`: `handle_viewport_commands`
of `path@elsewhere@src/web/app_runner.rs` logs it as not implemented. So, on the wasm target,
`App::frame` asks the page's document for full screen when presenter mode starts, and leaves it
when it ends. The browser takes Esc to leave full screen, and the page may not receive the key, so
presenter mode ends when the page leaves full screen while presenting, as Esc ends it natively;
`goal@rootpoint-app@presents-full-screen` asks presenter mode to be full screen.

### In the browser, no key of the editor reaches a browser action the author did not ask for `##editor-keys-kept-from-browser`

A key's default action in the browser, a reload or a step back in the history, leaves the page,
which holds its edits in memory alone, `design@rootpoint-app@browser-pack-save`, after the page's
question when an edit is in no download: a key the editor uses would raise that question, or leave. eframe 0.37.0 prevents
the default of some keys itself, in `should_prevent_default_for_key` of
`path@elsewhere@src/web/events.rs`: every arrow key, so Alt with an arrow,
`design@rootpoint-app@selection-keys`, never goes back in the history, and Ctrl with Comma, O, P
or S. It
never prevents F5's, which starts presenter mode, and its listener on the canvas stops the
propagation of every key, `WebOptions::should_stop_propagation` being true by default. So a
listener of the window in the capture phase prevents F5's default before the canvas has the key,
and the canvas still receives it. A listener in the bubble phase never sees the key while the
canvas has focus, `rejected@rootpoint-app@bubble-phase-f5-listener`. The test
`f5_is_kept_from_the_browser_when_the_canvas_stops_its_propagation` of
`path@rootpoint-app@tests/web.rs` checks it in a headless browser.

The rule covers every key the editor consumes whose default in the browser is an action the
author did not ask for, whether it leaves the page or not. The editor reads its shortcuts with
egui's `command`, which eframe 0.37.0's web input sets from Ctrl or Meta, `modifiers_from_kb_event`
of `path@elsewhere@src/web/input.rs`: open and save, with O and S, which eframe keeps itself; undo
and redo, with Z and Y, always; and while a text is edited select all, with A, and bold and
italic, with B and I, `design@rootpoint-app@style-action-target`; and, while a text is edited too,
Ctrl+Space, `design@rootpoint-app@split-at-caret`.
`should_prevent_default_for_key` names none of these letters, and prevents Space's default only
while its text agent lacks the focus, which it holds while a text is edited, since the app sets
`PlatformOutput::ime` then, `path@elsewhere@src/web/app_runner.rs`. So the same listener prevents
their defaults, those of editing while a flag of the editor holds, which `App::frame` sets at each
frame to whether a text is edited: eframe's text agent, which holds the focus while editing, hands
its keydown to the canvas's handler, `path@elsewhere@src/web/text_agent.rs`, and the window's
listener in the capture phase sees both first. Outside editing the editor consumes none of the keys
of editing, and the browser keeps them. The rival, letting the browser act too, makes one key both
edit the presentation and act in the browser: Firefox gives Ctrl+B to its bookmarks sidebar and
Ctrl+I to page information, by the session's knowledge of the browser, not measured, and the rule
does not rest on it, since a prevented default costs a key the editor consumes nothing. The
editor's other keys have no such default: Home, End, Page Up and Page Down scroll a page that
cannot scroll, `path@rootpoint-app@web/index.html` hiding the overflow of a page the canvas fills;
and the default of Ctrl with C, V and X is the copy, the paste and the cut the author asked for,
which carry the clipboard's events to eframe, so preventing it would stop them. The rival that
keeps every key the editor reads from the browser stops the clipboard. The test
`the_keys_of_editing_are_kept_from_the_browser_only_while_editing` of the same file checks the
keys with Ctrl or Meta.
