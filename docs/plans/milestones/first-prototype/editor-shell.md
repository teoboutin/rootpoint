# Slice: The native GUI and presenter mode

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites.

## Builds

- The crate `rootpoint-app` at `planned@rootpoint@crates/rootpoint-app/`, with the binary
  `rootpoint`, native only in this slice: a slide list, a slide view that shows `render_slide`'s
  image, selection of an area or of a split, dragging a divider, splitting, removing and swapping
  from the UI, setting an area's content and style from a side panel, undo, open and save, and
  presenter mode, full screen, with previous and next.
- Per D2, the crate's Component.

## Claims

1. **Every UI edit goes through the session.** Each UI action issues a protocol `Command` to a
   `Session`, so the document after a UI action equals the document after the same command. Test:
   the app's action handlers are called on a session in a unit test, with no window. Shown to fail:
   a handler that edits the document directly bypasses the undo stack, which the test detects by
   `undo` not restoring it.
2. **Hit testing matches the drawing.** A click at a point selects the area whose rectangle from
   `area_rects`, scaled to the view, contains it, and a press within 6 pixels of a divider grabs
   that divider. Test: unit tests over the geometry with no window.
3. **Presenter mode shows the slide the editor shows.** At the same pixel size, the presenter's
   image equals the editor view's image of the same slide.

## Audit subjects

- `thread@first-prototype@gui-toolkit`, `thread@first-prototype@slide-renderer`.
- The documentation of `eframe` at its current version (0.37.0 per
  `argument@first-prototype@arg-26`): textures, input, full screen.
- The duplicate `vello_cpu` versions of the render slice's premise.

The interaction design (how a split is started, how a subtree is selected for a spanning split, how
a divider is shown) is not decided by the milestone. The audit of this slice opens a design session
on it under `skill@knowledge-architect-design`, before the code is written.

## Fails alone on

- A UI action whose effect differs from its command's, with the protocol's tests passing.
- A drag frame over AC2's bound with the render crate's measured times within it, which points at
  the host's texture path, not at the renderer.

## Premises that expire

- The text of an area is edited in the side panel's plain text field in this slice. The slice
  `spec@first-prototype@text-editing` replaces it with editing in place.

## Implementation sequence

1. **A window that shows one slide.** The app opens a file and shows its first slide's render,
   letterboxed. Fails alone on: the texture upload.
2. **Selection and dividers.** Claim 2; dragging a divider issues `divider` commands.
3. **Editing from the UI.** Splitting, removing, swapping, content and style, undo, open and save.
   Claim 1.
4. **Presenter mode.** Claim 3. Then AC2's drag bound.

## Decided design

### The host

eframe hosts the slide view as a texture of `render_slide`'s image and forwards input to the app's
own hit testing; it draws no slide content itself (`thread@first-prototype@gui-toolkit`,
`thread@first-prototype@slide-renderer`).

## Acceptance criteria

This slice judges the drag bound of AC2, `acceptance@first-prototype@render-fast-enough`, at
step 4: dragging a divider of the third render fixture in a 1600×900 editor view, release build, on
the owner's machine, with frame times taken over 100 frames and the 95th percentile reported.

## Harvest

Judged by the entry tests of `primer@design-heads`, into the design home of the app crate's
Component: `thread@first-prototype@gui-toolkit`, and
each decision of the interaction design session that earns a head; the losing hosts, judged for a
`rejected` entry.
