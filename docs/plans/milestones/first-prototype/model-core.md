# Slice: The document model, its operations and its file

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites.

## Builds

- The crate `rootpoint-model` at `planned@rootpoint@crates/rootpoint-model/`, a member of the
  workspace: the document types, `apply` with its inverses, the area geometry `area_rects`, and the
  RON format with `save_ron` and `load_ron`. Its library root reads `ROOTPOINT_CHECKOUT`, as
  `path@xtask@CLAUDE.md` requires of every library root added to the workspace.
- The gate `wasm` in `path@xtask@src/main.rs`, pushed onto the list `rust_project` returns, so it
  runs after `test`. `Gate` has no constructor, so the pushed value spells its seven fields: `name`
  `"wasm"`, `program` `"cargo"`, `args` the command below, `envs` empty, `resolves: true` (the gates
  then append `--locked`), `rebased_only: false`, `distiller: Distiller::WithoutProgress`. It runs
  `cargo build --target wasm32-unknown-unknown -p rootpoint-model`.
- Per D2, the manifest declares `crates/rootpoint-model` as a Component, with the documents and
  register homes of `skill@knowledge-architect-setup@component-documents`, and its goals drafted
  under `skill@knowledge-architect-goal-setting` for the owner's ruling.

## Claims

1. **Every operation's inverse restores the document.** For every `Op` variant, applying it then
   applying the returned inverse gives a document equal to the original, ids included, the id
   counter excepted: the counter only grows, so an id is never reused. Test: a table-driven test
   over one hand-built document per variant, plus a property test over random operation sequences,
   comparing with an equality that skips the counter. Shown to fail: an inverse of `MoveDivider`
   that returns the new position in place of the old one fails the test.
2. **No split has a child of its own axis.** Splitting a child of a vertical split vertically, or a
   vertical split itself vertically, gives one vertical split with one more child, never a nested
   vertical split; a removal that collapses a split keeps it so. Test: unit tests on each side
   (before, after), on a split of the root and on a collapse, plus a property test over random
   operation sequences. Shown to fail: removing the merge leaves a nested split, which the test's
   tree comparison reports.
3. **Removing an area gives its extent to one neighbour, and a split left with one child
   collapses.** Test: unit tests that read the divider positions and the tree after `Remove`.
   Shown to fail: a removal that keeps a one-child split fails the tree comparison.
