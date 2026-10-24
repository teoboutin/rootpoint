# Slice: assets at paths inside a root, and the session's store

The first slice of `milestone@plans@asset-storage`. It lands the store of
`thread@asset-storage@editing-modes` and the asset paths of `thread@asset-storage@asset-paths-only`,
with no pack, no lock and no socket: the native window and the browser build keep their present
actions, rewired onto the new commands.

## Builds

- `Asset { path: String }` in `path@rootpoint-model@src/document.rs`; `AssetSource` and its two
  variants leave. `Op::SetAssetSource` becomes `Op::SetAssetPath { id, path }` in
  `path@rootpoint-model@src/ops.rs`. `FORMAT_VERSION` goes from 3 to 4, and the four committed
  `.ron` files are rewritten, as the milestone's "How the work is done" says.
- `asset_path_is_confined`, and `Document::validate` refusing an asset whose path fails it, as a
  structural breach, `design@rootpoint-model@validated-documents`.
- `Store` and `Session::store` in `path@rootpoint-protocol@src/session.rs`; `Session::root` gives
  the workspace's root, or none in memory. `Session::directory` leaves, and with it the directory
  arithmetic of `Session::save` (`normalize`, `inside`, `relative` of
  `path@rootpoint-protocol@src/save.rs`, as far as nothing else uses them) and the stack rewrite of
  `design@rootpoint-protocol@save-updates-sources`.
- `Session::asset_bytes` reads from the map, or from the host's files at the root joined with the
  asset path; in a workspace, a path whose file is reached through a symlink that leads out of the
  root is refused. `Files::list(dir) -> Result<Vec<String>, String>` is added, for `open-copy`; the
  native implementations and `MemoryFiles` implement it.
- The commands `open <dir>`, `open-copy <dir>`, `import <host-path> [<asset-path>]` and `save` with
  no argument, by `thread@asset-storage@editing-modes`'s table; `load`, `save <path>` and `asset`
  leave; `new` and `sample` open in memory, the sample's images in the map.
- In the window: Open and Save of `path@rootpoint-app@src/actions.rs` call `open` and `save`; the
  native Open asks for a directory; `App::relative_if_absolute` leaves, since no path the session
  holds depends on the working directory. `App::open_directory` of the browser build runs
  `open-copy` over the picked files, and `App::add_picked_image` runs `import`. `Assets::refresh` of
  `path@rootpoint-app@src/assets.rs` keys on the store and the asset paths.
- `rootpoint-headless` runs `open`, `save` and `import` through its native files.

## Claims

- **A file round-trips in the new format.** The fixture's round-trip test passes after its
  rewrite with `ROOTPOINT_BLESS=1`; shown to fail by loading a file of version 3, which is
  refused.
- **The reproduction of the closed save-mixes defect keeps its paths.** Carried from the issue
  this milestone closed: in a workspace `D` holding `img/cat.png`, `open D`, `add-slide`,
  `import D/img/cat.png img/cat.png`, `image n2 a3 fit`, `save`, then `open <absolute D>` and
  `save`: `presentation.ron` holds `path: "img/cat.png"` and no `D/cat.png` exists. A protocol test
  over `MemoryFiles` runs it with a relative and an absolute form of `D`. Shown to fail by a store
  that joins the root by text with no single form, as the old code did.
- **Confinement**, AC2's first half, `acceptance@asset-storage@ac-confined-paths`: `load_ron`
  refuses a file whose asset path is absolute, holds `..`, starts with `.rootpoint` or is
  `presentation.ron`; `import` to such an asset path replies `err`; a read through a symlink out of
  the root replies `err`. One test per case; shown to fail with the check removed.
- **`open-copy` reads every asset of the presentation into the map**, and then no command reads the
  host's files for an asset: a test opens a copy, removes the files from the host, and renders.
- **`import` copies from outside the root**, at the root under its name, with `-2` before the
  extension when the name holds other bytes, and uses the same name when it holds the same bytes.
- **The sample opens in memory, and a `save` there replies `err` naming `pack` and `unpack`** until
  #pack-commands; a test checks the reply.
