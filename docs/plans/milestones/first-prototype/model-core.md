# Slice: The document model, its operations and its file

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites.

## Builds

- The crate `rootpoint-model` at `planned@rootpoint@crates/rootpoint-model/`, a member of the
  workspace: the document types, `apply` with its inverses, the area geometry `area_rects`, and the
  RON format with `save_ron` and `load_ron`.
- The gate `wasm` in `path@xtask@src/main.rs`, pushed onto the list `rust_project` returns, with
  `Distiller::WithoutProgress` and `resolves: true`. It runs
  `cargo build --target wasm32-unknown-unknown -p rootpoint-model`.
- Per D2, the manifest declares `crates/rootpoint-model` as a Component, with its documents and
  register homes, and its goals drafted for the owner's ruling.

## Claims

1. **Every operation's inverse restores the document.** For every `Op` variant, applying it then
   applying the returned inverse gives a document equal to the original, ids included. Test: a
   table-driven test over one hand-built document per variant, plus a property test over random
   operation sequences. Shown to fail: an inverse of `MoveDivider` that returns the new position in
   place of the old one fails the test.
2. **A split along its parent's axis merges into the parent.** Splitting a child of a vertical
   split vertically gives one vertical split with one more child, never a nested vertical split.
   Test: unit tests on each side (before, after) and on a split of the root. Shown to fail: removing
   the merge leaves a nested split, which the test's tree comparison reports.
3. **Removing an area gives its space to its neighbours, and a split left with one child
   collapses.** Test: unit tests that read the divider positions and the tree after `Remove`.
   Shown to fail: a removal that keeps a one-child split fails the tree comparison.
4. **Area rectangles tile the slide.** For any document, `area_rects` of a slide gives rectangles
   that do not overlap and whose areas sum to the slide's area in slide units, within 1e-6 of a
   unit. Test: a property test over random trees. Shown to fail: an off-by-one in divider indexing
   makes two rectangles overlap.
5. **Saving then loading gives back the same document.** `load_ron(save_ron(d)) == d` for every
   document the property tests generate, and the file holds a `version` field equal to
   `FORMAT_VERSION`. Shown to fail: a field marked `#[serde(skip)]` breaks equality.
6. **The model builds for the wasm target.** The `wasm` gate passes. Shown to fail: a temporary
   `std::fs` call in a non-gated module makes the gate fail; reverted before the commit.

## Fixtures

A small document built in code, in the test module: one slide, a vertical split into an image area
and a horizontal split of two text areas. It uses only the names of "New names, in one place" of
the milestone document. A RON file of the same document is written by the test for AC5.

## Audit subjects

- `path@xtask@src/main.rs`, and the `Gate` struct of `knowledge-architect-gates` 0.8.0 in the cargo
  registry.
- `thread@first-prototype@split-tree-arity`, `thread@first-prototype@split-any-node`,
  `thread@first-prototype@edit-operation-core`, `thread@first-prototype@inline-content-assets`,
  `thread@first-prototype@asset-source`, `thread@first-prototype@save-format`,
  `thread@first-prototype@protocol-addressing`, `thread@first-prototype@slide-coordinate-space`.
- The crate documentation of `ron` and `serde` at the versions chosen, for enum representation and
  byte arrays (`AssetSource::Embedded`).

## Fails alone on

- A failure of claims 1 to 5 is this slice's: no other crate exists yet.
- The `wasm` gate failing on `rootpoint-model` points at a dependency of this crate.

## Premises that expire

- No crate renders or drives the model yet, so `Content` holds only `Empty`, `Text` and `Image`. The
  later content kinds add variants, per "Later consequences" of the milestone document.
- Only `AssetSource::Path` is written by any code of the prototype. Packing ends this; claim 5 then
  covers `Embedded` too.

## Implementation sequence

1. **The empty crate and the wasm gate.** Add the crate with an empty `Document`, add it to the
   workspace, install the wasm target (`rustup target add wasm32-unknown-unknown`), push the `wasm`
   gate, and declare the Component per D2. Fails alone on: the gate, or the checker's Component
   findings.
2. **The tree and its operations.** `Document`, `Slide`, `Node`, ids, `Op`, `apply` and the
   inverses. Claims 1 to 3.
3. **The geometry.** `area_rects`. Claim 4.
4. **The file.** `save_ron`, `load_ron`, `FORMAT_VERSION`. Claim 5; then AC5.

## Decided design

### The tree

