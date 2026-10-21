# Slice: Editing text in place

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites.

## Builds

- Editing the text of an area in place, in the slide view: a caret, selection by drag and by
  shift+arrow, typing with IME and dead keys, copy and paste, driven by Parley's `PlainEditor` on
  the same layout `render_slide` draws. Each committed edit issues one `text` command.
- The side panel's plain text field of the slice `spec@first-prototype@editor-shell` is removed.

## Claims

1. **The caret is where the glyph is.** For a fixture text, the caret's x position after moving to
   every offset equals the glyph boundary of the render's layout at that offset, within 0.5 pixel,
   at three window sizes. Test: a unit test over the editor and the layout, with no window. Shown to
   fail: positioning the caret from a layout made at the pixel size moves it.
2. **An edit is one undoable command.** Typing a word then pressing undo restores the text before
   the word. Test: the session's undo stack after a scripted edit.

## Audit subjects

- `thread@first-prototype@slide-renderer`, `thread@first-prototype@gui-toolkit`,
  `thread@first-prototype@text-style-per-area`.
- The documentation of Parley's `PlainEditor`, and eframe's text and IME events.

## Fails alone on

- A caret offset that the layout and the editor disagree on, with the render's golden images
  passing.

## Premises that expire

- Style stays per area. Rich text with spans, per "Later consequences" of the milestone document,
  builds on this editor.

## Implementation sequence

1. **A caret over a rendered text.** Entering an area shows the caret at the clicked offset; no
   editing. Claim 1.
2. **Typing, selection, IME.** Then claim 2, then AC1.

## Decided design

The editor and the renderer share one Parley layout of the area, in slide units; the host forwards
keys, text and IME events to it and draws nothing of the text itself
(`thread@first-prototype@slide-renderer`). A toolkit text field over the slide is excluded, since
its line breaks differ and the caret would drift (`argument@first-prototype@arg-35`).

## Acceptance criteria

This slice judges AC1, `acceptance@first-prototype@caret-on-glyph`, after step 2.

## Harvest

Judged by the entry tests of `primer@design-heads`: the shared layout between editing and drawing,
into the design home of the app crate's Component or of the render crate's, by where the code that
holds it lands.
