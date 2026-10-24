# Slice: packs

The second slice of `milestone@plans@asset-storage`. It lands `thread@asset-storage@pack-container`:
writing a pack, opening one in memory, and unpacking one into a workspace, by commands, natively and
headless. The browser's picker and download wait for #browser-packs.

## Builds

- `path@rootpoint-protocol@src/pack.rs`: `write_pack`, which writes a zip of the entry
  `mimetype`, stored, holding `PACK_MIMETYPE`, then `presentation.ron`, deflated, then each asset of
  the table at its asset path, stored; and `read_pack`, which reads one and refuses a pack whose
  first entry is not that `mimetype`, an entry whose name fails confinement, and a symlink entry.
- `zip` 9.1.0 as a dependency of the protocol crate, its default features off and `deflate-flate2`
  on, beside `flate2` 1.1 named with its feature `rust_backend` alone: zip names `flate2` with its
  default features off, and only its default feature `deflate` would pick a backend, zopfli and
  zlib-rs. `rust_backend` is `miniz_oxide`, pure Rust, already in `Cargo.lock` through `png`. On the
  wasm target zip pulls `getrandom` 0.4 with `wasm_js`; a scratch crate of the same dependencies
  built for `wasm32-unknown-unknown` at the audit.
- `Files::list(dir) -> Result<Vec<String>, String>`, every file under `dir`, recursively, as paths
  relative to it, a default method replying `Err("unsupported")`, implemented by `NativeFiles` of
  both binaries, the app's `MemoryFiles` and the protocol tests' `MemoryFiles`: `unpack` refuses a
  directory that holds anything, and is its first caller, before #asset-commands.
- `mimetype` is no asset path: `asset_path_is_confined` refuses it, as it refuses
  `presentation.ron`, since it is the name of a pack's first entry, and an asset of that name would
  give the zip two entries of one name.
- The commands `pack <file>`, `open-pack <file>` and `unpack <dir>`, by the milestone's table of
  #editing-modes. In memory, `save` writes the pack last opened by `open-pack` or written by `pack`.
  - `Session::pack: Option<String>`, that pack's host path, in memory only: `open-pack` and a `pack`
    in memory set it and reset `Session::unsaved`; a `pack` in a workspace changes neither; a
    command that opens a store drops it. `store` replies `ok memory unsaved=<n> pack=<file>` once it
    is set.
  - `pack` replies `ok missing=<ids>`, the assets it wrote no bytes for, `-` for none, the form of
    `render`'s `missing=`; a `save` that writes the pack replies as `pack` does.
  - `unpack <dir>` refuses a directory that holds anything, by `Files::list`, and a path that is a
    file; it writes `presentation.ron` and each file of the map, then the store is a workspace at
    the directory's canonical path, with no pack and `Session::unsaved` 0. A write that fails leaves
    the files written before it, and the store unchanged.
  - `read_pack` skips a directory entry, which a zip tool writes for each folder, and refuses a pack
    with no `presentation.ron`. `write_pack` gives every entry one fixed time, so a presentation
    packs to the same bytes.
- In the native window, each a dialog and its command, labelled by #c-visible-mode: in memory,
  "Save as pack…" (`pack`), and "Save as workspace…" (`unpack`), shown as "Unpack to…" when a pack is
  the session's target; in a workspace, "Pack to…" (`pack`). Open accepts a `.rpk` file: as a button
  "Open pack…" beside Open, since the native file dialog picks a file or a directory, never either.
  Save (Ctrl+S) in memory with no pack opens "Save as pack…", as Save with no file asked for one.
- `App::open_picked_pack(name, bytes)` in `path@rootpoint-app@src/actions.rs`, built on every
  target: the bytes are held in `MemoryFiles` under the name, and `open-pack` opens them; a refused
  pack leaves the held files as they were. #browser-packs gives it the page's picker.

## Claims

- **A pack round-trips**: `sample`, `pack s.rpk`, `open-pack s.rpk` gives a document equal to the
  sample's, and the same asset bytes; shown to fail by a pack that leaves out an asset.
- **A pack then an unpack elsewhere keeps the tree of assets**, as the closed `pack-and-unpack`
  asked: a workspace whose assets are `path@elsewhere@img/a.png`, `path@elsewhere@img/b/c.png` and `path@elsewhere@fonts/f.ttf` is packed,
  unpacked into another directory, and opened: the same document, the same paths, the same bytes.
- **A pack unzips with a common tool into a workspace that opens to the same presentation**: the
  same fixture, extracted by `unzip` where it is installed, then `open`; the test is skipped,
  saying so, where `unzip` is missing. #c-unpack-anywhere.
- **The pack's first entry is `mimetype`, stored**, read by its offset in the file's first bytes.
- **#c-pack-overhead holds**, for the sample and for the fixture above, its figures printed in the
  commit.
- **Confinement at a pack's opening**, AC2's second half, `acceptance@asset-storage@ac-confined-paths`:
  `open-pack` of a crafted zip with an entry `../x`, an entry `/x`, an entry `.rootpoint/lock` and
  a symlink entry is refused whole, and the store is unchanged; one test per case.
- **`pack` of an asset whose path fails confinement** writes the pack with no bytes for it, names it
  in its reply, and gives the host's files no path outside the root, AC2.
- **`unpack` makes the store a workspace**, refuses a directory that holds a file, and a `save` then
  writes `presentation.ron` there.
- **A pack opens in the wasm test**: a test of `path@rootpoint-app@tests/actions.rs` opens a pack's
  bytes through the app's `MemoryFiles` on the wasm target.

## Fixtures

The workspace of three assets in two levels of folders, written by the test from the sample's
`splits.png` and the built-in font's bytes; no new committed file.

## Audit subjects

- The zip crate's documentation and source for its compression methods, symlink entries and
  wasm support; EPUB's Open Container Format for the `mimetype` entry, the milestone's "Readings".
- `issue@rootpoint@thorough-testing`, for a fuzz lead on the second loader.
- `design@rootpoint@wasm-from-the-start`, `design@rootpoint@crate-existence-rule`.

## Fails alone on

- A pack's round trip, or its extraction by `unzip`.

## Premises that expire

- Packs come from trusted sources, on the owner's word of round 4; a later design for untrusted
  packs ends it, the milestone's "Later consequences".

## Decided design

The milestone's section on `thread@asset-storage@pack-container` holds the shape.

## Acceptance criteria

This slice judges `acceptance@asset-storage@ac-confined-paths` at `pack` and at `open-pack`.

## Harvest

- A head of the protocol for the pack, from `thread@asset-storage@pack-container`, by the tests of
  `primer@design-heads`; `rejected` entries for a binary format of rootpoint's own and for tar,
  #arg-13.
- Update `path@rootpoint-protocol@README.md` for the three commands.
