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
- `asset_path_is_confined`, by the milestone's "Decided design" of #asset-paths-only, and
  `Document::validate` refusing an asset whose path fails it, as a structural breach, on D4's
  default.
- `Files::canonical(path) -> Result<String, String>` and `Files::list(dir) -> Result<Vec<String>,
  String>`, the second listing every file under `dir`, recursively, as paths relative to it. Both
  are default methods of the trait that reply `Err("unsupported")`, so the 12 test doubles that
  implement `Files` change only where a test needs them; `NativeFiles` of the window and of the
  headless binary, the app's `MemoryFiles` and the protocol tests' `MemoryFiles` implement them.
- `Store`, `Session::store`, `Session::root` and `Session::unsaved` in
  `path@rootpoint-protocol@src/session.rs`. `Session::directory` leaves, and with it the directory
  arithmetic of `Session::save` (`normalize`, `inside` and `relative` of
  `path@rootpoint-protocol@src/save.rs`, as far as nothing else uses them) and the stack rewrite of
  `design@rootpoint-protocol@save-updates-sources`.
- `Session::asset_bytes` reads from the map, or from the host's files at the root joined with the
  asset path, refusing a read whose `Files::canonical` path is not under the root.
- The commands `open <dir>`, `open-copy <dir>`, `import <host-path> [<asset-path>]`, `save` with no
  argument and `store`, by the milestone's table of #editing-modes; `load`, `save <path>` and
  `asset` leave; `new` and `sample` open in memory, the sample's images in the map. A `save` in
  memory replies `err` naming `pack` and `unpack`.
- In the window, every caller of the session's directory and of the removed commands:
  - Open and Save of `path@rootpoint-app@src/actions.rs` run `open` and `save`; the native Open
    asks for a directory. `App::relative_if_absolute` leaves, with its callers `open_file` and
    `save_to`, and with `normalize` and `relative` of actions.rs and their unit tests, since the
    root is canonical.
  - `add_asset_dialog` and `App::add_picked_image` run `import`; `App::open_directory` runs
    `open-copy` over the picked files. Both reset `self.assets` today; `Assets::refresh` of
    `path@rootpoint-app@src/assets.rs` keys instead on the store and the asset paths.
  - The control socket's dispatch of `Load`, `Save`, `New` and `Sample`, in
    `path@rootpoint-app@src/control.rs`, follows the new commands.
  - The outline's label, `font_assets` and `App::asset_name` of `path@rootpoint-app@src/panels.rs`
    read `Asset::path`; `font_assets` no longer finds embedded fonts by their bytes.
- `rootpoint-headless` runs `open`, `save`, `import` and `store` through its native files; its doc
  comment, `path@rootpoint-protocol@src/bin/rootpoint-headless.rs`, says what a host path is relative
  to, `render`'s included.

## Claims

- **A file round-trips in the new format.** The fixture's round-trip test passes after its
  rewrite with `ROOTPOINT_BLESS=1`, run on that test alone, since the variable also rewrites the
  render and browser goldens; shown to fail by loading a file of version 3, which is refused.
- **The save-mixes defect stays closed.** Its text: the milestone's "Names". A test over the native
  files in a temporary directory `D` holding `img/cat.png`, with the working directory set to `D`'s
  parent: `open D`, `add-slide`, `import D/img/cat.png`, `image n2 a<id> fit`, `save`; then
  `open <absolute D>`, `import <absolute D>/img/cat.png`, `save`. `presentation.ron` names
  `path: "img/cat.png"` once, for one asset, and no `D/cat.png` exists. The test runs again with the
  absolute form first. Shown to fail with `open` storing the root as given rather than canonical.
- **Confinement**, AC2's first half, `acceptance@asset-storage@ac-confined-paths`: `load_ron`
  refuses a file whose asset path is absolute, holds `..`, starts with `.rootpoint` or is
  `presentation.ron`; `import` to such an asset path replies `err`; a read through a symlink out of
  the root replies `err`. One test per case; shown to fail with the check removed.
- **`open-copy` reads every asset of the presentation into the map**, and then no command reads the
  host's files for an asset: a test opens a copy, removes the files from the host, and renders.
- **`import` copies from outside the root**, at the root under its name, with `-2` before the
  extension when the name holds other bytes, and the same name when it holds the same bytes.
- **`store` replies the store and `Session::unsaved`**, which an edit raises and a `save` resets.
- **No command rewrites an asset path when the store changes**: the property test of
  `path@rootpoint-protocol@tests/properties.rs` replaces `asset` by `import`, adds `open-copy`, and
  checks that every asset path in the document and the stacks is confined after each command.
- **The window's actions match a second session**, in `path@rootpoint-app@tests/actions.rs`, for
  Open, Save and adding an image, natively and in the wasm test.

## Fixtures

