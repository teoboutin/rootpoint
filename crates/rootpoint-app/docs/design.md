# Design of rootpoint-app

How rootpoint-app is built, and why: one level-three heading per decision, under level-two
subjects. Design that is decided and not built is in a plan document, not here. What rootpoint-app
is for is in `register@rootpoint-app@goal`.

## The host

### The window is an eframe app that shows the renderer's image as a texture and draws no slide content itself `##gui-toolkit`

eframe hosts the editor: its panels, the slide list, the side panel and the slide view, and
presenter mode. The slide view and presenter mode show the image `render_slide` returns, rendered
with `Precision::Native` at the view's size in physical pixels and uploaded with no conversion,
since the render is premultiplied RGBA8 in sRGB, the format of `ColorImage::from_rgba_premultiplied`:
`Context::load_texture` once, then `TextureHandle::set`. A side is at most 8192 pixels, both sides
scaled by one factor beyond it, because egui-wgpu 0.37.0 asks the GPU for textures of at most 8192
pixels a side (`egui-wgpu/src/setup.rs`), below the render crate's own limit. eframe's feature
`system_fonts` is off: the editor's own text needs only egui's built-in fonts, the browser has none,
and the feature links a second fontique beside parley's. The frame code takes an `egui::Ui`, so a
test drives it with `Context::run_ui` and synthetic input, with no window and no GPU.

It depends on `design@rootpoint-render@slide-renderer`: under the project's own renderer the host
needs only panels, a list, a texture, input and a browser build. Its rivals lost on the toolkit
survey of the first prototype, `rejected@rootpoint-app@other-gui-hosts`; if the renderer decision
is reopened toward the toolkit's own painter, iced is the host to weigh again. The texture path
is fast enough for a drag: a divider drag in a 1600×900 view takes at most 5.9 ms a frame,
release build, on the owner's Intel Core Ultra 7 165H, the frame timed around `Context::run_ui`,
by `cargo test --release -p rootpoint-app --test presenter -- --ignored --nocapture`; the GPU's
upload of the texture is not in it.

### Every action of the editor is a protocol command, run on the one session `##actions-are-commands`

Each action, a click, a key, a drag, a button, issues the `Command`s of the protocol that do what
it does, through `App::run`, which runs them on the app's one `rootpoint_protocol::Session`; the
app changes the document and the view nowhere else. The commands each action issues are in the
doc comment of its function in `path@rootpoint-app@src/actions.rs`, and
`path@rootpoint-app@tests/actions.rs` checks each against the same commands on a second session,
the documents, the views and the files they write, and again after one `undo`. A gesture that
spans frames is one undo step through the protocol's groups, `design@rootpoint-protocol@gesture-undo`:
a divider drag, a grip drag and a focus of the text field each open a group, after closing any
group still open, and close it at the release, at the loss of the field's focus, or when the
window loses focus, so a release never delivered leaves no group open; while a drag runs, no key
but Esc acts.

So every operation the GUI offers runs headless, as `goal@rootpoint@headless-core` asks, and a
script of the same commands reproduces what the author did. The rival, the GUI changing the
document itself, `rejected@rootpoint-model@gui-mutation`, implements every edit twice.

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

A click lands in an area, so a pointer alone reaches no split: in the survey of split editing,
`path@rootpoint@docs/grounding/split-editing.md`, shipped tools go up by a command and back down,
i3's `focus parent`, `focus child` to the child last focused and its sibling steps, and Figma's
Shift+Enter and Enter. A click on a divider
grabs it rather than selecting its split, since a drag starts there. On the owner's desktop every
one of these keys reaches the app, by the owner's report on the built editor.

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
empty area takes the smaller part, `after` when the two are equal. A split is refused, its button
disabled and its click ignored, where a part would be under 40 slide units, the bound a divider
drag keeps. Delete removes the selection.

On the owner's word: the owner chose both ways over the middle split alone, judging, in the
owner's opinion, that the buttons are easy to learn and the pointer split faster once its keys are
mastered. The pointer split saves a drag
per split, and `split` takes `at` as a fraction of the node, so the pointer maps straight onto it.

### A split at the pointer shows its line before it commits, and Alt snaps the line `##split-preview`

While Ctrl or Shift is held over the selection, a line follows the pointer where the split would
go; releasing the key, or Esc, removes it and issues nothing. With Alt also held, the line snaps to
the nearest twelfth of the node, or to a divider across the same axis elsewhere on the slide within
8 points, the nearer of the two; a divider drag snaps the same way. Blender, in the survey of split
editing, previews a pointer split and snaps it to twelfths; snapping to other dividers lines up rows
across columns, the need `issue@rootpoint-model@linked-dividers` defers, without linking them. On
the owner's word too: the owner chose the preview with the snapping over the preview alone.

### A divider is grabbed within 4 points of its line, keeps each side at 40 slide units, and resets on a double-click `##divider-drag`

A press within 4 points of a divider grabs it, the nearer of two within 4 points, and the cursor
is a resize cursor there. The drag keeps the press's offset from the line and sends nothing until
the pointer moves, so a click leaves no step; it is one undo step, a group per
`design@rootpoint-protocol@gesture-undo`, and Esc cancels it. It stops where a child of the split
would be under 40 slide units along its axis. A double-click sets the divider halfway between its
neighbours, the neighbouring dividers or the split's edges, in one `divider` command. The band is
in points, so it is the same under the pointer at every size of the view; a band in slide units
narrows with the view.

The survey of split editing found shipped bands of 6 to 10 pixels in all (egui_tiles 6, iced's
example 10), a reset to the mean on a double-click in egui_tiles, and minimum sizes in Blender and
egui_tiles; within that range the values are defaults that no fact decides between.

## Moving content

### An area is swapped with another, or moved to its edge, by dragging its grip, with a hint of the drop's effect `##swap-gesture`

The selected area shows a grip, a square of 12 points set 8 points inside its top-left corner, so
no divider's band reaches it. A drag of the grip runs in a group, per
`design@rootpoint-protocol@gesture-undo`. Over another area, its middle third, by width and by
height, swaps: the drop sends `swap`. The rest moves the dragged area to the edge nearest the
pointer, the distances taken as fractions of the target's width and height, left and right winning
a tie: the drop splits the target on that edge at its middle, swaps the dragged area with the new
one and removes the dragged one. The zone under the pointer is tinted, with a label, "swap" or
"move left", "move right", "move up", "move down". A drop on the dragged area itself, outside any
area, or on a move zone whose split would leave a part under 40 slide units, cancels.

A plain drag inside an area is not the gesture, since a drag inside a text area is kept for
selecting its text, which editing text in place needs. Shipped tools in the survey of split editing, Blender, i3 and iced,
swap on a centre drop and move and split on an edge drop, and highlight the target. On the owner's
word: the owner accepted the edge moves on the condition that a preview or a hint shows what a
drop will do, judging that a preview of the exact result might not be meaningful here.
