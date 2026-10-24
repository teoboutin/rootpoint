# Slice: packs

The second slice of `milestone@plans@asset-storage`. It lands `thread@asset-storage@pack-container`:
writing a pack, opening one in memory, and unpacking one into a workspace, by commands, natively and
headless. The browser's picker and download wait for #browser-packs.

## Builds

- `planned@rootpoint-protocol@src/pack.rs`: `write_pack`, which writes a zip of the entry
  `mimetype`, stored, holding `PACK_MIMETYPE`, then `presentation.ron`, deflated, then each asset of
  the table at its asset path, stored; and `read_pack`, which reads one and refuses a pack whose
  first entry is not that `mimetype`, an entry whose name fails confinement, and a symlink entry.
- A zip crate as a dependency of the protocol crate, pure Rust, built by the gate `wasm`, chosen at
  the audit. `zip` 0.6.6 is in the local cargo registry at the time of writing; the audit reads the
  current version and its features, and enables no feature that links C code.
- The commands `pack <file>`, `open-pack <file>` and `unpack <dir>`, by the milestone's table of
  #editing-modes. In memory, `save` writes the pack last opened by `open-pack` or written by `pack`.
- In the native window, each a dialog and its command, labelled by #c-visible-mode: in memory,
  "Save as pack…" (`pack`), and "Save as workspace…" (`unpack`), shown as "Unpack to…" when a pack is
  the session's target; in a workspace, "Pack to…" (`pack`). Open accepts a `.rpk` file.

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

This slice judges the second half of `acceptance@asset-storage@ac-confined-paths`, at `open-pack`.

## Harvest

- A head of the protocol for the pack, from `thread@asset-storage@pack-container`, by the tests of
  `primer@design-heads`; `rejected` entries for a binary format of rootpoint's own and for tar,
  #arg-13.
- Update `path@rootpoint-protocol@README.md` for the three commands.
