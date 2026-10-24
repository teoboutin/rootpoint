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
- A `beforeunload` listener that asks while `Session::unsaved` is not 0.
- The state label in the browser build, by the milestone's table of its states.
- One text slide in `path@rootpoint-protocol@sample/sample.ron`, by the milestone's "Decided design"
  of #sample-explains-modes.

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
- Rewrite `design@rootpoint-model@image-storage` where it names the browser.
- Update `path@rootpoint-app@README.md`, whose section on the browser build says it saves nothing
  and loses its changes on a reload.
- Re-read `issue@rootpoint-app@browser-autosave` against what the browser build then holds.
- Write T4, guarding the browser's head.
- The milestone document's own row: its "Harvest" section.