- The four committed `.ron` files, rewritten to version 4 with `path:` for each asset.
- The sample's images stay files of `path@rootpoint-protocol@sample/`, included at build time;
  `sample` puts them in the map under their file names. The test of
  `path@rootpoint-protocol@tests/commands.rs` that asserts the sample's assets are `Embedded`
  asserts they are in the map.
- The script tests of `path@rootpoint-protocol@tests/scripts/`, `golden.txt`, `ids-build.txt` and
  `ids-readdress.txt`, and every test that runs `load`, `save <path>`, `asset` or `remove-asset`
  (`git grep -lE 'Command::(Load|Save|Asset|RemoveAsset)\b|"(load|save|asset|remove-asset) '`
  lists them), move to the new commands. `remove-asset` stays until #asset-commands replaces it.

## Audit subjects

- `path@rootpoint-protocol@src/session.rs`, `path@rootpoint-protocol@src/save.rs`,
  `path@rootpoint-protocol@src/files.rs`, `path@rootpoint-protocol@src/command.rs`,
  `path@rootpoint-protocol@src/sample.rs`, `path@rootpoint-protocol@src/bin/rootpoint-headless.rs`.
- `path@rootpoint-model@src/document.rs`, `path@rootpoint-model@src/ops.rs`,
  `path@rootpoint-model@src/validate.rs`, `path@rootpoint-model@src/file.rs`.
- `path@rootpoint-app@src/actions.rs`, `path@rootpoint-app@src/assets.rs`,
  `path@rootpoint-app@src/memory.rs`, `path@rootpoint-app@src/main.rs`,
  `path@rootpoint-app@src/control.rs`, `path@rootpoint-app@src/panels.rs`; the uses of
  `AssetSource` in `path@rootpoint-app@tests/web.rs` and `path@rootpoint-render@tests/common/mod.rs`.
- `design@rootpoint-protocol@session-files`, `design@rootpoint-protocol@asset-save-rule`,
  `design@rootpoint-protocol@save-updates-sources`, `design@rootpoint-protocol@builtin-sample`,
  `design@rootpoint-protocol@gesture-undo`, `design@rootpoint-model@asset-source`,
  `design@rootpoint-model@validated-documents`, `design@rootpoint-model@cosmetic-fallbacks`,
  `design@rootpoint-app@picked-files-by-path`.
- D2 and D4 of the milestone, built on their defaults.

## Fails alone on

- The fixture's round trip, and the reproduction of the save-mixes defect.
- A confinement test that loads, imports or reads outside the root.

## Premises that expire

- A `save` in memory replies `err` until #pack-commands gives it a pack to write; #pack-commands ends
  it, and its claim that `save` after `open-pack` writes the pack watches it.
- The native window's Open opens a directory as a workspace with no lock until #workspace-host; two
  windows may open one directory in between, as two windows may open one file today. No criterion
  watches it: no release is made between two slices, and #workspace-host's claims end it.

## Decided design

The milestone's sections on `thread@asset-storage@editing-modes`,
`thread@asset-storage@asset-paths-only` and `thread@asset-storage@asset-tree-shape` hold the shape.
This slice builds the store, the paths and `store`; the trash, `rename` and `remove` of `Files` wait
for #asset-commands.

## Acceptance criteria

This slice judges the first half of `acceptance@asset-storage@ac-confined-paths`: the RON, `import`
and a read through a symlink.

## Harvest

- Rewrite `design@rootpoint-protocol@session-files` to the store, citing the goals it did; record
  the decision of `thread@asset-storage@editing-modes` in it or in a head of its own, by the tests
  of `primer@design-heads`.
- Reverse `design@rootpoint-model@asset-source` to `Asset { path }` and confinement, with a
  `rejected` entry for the bytes embedded in the RON, its reason #arg-1's measurement; widen
  `design@rootpoint-model@validated-documents` by the unconfined path, as D4 rules.
- Reverse `design@rootpoint-protocol@asset-save-rule` into the import's rule, and remove
  `design@rootpoint-protocol@save-updates-sources`, with a `rejected` entry for a free save path if
  it meets a test of `skill@knowledge-architect-decision-recording`.
- Rewrite `design@rootpoint-protocol@builtin-sample` and `design@rootpoint-app@picked-files-by-path`
  to the map; reword `design@rootpoint-render@caller-supplies-assets`'s premise to a root; remove
  `load` from the list of `design@rootpoint-protocol@gesture-undo`.
- Update the texts the milestone's table lists for these heads: the lines of
  `register@rootpoint-protocol@design`, `register@rootpoint-app@design`, `register@rootpoint@design`,
  `path@rootpoint-protocol@CLAUDE.md`, `path@rootpoint-render@CLAUDE.md`, and the comments of
  `path@rootpoint-protocol@src/files.rs` and `path@rootpoint-app@src/memory.rs`;
  `App::relative_if_absolute` and its comment leave with the code.
- Update `path@rootpoint-protocol@README.md` and `path@rootpoint-app@README.md` for the new
  commands and Open.
