# Slice: packs in the browser, and the sample's slide on the modes

The seventh and last slice of `milestone@plans@asset-storage`. It lands
`thread@asset-storage@browser-pack-save`, the browser's half of
`thread@asset-storage@unsaved-close-warning` and `criterion@asset-storage@c-visible-mode`, and
`thread@asset-storage@sample-explains-modes`.

## Builds

- In `path@rootpoint-app@src/web.rs`: Open accepts a `.rpk` file from the browser's file picker,
  beside the directory picker, and runs `open-pack` on its bytes; "Save as pack…" runs `pack` and
  hands the bytes to the page as a download; Ctrl+S does the same once a pack is the session's
  target. The Save buttons of `path@rootpoint-app@src/panels.rs` show in the browser build, and the
  app's `MemoryFiles` accepts the writes of a pack.
  - The toolbar's file buttons show in both builds: "Open pack…", "Save" and "Save as pack…"; the
    native build adds "Pack to…" in a workspace and "Save as workspace…" or "Unpack to…" in memory,
    which write a directory the page cannot. In the browser, "Open pack…" asks the page's file
    picker for a `.rpk` file, `Web::pick_pack`, whose bytes reach `App::open_picked_pack`.
  - "Save as pack…" runs `pack <name>`, the session's pack or `presentation.rpk`, into the app's
    `MemoryFiles`, which keeps the writes in its map; then the bytes written go to the page, which
    downloads them through a `Blob`, an object URL and a click on an `<a download>` element,
    `Web::download`. Ctrl+S, with a pack as the target, runs `save`, then the same download.
- A `beforeunload` listener that asks while `Session::unsaved` is not 0: `App::in_memory`, in the
  browser build, installs it on the page's window, holding a weak reference to a flag the app sets
  after each command and each frame from the edits no file holds, so a dropped app asks nothing;
  the listener prevents the event's default and sets its `returnValue`, which browsers read as the
  question.
- The state label in the browser build, by the milestone's table of its states: the toolbar,
  shared by both builds, draws it already; this slice tests it there.
- One text slide in `path@rootpoint-protocol@sample/sample.ron`, by the milestone's "Decided design"
  of #sample-explains-modes, after the others, its ids 20 to 23, so the ids of the sample's slides,
  nodes and image stay as the tests name them; an id a test takes from the counter after the
  sample, as an import's, grows by 4.

## Claims

- **A pack round-trips in the browser**: a test of `path@rootpoint-app@tests/web.rs` opens a pack,
  edits, saves and opens the saved bytes, on the wasm target.
- **The warning is asked only with an edit**: a browser test dispatches `beforeunload` before and
  after an edit, and reads the event's default prevented.
- **The state label reads each state the browser build has**, the three in memory.
- **The sample's images stay under 50 KB in all**, a test that sums them.

## Audit subjects

- `path@rootpoint-app@src/web.rs`, `path@rootpoint-app@src/panels.rs`,
  `path@rootpoint-app@src/memory.rs`, `path@rootpoint-app@tests/web.rs`.
- `design@rootpoint-app@browser-load-only`, `design@rootpoint-app@editor-keys-kept-from-browser`,
  `design@rootpoint-protocol@builtin-sample`, `issue@rootpoint-app@browser-autosave`.

## Fails alone on

- The browser tests of a pack's round trip.

## Premises that expire

- The map holds every byte of a presentation; T4 watches a presentation too large for the browser.

## Decided design

The milestone's sections on `thread@asset-storage@browser-pack-save`,
`thread@asset-storage@sample-explains-modes` and `thread@asset-storage@unsaved-close-warning` hold
the shape.

## Acceptance criteria

None is judged at this slice; the milestone's last report on every criterion is in its harvest.

## Harvest

- Reverse `design@rootpoint-app@browser-load-only` to the browser's packs, on the owner's word of
  round 2, and update the texts the milestone's table lists for it: the lines of
  `register@rootpoint-app@design` and `register@rootpoint-protocol@design`, and the comments of
  `path@rootpoint-app@src/actions.rs`, `path@rootpoint-app@src/memory.rs` and
  `path@rootpoint-app@src/panels.rs`.
- Add the browser's half to the head of the warning on close written at #workspace-host.
- Rewrite `image-storage`, now `design@rootpoint-model@workspace-layout`, where it names the browser.
- Update `path@rootpoint-app@README.md`, whose section on the browser build says it saves nothing
  and loses its changes on a reload.
- Re-read `issue@rootpoint-app@browser-autosave` against what the browser build then holds.
- Write T4, guarding the browser's head.
- The milestone document's own row: its "Harvest" section.
