# Slice: the style actions, "split here", the frames and the span styles in the window

The fourth and last slice of `milestone@plans@rich-text`. It lands
`thread@rich-text@style-action-target`, `thread@rich-text@ancestor-frames`, the window's half of
`thread@rich-text@split-at-caret` and of `thread@rich-text@named-span-styles`. It judges
`acceptance@rich-text@ac-edge-positions-differ`.

## Builds

- Style actions, by the milestone's style action table: Ctrl+B and Ctrl+I, D4, and the controls of
  the side panel's Selection tab while editing, acting on the caret's span or the selection.
- "Split here": Ctrl+Space, and a button of the ancestor row, sending `split-span`, or moving the
  caret out at a span's edge.
- The frames of the caret's spans, drawn by the overlay, and the caret drawn at the height and in
  the colour of the values it types with.
- The ancestor row in the side panel's Selection tab while editing.
- The span styles in the Styles tab, in their own section, sending `span-style add`, `set`, `rename`
  and `remove`.
- In the browser build, Ctrl+B, Ctrl+I and Ctrl+Space kept from the browser by the capture-phase
  listener of `design@rootpoint-app@editor-keys-kept-from-browser`.
- One slide of `path@rootpoint-protocol@sample/sample.ron` with a styled text, which teaches the
  keys.
- A styled span in `path@rootpoint-app@tests/data/mixed.ron`, so that the divider drag
  `tripwire@rootpoint-render@render-within-bounds` times draws a span, as #text-tree gave the
  renderer's mixed slide one.

## Steps

1. The style actions and "split here", with their checks against a second session. Fails alone on:
   the actions' tests.
2. The frames and the caret's style, with AC2. Fails alone on: AC2.
3. The ancestor row and the span styles of the Styles tab. Fails alone on: their actions' tests.
4. The browser's keys, and the sample's slide. Fails alone on: the browser test of the keys.

## Claims

- **AC2**: the test of `acceptance@rich-text@ac-edge-positions-differ`, on the overlay's shapes, as
  `path@rootpoint-app@tests/gestures.rs` reads them with no window; shown to fail by an overlay
  that draws no frame.
- **Each action is its commands**: `path@rootpoint-app@tests/actions.rs` for each style action,
  "split here" and each span style action, against a second session.
- **No frame reaches the slide's image**: `path@rootpoint-app@tests/presenter.rs` with the caret in
  a span.
- **The browser keeps the new keys**: a test of `path@rootpoint-app@tests/web.rs`, as the F5 test
  does.

## Audit subjects

- `path@rootpoint-app@src/asset_tab.rs`, whose comment shows a span style's use as a label until
  this slice lists span styles in the Styles tab.
- `path@rootpoint-app@src/view.rs`, `path@rootpoint-app@src/panels.rs`,
  `path@rootpoint-app@src/actions.rs`, `path@rootpoint-app@src/web.rs`,
  `path@rootpoint-protocol@sample/sample.ron`.
- Two heads the milestone's table lists as to be changed, each a load-bearing gap taken to the
  design skill at this audit: `design@rootpoint-app@gesture-groups`, which ends a stay at any
  action, against the panel's controls and the ancestor row's button acting during one; and
  `design@rootpoint-app@editor-keys-kept-from-browser`, whose argument covers keys whose default
  leaves the page, which Ctrl+B, Ctrl+I and Ctrl+Space do not. Also whether the Selection tab's
  controls, which change a text's own values, `design@rootpoint-app@styles-tab-edits-styles`, act
  on a span while editing.
- `design@rootpoint-app@editor-overlay`, `design@rootpoint-app@styles-tab-edits-styles`,
  `design@rootpoint-app@editor-keys-kept-from-browser`, `design@rootpoint-app@selection-actions`,
  `design@rootpoint-app@sidebar-width`, `design@rootpoint-protocol@builtin-sample`,
  `tripwire@rootpoint-app@keys-confuse-with-the-outline`,
  `issue@rootpoint-app@editing-actions-outside-actions-rs`.

## Fails alone on

- AC2, and the actions checked against a second session.

## Premises that expire

- The frames' form is the one the owner will see first; T1 watches it, and the owner said they would
  probably iterate on it once they see it live.

## Decided design

The milestone's "Decided design" of #style-action-target, #ancestor-frames, #split-at-caret and
#named-span-styles holds the shape.

## Acceptance criteria

AC2 is judged here, and the landing commits report on it.

## Harvest

- Write the app's heads of #style-action-target and of #split-at-caret's key and button, if the
  entry tests give them heads.
- Record the frames as members of `design@rootpoint-app@editor-overlay`, which its argument covers,
  directly, with the reason they take no space; judge the ancestor row by the entry tests, a
  comment at `path@rootpoint-app@src/panels.rs` if it earns no head.
- Rewrite `design@rootpoint-app@styles-tab-edits-styles` for the span styles; update the texts the
  milestone's table lists for these heads, the restatement in `path@rootpoint-app@CLAUDE.md`
  included.
- Update `path@rootpoint-app@README.md` for the keys, and `design@rootpoint-protocol@builtin-sample`,
  whose list of what the sample teaches gains the styled text.
- The losing alternatives of these threads, reserved space for the frames among them, which T1's
  response names, judged under `skill@knowledge-architect-decision-recording@losing-alternatives`.
- Write T1, guarding `design@rootpoint-app@editor-overlay`, with the frames recorded in it, and T4,
  guarding the model's head of #named-span-styles; cite T4 from
  `rejected@rootpoint-model@one-table-of-optional-styles`, which its response names.
- The milestone document's own row: its "Harvest" section.
