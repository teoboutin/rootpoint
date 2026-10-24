# Slice: autosave, its guard, and the asset watcher

The fourth slice of `milestone@plans@asset-storage`. It lands the autosave of
`thread@asset-storage@asset-disk-changes`, `thread@asset-storage@ron-overwrite-guard` and
`thread@asset-storage@asset-watch`, in the native window's workspace.

## Builds

- `Op::ReplaceDocument { document }` in `path@rootpoint-model@src/ops.rs`, whose inverse holds the
  document it replaced, and the command `reload`, which reads `presentation.ron` at the root and
  applies it, one undo step; `err` naming the error of `load_ron` when the file does not load, and
  `err` in memory, where no file holds the presentation.
- `path@rootpoint-app@src/autosave.rs`: a thread that receives a clone of the document at the end
  of each undo step, keeps only the newest while a write runs, compares `presentation.ron`'s bytes
  with its baseline, and writes to a temporary file at the root then renames it, with the native
  file system; or stops and reports the difference to the window. Its baseline is read when the
  workspace opens, and is the bytes it last wrote or read. It counts its writes, for the tests.
  - The temporary file is `.rootpoint/presentation.ron.new`: inside the root, on its file system,
    so the rename is atomic, and ignored by git with the rest of `.rootpoint/`. The write is
    flushed by `File::sync_all` before the rename, as #arg-16 measured it.
  - The native window starts the thread when its store becomes a workspace, `open` or `unpack`, and
    stops it when the store leaves the workspace; it sends a clone after each command that changed
    `Session::unsaved`, the count of undo steps. An app made by `App::new`, as the tests over files
    held in memory make, has no autosave, and its label keeps the count of changes since a `save`.
  - `App::autosave()` gives the thread's handle to the tests: its count of writes, and a hold that
    keeps the next write waiting, for the claim on a burst.
  - A workspace with no `presentation.ron` yet has no baseline: the guard compares "no file" with
    "no file".
- The banner "presentation.ron was changed outside rootpoint", with "Load the file" (`reload`) and
  "Keep mine"; after either, autosave resumes. A `reload` that fails names its error in the banner,
  which stays.
- `path@rootpoint-app@src/watch.rs`: a watcher of the directories that hold an asset of the
  table, not recursive, which wakes the window by `Context::request_repaint`. The crate is `notify`
  8.2.0, the current release, 9.0.0 being a release candidate, a dependency of the app on native
  targets only. Read at the audit in its documentation: each watch counts toward
  `fs.inotify.max_user_watches`, 65,536 on telmon, and a recursive watch counts every file and
  folder below it, which a watch of each asset's directory, not recursive, keeps to the number of
  those directories.
  - An event that names the file of an asset of the table drops the decoded assets, which
    `Assets::refresh` then decodes again, as `App::run` drops them after a store's change: the
    cache's key holds paths, not bytes, and `Files` has no modification time to key on. An event of
    any other file, `presentation.ron` and `.git/` included, decodes nothing. `Assets` counts its
    decodings, for AC3's test.
- The state label's states "before #autosave-and-watch" leave: the label of a workspace whose
  autosave runs reads "Workspace `<path>`, saved", and that of one the guard stopped reads
  "Workspace `<path>`, not saved: the file changed outside rootpoint", by the milestone's table
  "The store's state to the state label".
- `autosave_gui_cost` in `path@rootpoint-app@tests/autosave.rs`, `#[ignore]`, run by
  `cargo test -p rootpoint-app --release --test autosave -- --ignored`, for AC1.

## Claims

- **AC1**, `acceptance@asset-storage@ac-autosave-gui-cost`: `autosave_gui_cost` prints the median of
  21 runs of the GUI thread's share at 1000 slides, under 1 ms on the owner's AMD Ryzen 9 9950X; the
  commit carries the figure.
- **An autosave follows each undo step, and only the newest of a burst is written**: a test that
  holds the thread's first write counts the writes after a burst of steps.
- **The guard writes nothing over an outside change**: a test changes `presentation.ron` between two
  gestures and finds it unchanged, and the banner shown; "Load the file" then gives the outside
  document, and undo the window's; "Keep mine" writes the window's; autosave resumes after either.
  Shown to fail with the comparison removed.
- **AC3**, `acceptance@asset-storage@ac-watch-asset-dirs`: a commit in a git repository at the root
  decodes nothing; a changed asset file is decoded and drawn again, which serves
  `goal@rootpoint-app@faithful-slide-view`, the goal of the closed `changed-asset-not-read-again`.
- **The state label leaves its states before autosave**: a test of `App::state_label` in a
  native workspace reads "Workspace `<path>`, saved" after an edit, and "Workspace `<path>`, not
  saved: the file changed outside rootpoint" once the guard stops the writes.
- **No thread and no watcher in the browser build**: the gate `wasm` builds, and the browser tests
  pass.

## Audit subjects

- `path@rootpoint-app@src/assets.rs`, `path@rootpoint-app@src/lib.rs` (`App::frame`, `App::run`).
- `design@rootpoint-app@gui-toolkit`, `design@rootpoint-model@edit-operation-core`,
  `design@rootpoint-app@no-discard-confirmation`, `design@rootpoint-app@actions-are-commands`,
  `rejected@rootpoint-app@removal-guard`.
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
- Add to `issue@rootpoint-app@abandoned-composition-undo-step` that its empty step also costs one
  autosave write that changes nothing.
- Update `path@rootpoint-app@README.md` for autosave and the banner, and
  `path@rootpoint-protocol@README.md` for `reload`.
