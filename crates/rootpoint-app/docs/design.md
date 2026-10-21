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
asks the GPU for textures of at most 8192 pixels a side (`egui-wgpu/src/setup.rs`), below the
render crate's own limit.

It depends on `design@rootpoint-render@slide-renderer`: under the project's own renderer the host
needs only panels, a list, a texture view, input forwarding, a browser build, which
`goal@rootpoint-app@browser-window` asks, and a way for another thread to wake it, which
`goal@rootpoint-app@live-agent-edits` needs: eframe sets egui's repaint callback, natively and on
the web, so `Context::request_repaint` called from another thread wakes the window. Its rivals
lost on the toolkit survey of the first prototype, `rejected@rootpoint-app@other-gui-hosts`; if
the renderer decision is reopened toward the toolkit's own painter, iced is the host to weigh
again. The texture path is fast enough for a drag: a divider drag in a 1600×900 view takes at most
5.9 ms a frame, release build, on the owner's Intel Core Ultra 7 165H, the frame timed around
`Context::run_ui`, by `cargo test --release -p rootpoint-app --test presenter -- --ignored
--nocapture`; the GPU's upload of the texture is not in it.

### Every action of the editor is a protocol command, run on the one session `##actions-are-commands`

Each action, a click, a key, a drag, a button, issues the `Command`s of the protocol that do what
it does, through `App::run`, which runs them on the app's one `rootpoint_protocol::Session`; the
app changes the document and the view nowhere else. Each action is a function of
`path@rootpoint-app@src/actions.rs`, which issues its commands, and
`path@rootpoint-app@tests/actions.rs` checks each against the same commands on a second session,
the documents, the views and the files they write, and again after one `undo`.

So every operation the GUI offers runs headless, as `goal@rootpoint@headless-core` asks, and a
script of the same commands reproduces what the author did. The rival, the GUI changing the
document itself, `rejected@rootpoint-model@gui-mutation`, implements every edit twice.

### Each gesture of the window is one undo group, closed at its end or when focus is lost `##gesture-groups`

A divider drag, a grip drag and a focus of the text field each open a group with `begin` when they
start, after closing any group still open, so that within one frame a group closes before another
opens. The release closes a drag's group with `end`, and Esc with `cancel`; the loss of the field's
focus closes its group; and the loss of the window's focus closes either, so a release never
delivered leaves no group open. While a drag runs, no key but Esc acts, since a key's edit would
join the drag's step. The groups are the protocol's, `design@rootpoint-protocol@gesture-undo`, so a
gesture is one undo step in a script as in the window.

### The editor's marks are drawn over the slide's texture with egui, never into the slide's image `##editor-overlay`

The divider lines, the highlight of the divider under the pointer, the selection's outline and its
grip, the preview line of a split and the hint of a drop are drawn with egui's painter over the
texture, in `App::overlay`. `render_slide` draws only the slide, so the image presenter mode shows
is the editor's, pixel for pixel at one size, which `path@rootpoint-app@tests/presenter.rs`
compares, and which `goal@rootpoint-app@faithful-slide-view` asks. The marks are lines, rectangles
and the hint's label, editor text that is no slide content, so they are not the slide text that
`rejected@rootpoint-render@toolkit-painter` keeps out of the toolkit.

## Selecting

### A node is selected by a click, and its parent, children and siblings by keys and a breadcrumb `##node-selection`

A click on an area sends `select`. Alt+↑ selects the parent, and does nothing at the root; Alt+↓
the child of the selection selected last, or its first child, and nothing on an area; Alt+← and
Alt+→ the previous and next sibling, and nothing at the first or the last. The app keeps for each
split the child last selected in it, which only chooses the node a `select` names. The side panel
shows the path from the slide's root to the selection, `s1 › n4 stacked › n6 beside › n3 area`,
and a click on an entry selects it. The selection is the session's view, per
`design@rootpoint-protocol@view-state`.

A click lands in an area, so a pointer alone reaches no split, and a split is what a spanning
split needs, as `goal@rootpoint@free-slide-division` asks; the keys and the breadcrumb are the two
ways up and back down. Shipped tools do the same in the survey of split editing,
`path@rootpoint@docs/grounding/split-editing.md`: i3's `focus parent`, `focus child` to the child
last focused and its sibling steps, and Figma's Shift+Enter and Enter. A click on a divider grabs
it rather than selecting its split, since a drag starts there. On the owner's desktop every one of
these keys reaches the app, by the owner's report on the built editor.

### A subtree is selected for a spanning split by moving up to its node, and no rectangle selects `##span-selection`

To split across several areas, the author selects the split that holds them, with Alt+↑ or the
breadcrumb, and splits it. No rectangle drawn across areas selects anything. A rectangle selection
is a second way to select, and its result surprises: the smallest node holding the rectangle can be
much larger than it, and two of three children of a split are no node of their own, so selecting
them would build the grouping that `tripwire@rootpoint-model@grouping-lost-to-merge` watches.

## Splitting and resizing

### A split is started at the selection's middle by buttons and keys, or at the pointer with Ctrl or Shift `##selection-actions`

Four buttons of the side panel, and Shift with an arrow, split the selection at its middle, the
new area left, right, above or below: `split <node> beside|stacked 0.5 before|after`. With Ctrl
held over the selection, a click splits it beside at the pointer, and with Shift, stacked; the new
empty area takes the smaller part, `after` when the two are equal. Delete removes the selection.

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
slide units is one point.

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

## Moving content

### An area is swapped with another, or moved to its edge, by dragging its grip, with a hint of the drop's effect `##swap-gesture`

The selected area shows a grip, a square of 12 points set 8 points inside its top-left corner, so
no divider's band reaches it. A drag of the grip runs in a group, per
`design@rootpoint-app@gesture-groups`. Over another area, its middle third, by width and by
height, swaps: the drop sends `swap`. The rest moves the dragged area to the edge nearest the
pointer, the distances taken as fractions of the target's width and height, left and right winning
a tie: the drop splits the target on that edge at its middle, swaps the dragged area with the new
one and removes the dragged one. The zone under the pointer is tinted, with a label, "swap" or
"move left", "move right", "move up", "move down". A drop on the dragged area itself, or outside
any area, cancels.

A plain drag inside an area is not the gesture, since a drag inside a text area is kept for
selecting its text, which editing text in place, `spec@first-prototype@text-editing`, needs. A
swap button followed by a click on the target, which no fact defeats, adds a mode and costs the
same two actions. The edge move replaces a removal, a split and a new edit of the content with one
gesture, as `goal@rootpoint-app@direct-manipulation` asks. In the survey of split editing, iced
swaps on a centre drop and splits on an edge drop; i3 moves on a centre drop, swaps with a modifier,
and inserts on an edge drop; Blender swaps by a drag from a corner, and docks by an edge drop; and
the tools highlight the target. On the owner's word: the owner accepted the edge moves on the condition that
a preview or a hint shows what a drop will do, judging that a preview of the exact result might not
be meaningful here.
