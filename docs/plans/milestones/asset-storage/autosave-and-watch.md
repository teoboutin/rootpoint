# Slice: autosave, its guard, and the asset watcher

The fourth slice of `milestone@plans@asset-storage`. It lands the autosave of
`thread@asset-storage@asset-disk-changes`, `thread@asset-storage@ron-overwrite-guard` and
`thread@asset-storage@asset-watch`, in the native window's workspace.

## Builds

- `Op::ReplaceDocument { document }` in `path@rootpoint-model@src/ops.rs`, whose inverse holds the
  document it replaced, and a command that applies it from the store's `presentation.ron`, named at
  the audit; it is one undo step.
- `planned@rootpoint-app@src/autosave.rs`: a thread that receives a clone of the document at the end of
  each undo step, keeps only the newest while a write runs, compares `presentation.ron`'s bytes with
  those it last wrote or read, and writes to a temporary file at the root then renames it, or stops
  and reports the difference to the window. The window shows the banner "presentation.ron was
  changed outside rootpoint" with "Load the file" and "Keep mine".
- `planned@rootpoint-app@src/watch.rs`: a watcher of the directories that hold an asset of the table,
  not recursive, which wakes the window by `Context::request_repaint`; `Assets::refresh` decodes
  again an asset whose file's modification time changed. The crate is chosen at the audit, and
  built for native targets only.
- A benchmark, outside the gates, for AC1.

## Claims

- **AC1**, `acceptance@asset-storage@ac-autosave-gui-cost`: the benchmark prints the median of 21
  runs of the GUI thread's share at 1000 slides, under 1 ms on the owner's machine; the commit
  carries the figure.
- **An autosave follows each undo step, and only the newest of a burst is written**: a test with a
  slow `Files` counts the writes.
- **The guard writes nothing over an outside change**: a test changes `presentation.ron` between two
  gestures and finds it unchanged, and the banner shown; "Load the file" then gives the outside
  document, and undo the window's; "Keep mine" writes the window's. Shown to fail with the
  comparison removed.
- **AC3**, `acceptance@asset-storage@ac-watch-asset-dirs`: a commit in a git repository at the root
  decodes nothing; a changed asset file is decoded and drawn again.
- **No thread and no watcher in the browser build**: the gate `wasm` builds, and the browser tests
  pass.

## Audit subjects

- `path@rootpoint-app@src/assets.rs`, `path@rootpoint-app@src/lib.rs` (`App::frame`, `App::run`).
- `design@rootpoint-app@gui-toolkit`, `design@rootpoint-model@edit-operation-core`,
  `design@rootpoint-app@no-discard-confirmation`, `rejected@rootpoint-app@removal-guard`.
- The watcher crate's documentation on inotify's limits.

## Fails alone on

- AC1, AC3, and the guard's banner after an outside write.

## Premises that expire

- Only rootpoint writes `presentation.ron` byte for byte as it last wrote; T2 watches a tool that
  rewrites the bytes with no change.

## Decided design

The milestone's sections on `thread@asset-storage@asset-disk-changes`,
`thread@asset-storage@ron-overwrite-guard` and `thread@asset-storage@asset-watch` hold the shape.

## Acceptance criteria

This slice judges `acceptance@asset-storage@ac-autosave-gui-cost` and
`acceptance@asset-storage@ac-watch-asset-dirs`.

## Harvest

- Heads of the app for the autosave, the guard and the watcher, by the tests of
  `primer@design-heads`; a `rejected` entry for shape B, #arg-10, and for a reload of
  `presentation.ron` on each change, #arg-11 and #arg-15.
- Write T2, guarding the guard's head.
- `changed-asset-not-read-again` was closed when the milestone was added; nothing
  to close here.
- Update `path@rootpoint-app@README.md` for autosave and the banner.
