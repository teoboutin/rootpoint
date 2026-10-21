# Slice: The native GUI and presenter mode

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites.

## Builds

- The crate `rootpoint-app` at `planned@rootpoint@crates/rootpoint-app/`, with the binary
  `rootpoint`, native only in this slice: every UI action that "The UI actions" in "The prototype's
  scope" of the milestone document lists, the text of an area being edited in the side panel's
  plain text field, and presenter mode, full screen. The binary target reads `ROOTPOINT_CHECKOUT`,
  as `path@xtask@CLAUDE.md` requires.
- The built-in sample presentation, embedded in the binary with its images as
  `AssetSource::Embedded`, opened from a menu, as a tutorial and demonstrator, per D3.
- The crate is not added to the `wasm` gate here: its web target is built by
  `spec@first-prototype@browser-build`, as "How the work is done" says.
- Per D2, the crate's Component.

## Claims

1. **Every UI action goes through the session.** Each UI action issues a protocol `Command` to a
   `Session`, an editing command or a view command, so the document and the view after a UI action
   equal those after the same command. Test: the app's action handlers are called on a session in a
   unit test, with no window. Shown to fail: a handler that edits the document directly bypasses
   the undo stack, which the test detects by `undo` not restoring it.
2. **Hit testing matches the drawing.** A click at a point selects the area whose rectangle from
   `area_rects`, scaled to the view, contains it, and a press within 6 pixels of a divider grabs
   that divider (a grab distance of the session's choice, which the interaction design session may
   change). Test: unit tests over the geometry with no window.
3. **Presenter mode shows the slide the editor shows.** At the same pixel size, the presenter's
   image equals the editor view's image of the same slide.

## Audit subjects

- `thread@first-prototype@gui-toolkit`, `thread@first-prototype@slide-renderer`,
  `thread@first-prototype@slide-coordinate-space`.
- The documentation of `eframe` 0.37.0: textures, input, full screen.
- The native libraries eframe's windowing needs on the owner's machine, which the existing `clippy`
  and `test` gates build from this slice on, since they run over the whole workspace; not yet
  established.
- The duplicate `vello_cpu` versions: egui 0.37 depends, through epaint, on `vello_cpu` 0.2, while
  `path@rootpoint@crates/rootpoint-render/` uses 0.3, so the workspace builds both once the app
  crate exists (V, quoted in `argument@first-prototype@arg-26`). The audit measures the build time and the binary size with both, and judges
  whether that costs anything.

The interaction design (how a split is started, how a subtree is selected for a spanning split, how
a divider is shown) is not decided by the milestone. The audit of this slice opens a design session
on it under `skill@knowledge-architect-design`, before the code is written.

## Fails alone on

- A UI action whose effect differs from its command's, with the protocol's tests passing.
- A drag frame over AC2's bound with the render crate's measured times within it, which points at
  the host's texture path, not at the renderer.

## Premises that expire

- The text of an area is edited in the side panel's plain text field in this slice. Guard:
  `spec@first-prototype@text-editing` replaces it with editing in place, and removes the field.

## Implementation sequence

1. **A window that shows one slide.** The app opens a file and shows its first slide's render,
   letterboxed. Fails alone on: the texture upload.
2. **Selection and dividers.** Claim 2; dragging a divider issues `divider` commands.
3. **Editing from the UI.** The slide list's actions, splitting, removing, swapping, content and
   style, undo, open and save. Claim 1.
4. **Presenter mode.** Claim 3. Then AC2's drag bound.

## Decided design

### The host

eframe hosts the slide view as a texture of `render_slide`'s image and forwards input to the app's
own hit testing; it draws no slide content itself (`thread@first-prototype@gui-toolkit`,
`thread@first-prototype@slide-renderer`).

Nearest rival: iced, and the others; see "Losing alternatives" of the milestone document.

### Uniform scaling

The slide view and presenter mode scale the slide uniformly to the largest size that fits the
window, and letterbox the rest (`thread@first-prototype@slide-coordinate-space`).

## Acceptance criteria

This slice judges the drag bound of AC2, `acceptance@first-prototype@render-fast-enough`, at
step 4: dragging a divider of the third render fixture in a 1600×900 editor view, release build, on
the owner's machine, with the time of each of 100 frames taken; the bound holds when every frame is
within it, and the largest is reported.

## Harvest

Judged by the entry tests of `primer@design-heads`: `thread@first-prototype@gui-toolkit`, and each
decision of the interaction design session that earns a head, into the design home of the app
crate's Component; the built-in sample as a permanent feature of every build, attributed to the
owner, per D3, into the same home; the uniform scaling of
`thread@first-prototype@slide-coordinate-space`, which the render and app crates both respect, into
rootpoint's design home; the losing hosts, judged for a `rejected` entry.
