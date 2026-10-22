# Slice: The editor and presenter in a web browser

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites.

## Builds

- The app crate built for the wasm target with eframe's web support, served as a static directory:
  the same editing and presenter mode as native. Per D3, ruled, it is load only: it opens the
  built-in sample, and a presentation directory, the RON file and its images, that the user picks
  with the browser's directory picker; it edits them in memory and does not save. eframe 0.37.0
  has no file picker, so the app creates an `<input type="file">` with the `webkitdirectory`
  attribute through web-sys, and reads each file's `webkitRelativePath` by reflection, since
  web-sys 0.3.106 binds no getter for it. The picked files, and an image added, are held in memory
  by path, `MemoryFiles`, which is the session's `Files`, so the document keeps the
  `AssetSource::Path` its file names and `Session::asset_bytes` reads each path from memory.
- The page's side of presenter mode: eframe 0.37.0's web backend ignores
  `ViewportCommand::Fullscreen` (`handle_viewport_commands` of `path@elsewhere@src/web/app_runner.rs`), so the app
  asks the page for full screen itself, and leaves presenter mode when the page leaves full screen,
  since the browser takes Esc for that. eframe never prevents F5's default action
  (`should_prevent_default_for_key` of `path@elsewhere@src/web/events.rs`), which reloads the page and loses
  what is held in memory, so a listener of the window prevents it. eframe prevents the default of
  every arrow key, so Alt with an arrow, `design@rootpoint-app@node-selection`, never goes back in
  the browser's history.
- The app crate added to the `wasm` gate.
- The command that builds the static directory, `cargo x web`, per
  `goal@xtask@repeated-tasks-are-automated`: the editor's binary built in release for the wasm
  target, bound by the `wasm-bindgen` program of `wasm-bindgen-cli` at the version of the
  `wasm-bindgen` crate in `Cargo.lock`, and the page of `planned@rootpoint-app@web/index.html`.
  wasm-bindgen-cli over trunk: claim 1's runner, `wasm-bindgen-test-runner`, comes with it, so one
  tool serves both.

## Claims

1. **The browser build has the same editing operations.** Every UI action that "The UI actions" in
   "The prototype's scope" of the milestone document lists is available in the browser build, open,
   save excepted per D3; adding an image reads it through the browser's file picker. Test:
   `path@rootpoint-app@tests/actions.rs` runs in a headless Firefox through
   `wasm-bindgen-test-runner` and geckodriver, both on the owner's machine, as the new gate
   `wasm-test` of `path@xtask@src/main.rs`; and `planned@rootpoint-app@tests/web.rs` checks
   `App::open_directory` and `App::add_picked_image`, the actions a picker's files go through, on
   both targets.
2. **The same layout.** A slide rendered in the browser build and natively at the same size gives
   the same image. Test: in `planned@rootpoint-app@tests/web.rs`, the image the app hands its
   texture for the sample's first slide in presenter mode, in a 640×360 window, equals pixel for
   pixel the golden image a native run of the same test writes,
   `planned@rootpoint-app@tests/data/sample-s1.png`.

## Audit subjects

- `thread@first-prototype@gui-toolkit`, `thread@first-prototype@font-sources`,
  `thread@first-prototype@protocol-transport`, `goal@rootpoint@runs-in-browser`.
- eframe's web template, and the tool that builds it: wasm-bindgen-cli, chosen at the audit, see
  "Builds".
- The socket's listener, `design@rootpoint-app@control-socket`, built on Unix only, under
  `cfg(unix)`, which the wasm target is not: the app crate joins the wasm gate in this slice, and
  the browser build has no socket, so its window holds control always.

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