4. **Area rectangles tile the slide.** For any document, `area_rects` of a slide gives rectangles
   that do not overlap and whose areas sum to the slide's area in slide units, within 1e-6 of a
   unit (a test tolerance for floating-point sums, the session's choice). Test: a property test
   over random trees. Shown to fail: an off-by-one in divider indexing makes two rectangles overlap.
5. **Saving then loading gives back the same document.** `load_ron(&save_ron(&d)) == Ok(d)` for
   every document the property tests generate, and the file holds a `version` field equal to
   `FORMAT_VERSION`. A file whose version differs is refused with `LoadError::Version`. Shown to
   fail: a field marked `#[serde(skip)]` breaks equality.
6. **The model builds for the wasm target.** The `wasm` gate passes. Shown to fail: a temporary
   `use std::os::unix::fs::PermissionsExt;` in the crate makes the gate fail, since `std::os::unix`
   exists only for unix targets (read in the toolchain's `library/std/src/os/mod.rs`); reverted
   before the commit. A `std::fs` call would not show it: on `wasm32-unknown-unknown` it compiles
   and fails only at run time.

## Fixtures

A small document built in code, in the test module: one slide, a vertical split into an image area
on the left and a horizontal split of two text areas on the right. It uses only the names of "New
names, in one place" of the milestone document and of "Decided design" below. A test writes its RON
file to the crate's test data directory, committed, for AC5.

## Audit subjects

- `path@xtask@src/main.rs`, `path@xtask@CLAUDE.md`, and the `Gate` struct of
  `knowledge-architect-gates` 0.8.0 in the cargo registry.
- `thread@first-prototype@split-tree-arity`, `thread@first-prototype@split-any-node`,
  `thread@first-prototype@edit-operation-core`, `thread@first-prototype@inline-content-assets`,
  `thread@first-prototype@asset-source`, `thread@first-prototype@save-format`,
  `thread@first-prototype@protocol-addressing`, `thread@first-prototype@slide-coordinate-space`.
- The crate documentation of `ron` and `serde` at the versions chosen, for enum representation and
  byte arrays (`AssetSource::Embedded`), and of the property-testing crate chosen (`proptest` by
  default).

This slice has no Component of its own until its first step declares one, so its grounding reads
rootpoint's and xtask's documents.

## Fails alone on

- A failure of claims 1 to 5 is this slice's: no other crate exists yet.
- The `wasm` gate failing on `rootpoint-model` points at a dependency of this crate.

## Premises that expire

- No crate renders or drives the model yet, so `Content` holds only `Empty`, `Text` and `Image`. The
  later content kinds add variants, per "Later consequences" of the milestone document. Guard: the
  inverse and round-trip property tests of claims 1 and 5 generate every variant, so a new one is
  covered once its generator is added.
- Only `AssetSource::Path` is written by any code of the prototype. Packing ends this. Guard: the
  round-trip test of claim 5 generates `Embedded` assets already, so the format is tested for them
  before any code writes one.

## Implementation sequence

1. **The empty crate and the wasm gate.** Add the crate with an empty `Document`, add it to the
   workspace, install the wasm target on the machine that runs the gates
   (`rustup target add wasm32-unknown-unknown`), push the `wasm` gate, and declare the Component per
   D2. Fails alone on: the gate, or the checker's Component findings.
2. **The tree and its operations.** `Document`, `Slide`, `Node`, ids, `Op`, `apply` and the
   inverses. Claims 1 to 3.
3. **The geometry.** `area_rects`. Claim 4.
4. **The file.** `save_ron`, `load_ron`, `FORMAT_VERSION`, `LoadError`. Claim 5; then AC5.

## Decided design

The shapes below are the design's; the names and signatures may be refined at the audit, in place.

### The tree

A slide holds one root node. A node is a split or an area. A split has an `Axis` and k ≥ 2
children, with k−1 divider positions, each a fraction of the split's extent, strictly increasing
and strictly between 0 and 1. A `Vertical` split has vertical dividers and lays its children side
by side, left to right; a `Horizontal` split has horizontal dividers and stacks its children top to
bottom (`thread@first-prototype@split-tree-arity`).

`Split { node, axis, at, side }` splits any node, an area or a split
(`thread@first-prototype@split-any-node`). `at` is a fraction of the node's own extent along
`axis`, strictly between 0 and 1, and `side` (`Side::Before` or `Side::After`) says whether the new
empty area takes the part before `at` or after it.

- When `axis` differs from the axis of the node's parent, or the node is the root, the node is
  wrapped in a new split of `axis` holding the node and the new area, with one divider at `at`.
- When `axis` equals the axis of the node's parent, the new area is inserted into the parent beside
  the node, and the parent gains one divider at the node's start plus `at` times its extent, in
  fractions of the parent. This is the merge, and the tree stays unique for each layout.
- When the node is itself a split of `axis`, the new area becomes its first child (`Before`) or
  its last (`After`), taking the part of its extent before or after `at`; its former dividers are
  scaled into the rest. Wrapping it would nest two splits of one axis.

`Remove { node }` removes any node but a slide's root, which `SetContent` empties instead. The
removed child's extent goes to the sibling before it, or to the sibling after it when it is the
first child: the divider between them is removed. A split left with one child is replaced by that
child; when that child is a split of its new parent's axis, its children are merged into the
parent, their dividers scaled into the extent it held, so no split has a child of its own axis.

`Swap { a, b }` exchanges the contents of two areas, of the same slide or not; the areas, their ids
and their positions stay.

Nearest rival: binary splits; see "Losing alternatives" of the milestone document.

### Ids

Every slide, node and asset carries an id that never changes for the life of the document and is
stored in the file: `SlideId`, `NodeId` and `AssetId`, printed `s<n>`, `n<n>` and `a<n>`. Ids are
allocated from one counter the document stores, and never reused, so a removed node's id never names
another node. `DuplicateSlide` gives every node of the copy a new id
(`thread@first-prototype@protocol-addressing`).

Nearest rival: addressing by position; see "Losing alternatives" of the milestone document,
addressing by paths.

### Content and assets

An area holds its content: `Empty`, `Text(TextContent)` or `Image(ImageContent)`.

- `TextContent` holds the text and a `TextStyle`: `font: FontRef` (`FontRef::Builtin(String)`, a
  built-in font name, or `FontRef::Asset(AssetId)`), `size` in slide units, `bold`, `italic`,
  `colour: Rgba` (four `u8`), `h_align: HAlign` (`Left`, `Center`, `Right`) and `v_align: VAlign`
  (`Top`, `Middle`, `Bottom`) (`thread@first-prototype@text-style-per-area`).
- `ImageContent` holds an `AssetId` and an `ImageFit`: `Fit` shows the whole image, scaled to fit
  inside the area and centred; `Fill` covers the whole area, scaled to cover it, centred and cropped.
- The document holds an asset table from `AssetId` to `Asset`, which holds one `AssetSource`:
  `Path(String)`, a path relative to the presentation file, or `Embedded(Vec<u8>)`. This slice
  writes `Path` only (`thread@first-prototype@inline-content-assets`,
  `thread@first-prototype@asset-source`, `thread@first-prototype@image-storage`).

`AddSlide { at }` inserts at index `at` a slide whose root is one empty area. `RemoveAsset` is
refused while an area uses the asset.

Nearest rival: content stored apart and referenced by areas; see "Losing alternatives" of the
milestone document.

### Geometry

The document holds an `AspectRatio { width, height }`, 16:9 by default. A slide's extent in slide
units is 1080 high and 1080 × width / height wide, so 1920×1080 for 16:9.
`area_rects(&Slide, AspectRatio) -> Vec<(NodeId, Rect)>` gives every area's rectangle,
`Rect { x, y, w, h }` in slide units as `f64`, in tree order.

### Operations

`apply(&mut Document, Op) -> Result<Op, OpError>` changes the document and returns the operation
that undoes it; an error leaves the document unchanged. The GUI, the protocol and the tests call
nothing else to change a document (`thread@first-prototype@edit-operation-core`). `OpError` names
the cause: an unknown id, a node of the wrong kind (an area where a split is needed, or the
reverse), an attempt to remove a root, an index out of range, a divider position out of order or
out of (0, 1), an asset in use.

Nearest rival: the GUI mutating the document directly; see "Losing alternatives" of the milestone
document.

### The file

`save_ron(&Document) -> String` and `load_ron(&str) -> Result<Document, LoadError>`: RON, through
serde, with a top-level `version` field equal to `FORMAT_VERSION`, 1 in this slice. `LoadError` is
`Parse` with the parser's message, or `Version { found, expected }`
(`thread@first-prototype@save-format`).

Nearest rival: JSON; see "Losing alternatives" of the milestone document.

## Mapping tables

Each operation to its inverse. A structural edit of a slide's tree has as inverse a `SetTree` that
holds the slide's whole former tree: slides are small, and a snapshot restores merges, collapses and
divider positions exactly, which a per-node inverse would have to reconstruct.

| `Op` | its inverse |
| --- | --- |
| `AddSlide { at }` | `RemoveSlide { slide }` of the new slide |
| `RemoveSlide { slide }` | `RestoreSlide { at, slide }` holding the removed slide whole |
| `RestoreSlide { at, slide }` | `RemoveSlide { slide }` |
| `MoveSlide { slide, to }` | `MoveSlide { slide, to: its former index }` |
| `DuplicateSlide { slide }` | `RemoveSlide { slide }` of the copy |
| `Split { node, axis, at, side }` | `SetTree { slide, root }` holding the slide's former tree |
| `Remove { node }` | `SetTree { slide, root }` holding the slide's former tree |
| `SetTree { slide, root }` | `SetTree { slide, root }` holding the tree it replaced |
| `MoveDivider { split, index, pos }` | `MoveDivider { split, index, pos: the former position }` |
| `Swap { a, b }` | `Swap { a, b }` |
| `SetContent { area, content }` | `SetContent { area, content: the former content }` |
| `AddAsset { asset }` | `RemoveAsset { id }` of the new asset |
| `RemoveAsset { id }` | `RestoreAsset { id, asset }` holding the removed asset |
| `RestoreAsset { id, asset }` | `RemoveAsset { id }` |

## Acceptance criteria

This slice judges AC5, `acceptance@first-prototype@file-editable-by-agent`, after step 4. The
session implementing the slice dispatches a fresh subagent, given the committed RON file of the
fixture and the brief "change the text of the top-right area to `Edited` and move the divider of
the vertical split to 0.4, by editing this file". A test in the crate then loads the edited file,
committed beside the original, and asserts those two changes and no other. The landing commit
reports the result.

## Harvest

At this slice's landing, under `skill@knowledge-architect-decision-recording`, judged by the entry
tests of `primer@design-heads`, into the design home of the model crate's Component:

- the threads `thread@first-prototype@split-tree-arity`, `thread@first-prototype@split-any-node`,
  `thread@first-prototype@inline-content-assets` with `thread@first-prototype@asset-source`,
  `thread@first-prototype@edit-operation-core`, `thread@first-prototype@save-format`,
  `thread@first-prototype@text-style-per-area`;
- `thread@first-prototype@image-storage`, attributed to the owner, keeping the scope and the hedge
  of the owner's answer (shape 1 "for prototyping"; shape 1 the preferred editing form and shape 3
  the final goal, as the owner thinks), with a title in the present tense;
- of `thread@first-prototype@slide-coordinate-space`, the slide units and the area geometry; its
  uniform scaling of every display is harvested by `spec@first-prototype@editor-shell`;
- of `thread@first-prototype@protocol-addressing`, the rule that ids are stored and never reused,
  which the protocol crate's head cites;
- `thread@first-prototype@crate-existence-rule`, into rootpoint's design home, attributed to the
  owner, with the hedge of the owner's words: a rule "something along the line of" the quoted
  sentence;
- the losing alternatives of these threads, judged under
  `skill@knowledge-architect-decision-recording` for a `rejected` entry: binary splits, content by
  reference, pixel units, a free aspect ratio with reflow, mutation of the document by the GUI,
  JSON;
- T1 into the model crate's Component's tripwires home;
- T3 and T4, which watch parked threads and so guard no recorded decision, as the triggers of two
  `deferred` issue entries in the model crate's Component's issue directory, as
  `skill@knowledge-architect-decision-recording@ruled-tripwires` says of a parked thread's tripwire.
