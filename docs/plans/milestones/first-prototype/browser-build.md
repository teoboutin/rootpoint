# Slice: The editor and presenter in a web browser

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites.

## Builds

- The app crate built for the wasm target with eframe's web support, served as a static directory:
  the same editing and presenter mode as native, on a sample presentation built into the binary.
  Opening, saving and adding an image wait on packing, per D3, which awaits the owner.
- The app crate added to the `wasm` gate.
- The command that builds the static directory, as a command of the maintenance tool, per
  `goal@xtask@repeated-tasks-are-automated`.

## Claims

1. **The browser build has the same editing operations.** Every UI action that "The UI actions" in
   "The prototype's scope" of the milestone document lists is available in the browser build, open,
   save and adding an image excepted per D3. Test: the app's action handlers are compiled for the
   wasm target and called by a `wasm-bindgen-test` test. If the audit finds no runner for that test
   on the owner's machine, the audit puts the replacement check to the owner as a scope change.
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

- The browser build opens only the built-in sample: with images as files beside the presentation,
  a browser cannot resolve their paths. Guard: packing, per "Later consequences" of the milestone
  document, ends it, and `goal@rootpoint@runs-in-browser` is met only then.

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
