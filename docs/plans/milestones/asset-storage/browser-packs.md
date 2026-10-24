# Slice: packs in the browser, and the sample's slide on the modes

The seventh and last slice of `milestone@plans@asset-storage`. It lands
`thread@asset-storage@browser-pack-save`, the browser's half of
`thread@asset-storage@unsaved-close-warning` and `criterion@asset-storage@c-visible-mode`, and
`thread@asset-storage@sample-explains-modes`.

## Builds

- In `path@rootpoint-app@src/web.rs`: Open accepts a `.rpk` file from the browser's file picker,
  beside the directory picker, and runs `open-pack` on its bytes; "Save as pack…" builds the pack
  and hands it to the page as a download; the Save buttons of `path@rootpoint-app@src/panels.rs`
  show in the browser build. `MemoryFiles` accepts the writes of a pack.
- A `beforeunload` listener that asks while the session holds an edit made since the last open or
  save.
- The state label in the browser build, by #c-visible-mode's form.
- One text slide in `path@rootpoint-protocol@sample/sample.ron` on the two modes, what each save
  writes, the `.rpk` file and `.rootpoint/`; no image, so the sample stays within its 50 KB of
  images, `design@rootpoint-protocol@builtin-sample`.

## Claims

- **A pack round-trips in the browser**: a test of `path@rootpoint-app@tests/web.rs` opens a pack,
  edits, saves and opens the saved bytes, on the wasm target.
- **The warning is asked only with an edit**: a browser test dispatches `beforeunload` before and
  after an edit, and reads the event's default prevented.
- **The state label reads each state the browser build has**, the two in memory.
- **The sample's slide count and image bytes**: a test reads the sample and checks its images stay
  under 50 KB in all.

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

The milestone's sections on `thread@asset-storage@browser-pack-save` and
`thread@asset-storage@unsaved-close-warning` hold the shape.

## Acceptance criteria

None is judged at this slice; the milestone's last report on every criterion is in its harvest.

## Harvest

- Reverse `design@rootpoint-app@browser-load-only` to the browser's packs, on the owner's word of
  round 2, and update the texts the milestone's table lists for it: the lines of
  `register@rootpoint-app@design` and `register@rootpoint-protocol@design`, and the comments
  of `path@rootpoint-app@src/actions.rs`, `path@rootpoint-app@src/memory.rs` and
  `path@rootpoint-app@src/panels.rs`.
- A head of the app for the warning on close, from `thread@asset-storage@unsaved-close-warning`,
  citing `design@rootpoint-app@no-discard-confirmation`.
- Rewrite `design@rootpoint-model@image-storage` where it names the browser.
- Write T4, guarding the browser's head.
- The milestone document's own row: its "Harvest" section.
