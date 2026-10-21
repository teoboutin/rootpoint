# Slice: The editor and presenter in a web browser

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites.

## Builds

- The app crate built for the wasm target with eframe's web support, served as a static directory:
  the same editing and presenter mode as native. Per D3, ruled, it is load only: it opens the
  built-in sample, and a presentation directory, the RON file and its images, that the user picks
  with the browser's directory picker (an `<input type="file">` with the `webkitdirectory`
  attribute; its support and eframe's access to it are assumed, checked at the audit); it edits
  them in memory and does not save. Images loaded this way are held as `AssetSource::Embedded` in
  memory.
- The app crate added to the `wasm` gate.
- The command that builds the static directory, as a command of the maintenance tool, per
  `goal@xtask@repeated-tasks-are-automated`.

## Claims

1. **The browser build has the same editing operations.** Every UI action that "The UI actions" in
   "The prototype's scope" of the milestone document lists is available in the browser build, open,
   save excepted per D3; adding an image reads it through the browser's file picker. Test: the app's
   action handlers are compiled for the wasm target and called by a `wasm-bindgen-test` test. If the
   audit finds no runner for that test on the owner's machine, the audit puts the replacement check
   to the owner as a scope change.
2. **The same layout.** A slide rendered in the browser build and natively at the same size gives
   the same image. Test: the browser build's render of the sample's first slide is saved and
   compared to the native golden image.

## Audit subjects

- `thread@first-prototype@gui-toolkit`, `thread@first-prototype@font-sources`,
  `thread@first-prototype@protocol-transport`, `goal@rootpoint@runs-in-browser`.
- eframe's web template, and the tool that builds it (trunk or wasm-bindgen-cli; assumed, chosen at
  the audit).

## Fails alone on

- A native build that passes every claim and a browser build that does not.

## Premises that expire

- The browser build does not save: a browser cannot write the files beside a presentation. Guard:
  packing, per "Later consequences" of the milestone document, ends it, and
  `goal@rootpoint@runs-in-browser` is met only then.

## Implementation sequence

1. **The empty web app.** The app compiles for the wasm target and shows an empty slide, served from
   a static directory. Fails alone on: the web build.
2. **The sample, editing and presenter.** Claims 1, 2; then AC3.

## Acceptance criteria

This slice judges AC3, `acceptance@first-prototype@browser-build-holds`, after step 2. After the
slice merges, the owner judges AC4, `acceptance@first-prototype@split-editing-convenient`.

## Harvest

Nothing of its own beyond the app crate's web build, judged by the entry tests of
`primer@design-heads` into the app crate's Component. The milestone's closing commit, which its
"Harvest" describes, follows this slice.
