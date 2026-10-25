# Slice: the text tree in the model, the file and the renderer

The first slice of `milestone@plans@rich-text`. It lands `thread@rich-text@text-node-tree`,
`thread@rich-text@text-tree-normal-form`, `thread@rich-text@span-style-overrides`, the model's half
of `thread@rich-text@text-node-ids`, `thread@rich-text@inline-math-node`, the model's half of
`thread@rich-text@split-at-caret` and of `thread@rich-text@named-span-styles`.

## Builds

- In `path@rootpoint-model@src/document.rs`: `TextNode`, `TextItem`, `Span`, `SpanId`,
  `SpanChoice`, `SpanStyle`, `TextNodeRef`, `CaretPos` and `Document::span_styles`, as the
  milestone's "New names, in one place" illustrates; `TextContent` holds a root `TextNode` in place
  of its string. A helper that gives a text's plain string, for the places that read it today, the
  outline's row among them.
- In `path@rootpoint-model@src/ops.rs`: the operations of the tree and of the span style table, by
  the milestone's mapping tables, each returning its inverse; span ids allocated by `apply`;
  `renumber`, which gives a duplicated slide's nodes new ids, giving its spans new ids too;
  `asset_in_use` counting a span's and a span style's font; `span_style_users` beside `users`; the
  canonical form of character runs, and the refusals of #text-tree-normal-form.
- In `path@rootpoint-model@src/validate.rs`: a span id unique in the document and below the
  counter, and a span's or a span style's font asset id below the counter, structural; a span
  naming a span style the table lacks, `Breach::MissingSpanStyle`, and a span's font asset the
  table lacks, cosmetic.
- In `path@rootpoint-model@src/file.rs`: `FORMAT_VERSION` 5, `span_styles` among the fields the
  file lists, and the committed `.ron` files rewritten, as the milestone's "How the work is done"
  says, with the two tests that hold the format as text.
- In `path@rootpoint-render@src/text.rs` and `path@rootpoint-render@src/lib.rs`: the tree laid out
  in one Parley layout, each span's values pushed for its range, the colour carried by each run as
  a brush in place of `()` in `LayoutContext`, `TextLayout` and `AreaText`, and drawn per run;
  `AreaText`'s map, from a caret position to its byte index, and each span's byte range; the
  report of a span's missing span style, and of its font fallback.
- The fonts a span names are loaded with the area's: `Assets::refresh` of
  `path@rootpoint-app@src/assets.rs` and the headless render of
  `path@rootpoint-protocol@src/session.rs` fill the `FontSet` from each span's font too.
- A span's and a span style's font count as uses of their asset wherever the model's
  `asset_in_use` counts them: the session's `asset_uses` and `renaming_ops`, so `assets uses=`,
  `asset-remove` and `asset-remove … into=`, and the Assets tab's `App::asset_uses`; a file edited
  by hand holds such a span before any command makes one. Brought into this slice by its review,
  from #span-commands.
- The protocol and the app keep compiling: every reader of `TextContent::text`, and the app's
  readers of `parley::Layout<()>` in `path@rootpoint-app@src/editing.rs`. The `text` command and the
  editor keep sending a whole string, read as a root with no span, until #span-commands and
  #tree-caret.

## Steps

1. The types, with a root of one run of characters and no span in any file, `FORMAT_VERSION` 5,
   the files rewritten; the identity map from an offset of the root to its byte index. Measure
   `time_the_mixed_slide_at_4k` of `path@rootpoint-render@tests/text.rs` before and after, by the
   tripwire's command in a release build on the owner's machine, and write the figures in the
   step's commit message. Fails alone on: the fixture's round trip, unchanged in content.
2. The operations of the tree, their inverses and the normal form, with the property test. Fails
   alone on: the property test.
3. The span style table: its operations, its users, its breaches. Fails alone on: the tests of a
   removal with a replacement and its inverse.
4. The renderer: spans laid out and drawn, the brush, the map with spans, the reports, the fonts
   loaded by the hosts; the mixed slide given a styled text, and `time_the_mixed_slide_at_4k`
   measured again. Fails alone on: the golden image of a styled text.

## Claims

- **A text tree round-trips through its file**: the fixture's test
  `the_fixture_round_trips_and_matches_its_committed_file` of `path@rootpoint-model@tests/file.rs`,
  its fixture given a styled text with a nested span and a named span, and blessed again; shown to
  fail by a writer that drops a span's id.
- **Every tree operation's inverse restores the document**: a property test over random trees and
  random operations, in `path@rootpoint-model@tests/ops.rs`, which applies each and its inverse and
  compares with `Document::same_content`, since an operation moves the counter; shown to fail by an
  inverse snapshot taken after the edit, for every operation or for `Op::SplitSpan` alone.
