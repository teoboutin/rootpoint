# Slice: Editing text in place

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites. Its design audit, recorded in the commit "Apply the design audit of the fifth slice, editing
text in place", rewrote the sections below against the editor the slice `editor-shell` built, and
left one thread open for the owner, `thread@first-prototype@edit-entry`, which the owner closed on
shape (a).

## Builds

- Editing the text of an area in place, in the slide view: a caret, selection by drag, by
  Shift+click and by Shift with an arrow, typing, IME composition and dead keys, and copy, cut and
  paste. The caret and the selection are placed with Parley's `Selection` and `Cursor` on the very
  layout `render_slide` draws for the area, which the render crate exposes. Each change of the
  text issues one `text` command with the area's whole text; one stay in editing is one undo step.
- In the render crate, `area_text`: the laid-out text of one text area of a slide, in slide units,
  with the origin it is drawn at, from the function `render_slide` draws the area's text with, so
  the editor and the drawing read one layout.
- The side panel's plain text field is removed. The side panel keeps the content kind, the style
  and the image controls.

## Claims

1. **The caret is where the glyph is.** For a fixture text, the caret's x position after moving to
   every offset equals the glyph boundary of the render's layout at that offset, within 0.5 pixel
   (a test tolerance for rounding, the session's choice), at three window sizes. Test: a unit test
   over the editor and the layout, with no window. Shown to fail: positioning the caret from a
   layout made at the pixel size moves it.
2. **One stay in editing is one undoable step.** Typing a word then pressing undo restores the text
   before the word. Test: the session's undo stack after a scripted edit, through the app's
   actions with no window.
3. **Every edit goes through the session.** Each editing action issues the `text` commands it maps
   to, checked against a second session as `path@rootpoint-app@tests/actions.rs` does for the other
   actions.

## Audit subjects

The audit is held; its findings are listed in the commit named in the status above, and applied in
this spec. What it read:

- `thread@first-prototype@slide-renderer`, `thread@first-prototype@gui-toolkit`,
  `thread@first-prototype@text-style-per-area`, and the heads the slice `editor-shell` harvested:
  `design@rootpoint-app@editor-overlay`, `design@rootpoint-app@gesture-groups`,
  `design@rootpoint-app@actions-are-commands`, `design@rootpoint-app@node-selection` and
  `design@rootpoint-app@selection-actions`.
- Parley 0.12.0's editing module, `parley/src/editing/`: `PlainEditor` owns its own `Layout`,
  built from its own buffer and `StyleSet` (`editor.rs`, the struct's field `layout`), while
  `Selection` and `Cursor` take any `&Layout` (`selection.rs`, `cursor.rs`).
- The render crate's `layout_text` (`path@rootpoint-render@src/text.rs`): a `ranged_builder` at
  scale 1.0 with quantizing on, wrapped to the area's inner width and aligned; `render_slide`
  draws it at the area's origin plus `TEXT_PADDING` and the vertical alignment's offset.
- egui 0.37.0's input and output: `Event::Ime(ImeEvent::Preedit { text, .. } | Commit(_))`, and
  `Event::Copy`, `Event::Cut` and `Event::Paste(String)` (`egui/src/data/input/`); egui-winit
  allows IME on the window only in a frame whose `PlatformOutput::ime` is set
  (`egui-winit/src/lib.rs`, `allow_ime = ime.is_some()`), with the text's rectangle and the caret's.
- The standing entries the search returned, each judged: `issue@rootpoint-app@changed-asset-not-read-again`
  bears on nothing here, since the caret and the drawing read one layout even from a stale font;
  `tripwire@rootpoint-protocol@client-joins-gui-group` is not met, and a stay in editing holds a
  group open as the text field's focus did, which the audit of `spec@first-prototype@live-control`
  reads; `tripwire@rootpoint-render@glyph-missing-everywhere` is not met, and the status line keeps
  naming the areas with a missing glyph.

## Threads

### How an area's text is entered for editing, and what the node keys do on a text area `##edit-entry`

Open at the audit, put to the owner as Q28 with a default, shape (a). Closed by the owner: "Q28:
(a) is good, I think it's quite intuitive to use." Final state: approved, shape (a). Proposed by
the session at the audit:
the slice's entry said only "Entering an area shows the caret at the clicked offset". A click on an
area selects it, and while a text is edited the keys go to the text, so Shift with an arrow, Delete
and Alt with an arrow, which act on the selected node, cannot also act while editing. The
discriminating fact: `design@rootpoint-app@node-selection` and `design@rootpoint-app@selection-actions`
bind those keys to the selection, and AC1 needs Shift with an arrow to select text. Two shapes meet
every binding constraint:

- **(a), the default.** A click on an area selects it, as now. A click on the selected text area,
  or a double-click on any text area, enters editing with the caret at the clicked offset; Esc, or a
  click outside the area, leaves it, the area still selected. The node keys keep acting on a
  selected text area outside editing. Cost: one more click before typing in an area not yet
  selected.
- **(b).** A click on a text area selects it and enters editing at once. Typing needs no second
  click; the node keys act on a text area only after Esc, and a click on a text area always places
  a caret.

Harvest: judged by the entry tests, into the app crate's design home, with the ruling attributed.

## Decided design

The editor and the renderer share one Parley layout of the area, in slide units; the host forwards
keys, text and IME events to the editor and draws nothing of the text itself
(`thread@first-prototype@slide-renderer`). A toolkit text field over the slide is excluded, since its
line breaks differ and the caret would drift (`argument@first-prototype@arg-35`).

- **One layout.** `area_text` of the render crate gives the area's laid-out text and its origin in
  slide units, from the same function `render_slide` draws with. The app keeps the caret and the
  selection as Parley's `Selection`, moved by its methods on that layout: `Selection::from_point`
  for a click, `extend_to_point` for a drag and `shift_click_extension` for a Shift+click,
  `next_visual` and `previous_visual`, `next_line` and `previous_line`, `line_start` and `line_end`,
  with `extend` for Shift. A typed string, a deletion, a cut or a paste replaces the selection's
  byte range in the area's text, and the edit is a `text` command with the whole new text; the
  render then lays it out again, and the selection is placed on the new layout by byte index with
  `Selection::from_byte_index`. `PlainEditor` is not used: it lays its own copy of the text out,
  from its own styles, which would be a second layout that has to agree with the drawn one.
- **One group.** Entering editing opens a group, per `design@rootpoint-app@gesture-groups`, which
  the text field's focus held before; leaving editing, the window's loss of focus, an undo or a redo
  close it, so one stay in editing is one undo step. An editing action that is no part of the stay,
  a button of the slide list or the side panel, closes it first, as every action does.
- **IME and dead keys.** A preedit string from `ImeEvent::Preedit` is put into the area's text at the
  caret, replacing the previous preedit, by a `text` command inside the group, so the renderer draws
  it where it will stand; the overlay underlines it. `ImeEvent::Commit` keeps its text and clears the
  preedit, and an empty preedit removes it. While editing, the app sets `PlatformOutput::ime` every
  frame with the area's rectangle and the caret's, so the window accepts IME input. A dead key's
  composed character arrives as `Event::Text` or through the IME, and is typed as either.
- **Copy, cut and paste.** `Event::Copy` puts the selected text on the clipboard with
  `Context::copy_text`; `Event::Cut` does so and deletes it; `Event::Paste` inserts its string.
- **Marks.** The caret, a 2-point line on the cursor's geometry, and the selection's rectangles,
  from `Selection::geometry`, are drawn by `App::overlay` over the texture, per
  `design@rootpoint-app@editor-overlay`, placed through the area's origin and `slide_to_pixels`.
- **Keys while editing.** The arrows, Shift with an arrow, Home, End, Backspace, Delete, Enter (a
  new line) and typed text act on the text; Ctrl+Z and Ctrl+Shift+Z or Ctrl+Y close the stay's
  group and undo or redo, as the protocol's `undo` inside a group does; Esc leaves editing. The
  editor's other shortcuts do not act while editing.
- **The pointer while editing.** Inside the edited area, after the grip and a divider's band, a
  press places the caret, a drag selects, the use `design@rootpoint-app@swap-gesture` keeps a drag
inside a text area for, and a Shift+click
  extends the selection; elsewhere a press leaves editing, then acts as "Pointer input to gestures"
  of the app gives it.
- **The view.** The caret and the selection are the editor's state during a stay, as a divider
  drag's position is, and no command of the protocol: every change of the text they lead to is a
  command, which is what `goal@rootpoint-protocol@commands-drive-everything` measures.

## New names

```text
crates/rootpoint-render/  area_text, AreaText { layout, origin }
crates/rootpoint-app/     the editing state of the app: the area, its Selection, the preedit's byte
                          range
```

## Fails alone on

- A caret offset that the layout and the editor disagree on, with the render's golden images
  passing.

## Premises that expire

- Style stays per area. Guard: rich text with spans, per "Later consequences" of the milestone
  document, is designed in a session of its own before it builds on this editor.

## Implementation sequence

1. **A caret over a rendered text.** `area_text`; entering editing per
   `thread@first-prototype@edit-entry` shows the caret at the clicked offset, and the arrows move it;
   no editing. Claim 1.
2. **Typing, selection, IME, the clipboard.** The side panel's text field removed. Claims 2 and 3,
   then AC1.

## Acceptance criteria

This slice judges AC1, `acceptance@first-prototype@caret-on-glyph`, after step 2.

Judged after step 2. The caret's place at three window sizes is claim 1's test. The owner tried the
other parts on the owner's desktop and reported: "AC1: 1, 2 and 3 work fine. However, on window
resize, the caret is lost, the selected text node is exited (like on pressing esc)." The three
were entering editing with the caret at the click, a dead key typing "é", and selection by drag and
by Shift with an arrow. The resize ended the stay through the window's loss of focus, and is
repaired in the commit "Keep editing in place when the window loses focus, for the text editing
slice", whose test resizes the window unfocused and keeps the caret. AC1 did not fire.

In the same message the owner reported that Alt+↓ from the title of the sample's first slide does
not always reach the bottom row, which the session measured as the tree navigation of
`design@rootpoint-app@node-selection`, and put to the owner as Q29: keep it, or design a spatial
one. The owner ruled: "agreed on default, keep a and open an issue to rediscuss, then proceed toward
review and merge". The issue is `issue@rootpoint-app@alt-arrows-follow-the-tree`.

## Harvest

Judged by the entry tests of `primer@design-heads`: the shared layout between editing and drawing,
into the design home of the app crate's Component or of the render crate's, by where the code that
holds it lands; `thread@first-prototype@edit-entry`, into the app crate's design home, with the
owner's ruling attributed; `design@rootpoint-app@gesture-groups` rewritten for a stay in editing in
place of the text field's focus.
