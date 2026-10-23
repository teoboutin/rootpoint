# Slice: Content::Empty removed

The first slice of `milestone@plans@editor-conveniences`. Read the milestone document whole before
this spec.

## Builds

- `Content::Empty` removed from `path@rootpoint-model@src/document.rs`. `Content` is `Text` or
  `Image`, and `Content::default()` is an empty text with `TextStyle::default()`.
- Every new area holds an empty text with `TextStyle::default()`: the area of `Op::AddSlide`, and
  every area `empty_area` makes for a split, in `path@rootpoint-model@src/ops.rs`. The style of a new area becomes the inherited one in
  `spec@editor-conveniences@styles-model`.
- `clear <area>` sets the area's text to the empty string. A text area keeps its style; an image area
  becomes an empty text with `TextStyle::default()`.
- `tree` prints an empty text as `n3 text ""`, which `quote_always` in
  `path@rootpoint-protocol@src/tree.rs` already gives.
- The side panel's content row offers "text" and "image" only, in `App::content` of
  `path@rootpoint-app@src/panels.rs`.
- `FORMAT_VERSION` is 2, and every committed `.ron` file is rewritten in it, per "How the work is
  done" of the milestone document.

## Claims

- **No site names `Content::Empty`.** Test: `grep -rn "Empty" crates --include=*.rs` returns no line
  naming the variant, and the workspace builds. Shown to fail: the build fails while one site of the
  mapping table below is left.
- **A split's new area holds an empty text with the default style.** Test: the test of
  `path@rootpoint-model@tests/ops.rs` that now asserts `Content::Empty` on the new child asserts
  `Content::Text` with an empty `text` and `TextStyle::default()`. Shown to fail: against an
  `empty_area` that gives a text "x", it fails.
- **`clear` keeps a text's style and gives an image area the default style.** Test: a protocol test
  in `path@rootpoint-protocol@tests/commands.rs` clears a text area of size 120 and an image area,
  then reads both contents. Shown to fail: against a `clear` that writes `TextStyle::default()` for
  both, the first assertion fails.
- **A file of version 1 is refused.** Test: a model test loads the fixture's text with `version: 1`
  and gets `LoadError::Version { found: 1, expected: 2 }`. Shown to fail: with `FORMAT_VERSION` left
  at 1, it loads.
- **An empty text draws nothing.** Test: `path@rootpoint-render@tests/empty.rs`, whose slide is
  made by `Op::AddSlide`, still finds every sampled pixel white. Shown to fail: against a renderer
  that draws a placeholder in an empty text area, the centre pixel is not white.
- **A second click on a new area enters editing.** Test: a test of `App::frame` in
  `path@rootpoint-app@tests/editing.rs` adds a slide, clicks its area twice, and finds the window
  editing that area. Shown to fail: against an entry rule, `editing.rs`, that requires a text with
  characters, it is not editing.

## Fixtures

The committed `.ron` files are rewritten at version 2. One of them holds `content: Empty`:
`path@rootpoint-app@tests/data/mixed.ron`, which becomes `content: Text(text: "", style: (…))`
with the default style. The scripts and the tests that expect the word `empty` in a reply of `tree`
expect `text ""`. A search for ` empty` in `path@rootpoint-protocol@tests/scripts/` and in the
`tests/` files of the four crates lists them; at the time of writing it returned
`path@rootpoint-protocol@tests/scripts/groups.txt` and its `.replies`, and test files of the
protocol, the render and the app crates, each to read for whether it means the content kind.

## Audit subjects

- `Content`, `Area` and `TextContent` in `path@rootpoint-model@src/document.rs`; `apply_in`,
  `empty_area` and `asset_in_use` in `path@rootpoint-model@src/ops.rs`.
- `Session::operation` (`Command::Text`, `Command::Clear`) and the asset loading of `render` in
  `path@rootpoint-protocol@src/session.rs`; `node_tree` in `path@rootpoint-protocol@src/tree.rs`.
- `App::content` in `path@rootpoint-app@src/panels.rs`; the loop over areas in
  `path@rootpoint-app@src/assets.rs`; `App::edit_at` and the entry rule of
  `design@rootpoint-app@edit-entry`.
- `render_slide` in `path@rootpoint-render@src/lib.rs`.
- `design@rootpoint-model@inline-content-assets`, `design@rootpoint-app@selection-actions`,
  `design@rootpoint-app@edit-entry`; `tripwire@rootpoint-render@render-within-bounds`, whose
  re-entry is this audit.