A slide holds one root node. A node is a split or an area. A split has an axis and k ≥ 2 children,
with k−1 divider positions, each a fraction of the split's extent in increasing order. A split
whose new child would have the split's own axis inserts the new child into it instead
(`thread@first-prototype@split-tree-arity`). Any node, an area or a split, can be split: the node is
wrapped in a new split of the other axis, or, on the same axis as its parent, gains a sibling, which
gives every guillotine layout (`thread@first-prototype@split-any-node`).

Nearest rival: binary splits; see "Losing alternatives" of the milestone document.

### Ids

Every slide and every node carries an id that never changes for the life of the document and is
stored in the file. Ids are allocated from a counter the document stores, and never reused, so a
removed node's id never names another node (`thread@first-prototype@protocol-addressing`).

### Content and assets

An area holds its content: `Empty`, `Text(TextContent)` or `Image(ImageContent)`. `TextContent`
holds the text and a `TextStyle`: font (a built-in font name or an asset id), size in slide units,
bold, italic, colour as RGBA, horizontal and vertical alignment
(`thread@first-prototype@text-style-per-area`). `ImageContent` holds an asset id and an `ImageFit`,
`Fit` or `Fill`. The document holds an asset table from `AssetId` to `Asset`, whose `AssetSource` is
`Path(String)`, a path relative to the presentation file, or `Embedded(Vec<u8>)`; this slice writes
`Path` only (`thread@first-prototype@inline-content-assets`, `thread@first-prototype@asset-source`,
`thread@first-prototype@image-storage`).

### Operations

`apply(&mut Document, Op) -> Result<Op, OpError>` changes the document and returns the operation
that undoes it; an error leaves the document unchanged. The GUI, the protocol and the tests call
nothing else to change a document (`thread@first-prototype@edit-operation-core`).

### The file

RON, through serde, with a top-level `version` field equal to `FORMAT_VERSION`, 1 in this slice. A
file of another version is refused with an error naming both versions
(`thread@first-prototype@save-format`).

## Mapping tables

Each operation to its inverse. The variant list is the shape the design needs; the audit may refine
it in place.

| `Op` | its inverse |
| --- | --- |
| `AddSlide { at }` | `RemoveSlide { slide }` of the new slide |
| `RemoveSlide { slide }` | `RestoreSlide { at, slide }` holding the removed slide whole |
| `RestoreSlide { at, slide }` | `RemoveSlide { slide }` |
| `MoveSlide { slide, to }` | `MoveSlide { slide, to: its former index }` |
| `DuplicateSlide { slide }` | `RemoveSlide { slide }` of the copy |
| `Split { node, axis, at, side }` | `Remove { node }` of the new area, with the merge undone where it happened |
| `Remove { node }` | `Restore { parent, index, node, dividers }` holding the removed subtree and the former divider positions |
| `Restore { parent, index, node, dividers }` | `Remove { node }` |
| `MoveDivider { split, index, pos }` | `MoveDivider { split, index, pos: the former position }` |
| `Swap { a, b }` | `Swap { a, b }` |
| `SetContent { area, content }` | `SetContent { area, content: the former content }` |
| `AddAsset { asset }` | `RemoveAsset { id }` of the new asset |
| `RemoveAsset { id }` | `AddAsset` with the removed asset and its id |

## Acceptance criteria

This slice judges AC5, `acceptance@first-prototype@file-editable-by-agent`, after step 4: a fresh
agent session, given only the RON file of the fixture, changes one text and one divider position by
editing the file, and the test loads it back with those changes.

## Harvest

At this slice's landing, under `skill@knowledge-architect-decision-recording`, judged by the entry
tests of `primer@design-heads`, into the design home of the model crate's Component:

- the threads `thread@first-prototype@split-tree-arity`, `thread@first-prototype@split-any-node`,
  `thread@first-prototype@slide-coordinate-space`, `thread@first-prototype@inline-content-assets`
  with `thread@first-prototype@asset-source`, `thread@first-prototype@edit-operation-core`,
  `thread@first-prototype@save-format`, `thread@first-prototype@image-storage`,
  `thread@first-prototype@text-style-per-area`, and the ids of
  `thread@first-prototype@protocol-addressing`;
- `thread@first-prototype@crate-existence-rule`, into rootpoint's design home, attributed to the
  owner;
- the losing alternatives of these threads, judged under
  `skill@knowledge-architect-decision-recording` for a `rejected` entry: binary splits, content by
  reference, pixel units, JSON;
- T1 into that Component's tripwires home, and T3 and T4 into rootpoint's, on the owner's word.