- **The normal form holds after every operation**: the same property test checks that no custom
  span sets no field, and that no two runs of characters are adjacent and none is empty; unit tests
  check that equal sibling spans are kept through an edit, and that a named span whose values equal
  its parent's is kept.
- **A styled text draws as its spans say**: a golden image of a text with a bold span and a
  coloured span, through `render_slide`, under `design@rootpoint-render@portable-precision`.
- **The map places every caret position on its byte index**: a test that walks every caret
  position of a nested tree against an independent count, and checks each span's byte range.
- **A span's missing style and font are named**: a test of `Document::breaches` and of the render's
  report.

## Audit subjects

- `path@rootpoint-model@src/document.rs`, `path@rootpoint-model@src/ops.rs`,
  `path@rootpoint-model@src/validate.rs`, `path@rootpoint-model@src/file.rs`,
  `path@rootpoint-render@src/text.rs`, `path@rootpoint-render@src/lib.rs`,
  `path@rootpoint-app@src/assets.rs`, `path@rootpoint-app@src/asset_tab.rs` (its `asset_uses`),
  `path@rootpoint-protocol@src/session.rs` (its render, `asset_uses`, `renaming_ops` and `users`),
  `path@rootpoint-model@tests/invalid.rs`, `path@rootpoint-protocol@tests/breaches.rs`.
- `design@rootpoint-model@text-style-per-area`, `design@rootpoint-model@named-styles`,
  `design@rootpoint-model@inline-content-assets`, `design@rootpoint-model@style-kinds`,
  `design@rootpoint-model@stable-ids`, `design@rootpoint-model@cosmetic-fallbacks`,
  `design@rootpoint-model@validated-documents`, `design@rootpoint-model@split-tree-arity`,
  `design@rootpoint-render@area-text-layout`, `tripwire@rootpoint-render@render-within-bounds`.
- Parley 0.12.0's `TreeBuilder`, with `push_style_modification_span`, `RangedBuilder` and
  `style_run_builder`, and how a brush is given per range; the serde shape that keeps the file
  readable, `criterion@rich-text@c-readable-tree`.

## Fails alone on

- The fixture's round trip, and the golden image of a styled text.

## Premises that expire

- The protocol's `text` command and the editor still send a whole string, which this slice reads
  as a root with no span; editing in the window a text loaded with spans drops them. #span-commands
  and #tree-caret end that; no release is made between the slices, and the tests of the editor in
  `path@rootpoint-app@tests/editing.rs` and `path@rootpoint-app@tests/edit_ops.rs` keep their whole
  strings until #tree-caret replaces them.

## Decided design

The milestone's "Decided design" holds the shape of each thread this slice lands.

## Acceptance criteria

None is judged at this slice.

## Harvest

- Reverse `design@rootpoint-model@text-style-per-area`, its ground the owner's word for the first
  prototype, into a head of the text tree, #text-node-tree, under
  `skill@knowledge-architect-decision-recording@reversal-check`; judge the incumbent, one style per
  area, for `register@rootpoint-model@rejected`, cited by the new head if it earns an entry; rewrite
  the doc comment of `TextContent`.
- Judge by the entry tests of `primer@design-heads`, each a head of its own if it earns one:
  #text-tree-normal-form; #span-style-overrides; #named-span-styles; #inline-math-node, under the
  slug `text-node-kinds`, the pair `#inline-math-node → text-node-kinds` stated in the harvest's
  commit; and `Op::SplitSpan`, #split-at-caret's model half, a comment at its code if it earns no
  head.
- Rewrite `design@rootpoint-model@named-styles` to keep the area's one full style and say that
  per-field overrides are the shape of a span, citing the head of #span-style-overrides;
  `design@rootpoint-model@inline-content-assets`, `design@rootpoint-model@style-kinds` and
  `design@rootpoint-model@stable-ids`, by the milestone's table, and update the texts it lists for
  them in the model's own files; the lines of the protocol's design home and the comment of
  `path@rootpoint-protocol@src/session.rs` are #span-commands'.
- Record the new members of `design@rootpoint-model@cosmetic-fallbacks` and
  `design@rootpoint-model@validated-documents`, which their arguments cover, directly.
- Rewrite `design@rootpoint-render@area-text-layout` for the map and the colour per run.
- Add to `design@rootpoint-model@save-format` the extension `implicit_some` the file declares
  beside `unwrap_variant_newtypes`.
- Amend in place the milestone's "What the work is" for what this slice changed: a text is a
  tree, the file is format 5, and the colour is carried per run.
- The losing alternatives of these threads, judged under
  `skill@knowledge-architect-decision-recording@losing-alternatives`.
- Write T3, guarding the rewritten `design@rootpoint-model@stable-ids`.