- **No command rewrites an asset path when the store changes**: the property test of
  `path@rootpoint-protocol@tests/properties.rs` gains `open`, `open-copy` and `import`, and checks
  that every asset path in the document and the stacks is confined after each command.
- **The window's actions match a second session**, in `path@rootpoint-app@tests/actions.rs`, for
  Open, Save and adding an image, natively and in the wasm test.

## Fixtures

The four committed `.ron` files, rewritten to version 4 with `path:` for each asset. The sample's
images stay files of `path@rootpoint-protocol@sample/`, included at build time; `sample` puts them
in the map under their file names.

## Audit subjects

- `path@rootpoint-protocol@src/session.rs`, `path@rootpoint-protocol@src/save.rs`,
  `path@rootpoint-protocol@src/files.rs`, `path@rootpoint-protocol@src/command.rs`,
  `path@rootpoint-protocol@src/sample.rs`.
- `path@rootpoint-model@src/document.rs`, `path@rootpoint-model@src/ops.rs`,
  `path@rootpoint-model@src/validate.rs`, `path@rootpoint-model@src/file.rs`.
- `path@rootpoint-app@src/actions.rs`, `path@rootpoint-app@src/assets.rs`,
  `path@rootpoint-app@src/memory.rs`, `path@rootpoint-app@src/main.rs`.
- `design@rootpoint-protocol@session-files`, `design@rootpoint-protocol@asset-save-rule`,
  `design@rootpoint-protocol@save-updates-sources`, `design@rootpoint-protocol@builtin-sample`,
  `design@rootpoint-model@asset-source`, `design@rootpoint-app@picked-files-by-path`.
- D2 of the milestone, the reading of confinement, which blocks this slice.

## Fails alone on

- The fixture's round trip, and the reproduction of the save-mixes defect.
- A confinement test that loads, imports or reads outside the root.

## Premises that expire

- A `save` in memory replies `err` until #pack-commands gives it a pack to write; #pack-commands ends it, and its
  claim that `save` after `open-pack` writes the pack watches it.
- The native window's Open asks for a directory and opens it as a workspace with no lock until
  #workspace-host; two windows may open one directory in between, as two windows may open one file
  today.

## Decided design

The milestone's sections on `thread@asset-storage@editing-modes`,
`thread@asset-storage@asset-paths-only` and `thread@asset-storage@asset-tree-shape` hold the shape.
This slice builds the store and the paths; the trash, `rename` and `remove` of `Files` wait for
#asset-commands.

## Acceptance criteria

This slice judges the first half of `acceptance@asset-storage@ac-confined-paths`: the RON, `import`
and a read through a symlink.

## Harvest

- Rewrite `design@rootpoint-protocol@session-files` to the store, citing the goals it did; record
  the decision of `thread@asset-storage@editing-modes` in it or in a head of its own, by the tests
  of `primer@design-heads`.
- Reverse `design@rootpoint-model@asset-source` to `Asset { path }` and confinement, with a
  `rejected` entry for the bytes embedded in the RON, its reason #arg-1's measurement.
- Reverse `design@rootpoint-protocol@asset-save-rule` into the import's rule, and remove
  `design@rootpoint-protocol@save-updates-sources`, with a `rejected` entry for a free save path if
  it meets a test of `skill@knowledge-architect-decision-recording`.
- Rewrite `design@rootpoint-protocol@builtin-sample` and `design@rootpoint-app@picked-files-by-path`
  to the map, and reword `design@rootpoint-render@caller-supplies-assets`'s premise to a root.
- Update the texts the milestone's table lists for these heads: the lines of
  `register@rootpoint-protocol@design`, `register@rootpoint-app@design`,
  `register@rootpoint@design`, `path@rootpoint-protocol@CLAUDE.md`,
  `path@rootpoint-render@CLAUDE.md`, and the comments of `path@rootpoint-protocol@src/files.rs`,
  `path@rootpoint-app@src/actions.rs` and `path@rootpoint-app@src/memory.rs`.
- Update `path@rootpoint-protocol@README.md` and `path@rootpoint-app@README.md` for the new
  commands and Open.
