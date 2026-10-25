# Slice: the text tree in the model, the file and the renderer

The first slice of `milestone@plans@rich-text`. It lands `thread@rich-text@text-node-tree`,
`thread@rich-text@text-tree-normal-form`, `thread@rich-text@span-style-overrides`, the model's half
of `thread@rich-text@text-node-ids`, `thread@rich-text@inline-math-node`, the model's half of
`thread@rich-text@split-at-caret` and of `thread@rich-text@named-span-styles`.

## Builds

- In `path@rootpoint-model@src/document.rs`: `TextNode`, `TextItem`, `Span`, `SpanId`,
  `SpanChoice`, `SpanStyle`, `CaretPos` and `Document::span_styles`, as the milestone's "New names,
  in one place" illustrates; `TextContent` holds a root `TextNode` in place of its string. A helper
  that gives a text's plain string, for the places that read it today, the outline's row among
  them.
- In `path@rootpoint-model@src/ops.rs`: the operations of the tree, `Op::InsertText`,
  `Op::DeleteText`, `Op::WrapSpan`, `Op::UnwrapSpan`, `Op::SetSpanStyle`, `Op::SplitSpan`, and of
  the span style table, each returning its inverse; the normal form applied by `apply`, per the
  milestone's "Decided design" of #text-tree-normal-form.
- In `path@rootpoint-model@src/validate.rs`: a span id unique and below the counter; a span naming
  a span style the table lacks, or a font asset the table lacks, among the cosmetic breaches.
- In `path@rootpoint-model@src/file.rs`: `FORMAT_VERSION` 5, and the four committed `.ron` files
  rewritten, as the milestone's "How the work is done" says.
- In `path@rootpoint-render@src/text.rs` and `path@rootpoint-render@src/lib.rs`: the tree laid out
  in one Parley layout, each span's values pushed for its range, the colour carried by each run and
  drawn per run; `AreaText` gains the map between caret positions and byte indices, and each span's
  byte range.
- The first step lands the types and an empty map with no span in any file, and measures a render
  of the sample against `tripwire@rootpoint-render@render-within-bounds`'s bound before and after.

## Claims

- **A text tree round-trips through its file**: the fixture's test
  `the_fixture_round_trips_and_matches_its_committed_file` of `path@rootpoint-model@tests/`, its
  fixture given a styled text with a nested span and a named span; shown to fail by a writer that
  drops a span's id.
- **Every tree operation's inverse restores the document**: a property test over random trees and
  random operations, which applies each and its inverse and compares; shown to fail by an inverse
  of `Op::SplitSpan` that keeps the second id.
- **The normal form holds after every operation**: the same property test checks that no span sets
  nothing, no span is empty but the one an operation has just opened, and equal siblings are kept.
- **A styled text draws as its spans say**: a golden image of a text with a bold span and a
  coloured span, through `render_slide`, under `design@rootpoint-render@portable-precision`.
- **The map places every caret position on its byte index**: a test over a nested tree.

## Audit subjects

- `path@rootpoint-model@src/document.rs`, `path@rootpoint-model@src/ops.rs`,
  `path@rootpoint-model@src/validate.rs`, `path@rootpoint-model@src/file.rs`,
  `path@rootpoint-render@src/text.rs`, `path@rootpoint-render@src/lib.rs`, and every reader of
  `TextContent::text` in the protocol and the app, which this slice keeps compiling.
- `design@rootpoint-model@text-style-per-area`, `design@rootpoint-model@named-styles`,
  `design@rootpoint-model@inline-content-assets`, `design@rootpoint-model@stable-ids`,
  `design@rootpoint-model@cosmetic-fallbacks`, `design@rootpoint-model@validated-documents`,
  `design@rootpoint-model@split-tree-arity`, `design@rootpoint-render@area-text-layout`,
  `tripwire@rootpoint-render@render-within-bounds`.
- Parley 0.12.0's `TreeBuilder` and `RangedBuilder`, and how a brush is given per range.

## Fails alone on

- The fixture's round trip, and the golden image of a styled text.

## Premises that expire

- The protocol's `text` command and the editor still send a whole string, which this slice reads
  as a root with no span; #span-commands and #tree-caret end that, and their claims replace the
  tests that rely on it.

## Decided design

The milestone's "Decided design" holds the shape of each thread this slice lands.

## Acceptance criteria

None is judged at this slice.

## Harvest

- Reverse `design@rootpoint-model@text-style-per-area`, its ground the owner's word for the first
  prototype, into a head of the text tree, #text-node-tree, with #text-tree-normal-form,
  #span-style-overrides and #inline-math-node judged by the entry tests of `primer@design-heads`.
- Rewrite `design@rootpoint-model@named-styles` (#span-style-overrides, #named-span-styles),
  `design@rootpoint-model@inline-content-assets`, `design@rootpoint-model@stable-ids` and
  `design@rootpoint-model@cosmetic-fallbacks`, and update the texts the milestone's table lists
  for them.
- Rewrite `design@rootpoint-render@area-text-layout` for the map and the colour per run.
- The losing alternatives of these threads, judged under
  `skill@knowledge-architect-decision-recording@losing-alternatives`.
- Write T3, guarding the head of #text-node-ids.
