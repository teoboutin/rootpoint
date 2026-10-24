# Slice: packs

The second slice of `milestone@plans@asset-storage`. It lands `thread@asset-storage@pack-container`:
writing a pack, opening one in memory, and unpacking one into a workspace, by commands, natively and
headless. The browser's picker and download wait for #browser-packs.

## Builds

- `planned@rootpoint-protocol@src/pack.rs`: `pack`, which writes a zip of the entry `mimetype`, stored,
  holding `PACK_MIMETYPE`, then `presentation.ron`, deflated, then each asset of the table at its
  asset path, stored; and `unpack`, which reads one and refuses a pack whose first entry is not that
  `mimetype`, an entry whose name fails confinement, and a symlink entry.
- A zip crate as a dependency of the protocol crate, pure Rust, built by the gate `wasm`, chosen at
  the audit. `zip` 0.6.6 is in the local cargo registry at the time of writing; the audit reads the
  current version and its features, and enables no feature that links C code.
- The commands `pack <file>`, `open-pack <file>` and `unpack <dir>`, by the milestone's table of
  `thread@asset-storage@editing-modes`. In memory, `save` writes the pack last opened by `open-pack`
  or written by `pack`, and replies `err` when there is none.
- In the native window: "Save as pack…" and "Unpack to…" in memory, "Pack to…" in a workspace, each
  a file or directory dialog and its command; Open accepts a `.rpk` file.

## Claims

- **A pack round-trips**: `sample`, `pack s.rpk`, `open-pack s.rpk` gives a document equal to the
  sample's, and the same asset bytes; shown to fail by a pack that leaves out an asset.
- **A pack unzips with a common tool into a workspace that opens to the same presentation**: a test
  that runs where `unzip` is installed extracts a pack and runs `open` on the directory; the test is
  skipped, saying so, where `unzip` is missing. #c-unpack-anywhere.
- **The pack's first entry is `mimetype`, stored**, read by its offset in the file's first bytes.
- **The overhead stays within 1%** of the assets' bytes for the sample and for a test presentation
  of one asset of 1 MB, #c-pack-overhead; the measurement is printed in the commit.
- **Confinement at unpack**, AC2's second half, `acceptance@asset-storage@ac-confined-paths`: a
  crafted zip with an entry `../x`, an entry `/x`, an entry `.rootpoint/lock` and a symlink entry is
  refused whole, and nothing is written; one test per case.
- **`unpack` makes the store a workspace**, and a `save` then writes `presentation.ron` there.
- **A pack opens in the wasm test**: a test of `path@rootpoint-app@tests/actions.rs` opens a pack's
  bytes through `MemoryFiles` on the wasm target.

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

This slice judges the second half of `acceptance@asset-storage@ac-confined-paths`, at `unpack`.

## Harvest

- A head of the protocol for the pack, from `thread@asset-storage@pack-container`, by the tests of
  `primer@design-heads`; `rejected` entries for a binary format of rootpoint's own and for tar,
  #arg-13.
- Update `path@rootpoint-protocol@README.md` for the three commands.