## Fails alone on

- A document whose areas held no content before this slice either fails to load at version 2 or
  loads with areas that are not texts.
- A click on a new area's second press does not enter editing.

## Premises that expire

- `TextContent::style` is a `TextStyle` here. `spec@editor-conveniences@styles-model` makes it a
  `StyleChoice`, and the mapping below then reads `StyleChoice::Custom(TextStyle::default())`, or the
  inherited style, for "the default style". Guard: that slice's own mapping table.

## Mapping tables

Every line of Rust that names `Content::Empty`, by `grep -rn Empty crates --include=*.rs` at the time
of writing (17 lines), and what it becomes.

| site | today | becomes |
| --- | --- | --- |
| `path@rootpoint-model@src/document.rs`, `enum Content` | the variant `Empty`, `#[default]` | removed; `impl Default for Content` gives an empty text with `TextStyle::default()` |
| `path@rootpoint-model@src/ops.rs`, `apply_in`, `Op::AddSlide` | the root area holds `Content::Empty` | holds `Content::default()` |
| `path@rootpoint-model@src/ops.rs`, `asset_in_use` | the arm `Content::Empty => false` | removed |
| `path@rootpoint-model@src/ops.rs`, `empty_area` | `Content::Empty` | `Content::default()` |
| `path@rootpoint-protocol@src/session.rs`, the asset loading of `render` | the arm `Content::Empty => {}` | removed |
| `path@rootpoint-protocol@src/session.rs`, `Command::Clear` | `Content::Empty` | a text `""` with the area's style, or `TextStyle::default()` for an image area |
| `path@rootpoint-protocol@src/tree.rs`, `node_tree` | `n<id> empty` | removed; the `Text` arm prints `n<id> text ""` |
| `path@rootpoint-render@src/lib.rs`, `render_slide` | the arm `Content::Empty => {}` | removed; an empty text lays out no glyph |
| `path@rootpoint-app@src/assets.rs`, the loop over areas | `Content::Empty => continue` | removed |
| `path@rootpoint-app@src/panels.rs`, `App::content`, two lines | the "empty" button | removed |
| `path@rootpoint-app@src/panels.rs`, `App::content`, the style match | `Content::Empty => {}` | removed |
| `path@rootpoint-protocol@tests/commands.rs` | an expected `Content::Empty` | the expected empty text |
| `path@rootpoint-model@tests/ops.rs`, two lines | a new area's `Content::Empty`, and the assertion on it | the empty text |
| `path@rootpoint-model@tests/invalid.rs` | an area holding `Content::Empty` | `Content::default()` |
| `path@rootpoint-model@tests/file.rs`, the strategy `content()` | `Just(Content::Empty)` | `Just(Content::default())` |

## Decided design

**`Content::Empty` is removed**, `thread@editor-conveniences@no-empty-kind`. An area holds a text,
possibly with no characters, or an image. The kind added nothing an empty text lacks: the renderer's
`Content::Empty => {}` arm was its only difference (`argument@editor-conveniences@arg-24`). The
nearest rival, keeping it, has no argument left. The consequence the owner accepted with it: a
second click on any area that is not an image enters editing, per
`design@rootpoint-app@edit-entry`, so typing into a new area needs no "text" button first
(`argument@editor-conveniences@arg-15`). T3 watches that the node keys are not lost to an empty
text.

## Implementation sequence

1. Remove the variant and map each site of the table above; the workspace builds, and the existing
   tests pass with the fixtures rewritten. Fails alone on the build.
2. Add the claims' new tests: `clear`, the version refusal, the second click. Fails alone on the
   test that names its claim.

## Acceptance criteria

This slice judges no acceptance criterion.

## Harvest

- `design@rootpoint-model@inline-content-assets`: "empty" leaves the list of what an area holds.
- `design@rootpoint-app@selection-actions`: "the new empty area" becomes the new area, an empty
  text.
- `design@rootpoint-app@edit-entry`: re-read; a second click enters editing on any area that is not
  an image. Its owner's rulings are about text areas, which every such area now is.
- T3, recorded on the owner's word in `register@rootpoint-app@tripwire`, guarding
  `design@rootpoint-app@edit-entry`, with its label in its text, as the premortem of the milestone
  document words it.
- Keeping `Content::Empty` earns no `rejected` entry: its refutation is the one-line fact above,
  which a reader derives again.
