# Slice: the style actions, "split here", the frames and the span styles in the window

The fourth and last slice of `milestone@plans@rich-text`. It lands
`thread@rich-text@style-action-target`, `thread@rich-text@ancestor-frames`, the window's half of
`thread@rich-text@split-at-caret` and of `thread@rich-text@named-span-styles`. It judges
`acceptance@rich-text@ac-edge-positions-differ`.

## Builds

- Style actions, by the milestone's second mapping table: Ctrl+B and Ctrl+I, D4, and the controls
  of the side panel's Selection tab while editing, acting on the caret's span or the selection.
- "Split here": Ctrl+Space, and a button of the ancestor row, sending `split-span`.
- The frames of the caret's spans, drawn by the overlay, and the caret drawn at the height and in
  the colour of the values it types with.
- The ancestor row in the side panel's Selection tab while editing.
- The span styles in the Styles tab, in their own section, sending `span-style add`, `set`, `rename`
  and `remove`.
- In the browser build, Ctrl+B, Ctrl+I and Ctrl+Space kept from the browser by the capture-phase
  listener of `design@rootpoint-app@editor-keys-kept-from-browser`.
- One slide of `path@rootpoint-protocol@sample/sample.ron` with a styled text, which teaches the
  keys.

## Claims

- **AC2**: the test of `acceptance@rich-text@ac-edge-positions-differ`, on the overlay's shapes;
  shown to fail by an overlay that draws no frame.
- **Each action is its commands**: `path@rootpoint-app@tests/actions.rs` for each style action,
  "split here" and each span style action, against a second session.
- **No frame reaches the slide's image**: `path@rootpoint-app@tests/presenter.rs` with the caret in
  a span.
- **The browser keeps the new keys**: a test of `path@rootpoint-app@tests/web.rs`, as the F5 test
  does.

## Audit subjects

- `path@rootpoint-app@src/view.rs`, `path@rootpoint-app@src/panels.rs`,
  `path@rootpoint-app@src/actions.rs`, `path@rootpoint-app@src/web.rs`,
  `path@rootpoint-protocol@sample/sample.ron`.
- `design@rootpoint-app@editor-overlay`, `design@rootpoint-app@styles-tab-edits-styles`,
  `design@rootpoint-app@editor-keys-kept-from-browser`, `design@rootpoint-app@selection-actions`,
  `design@rootpoint-app@sidebar-width`, `design@rootpoint-protocol@builtin-sample`,
  `tripwire@rootpoint-app@keys-confuse-with-the-outline`,
  `issue@rootpoint-app@editing-actions-outside-actions-rs`.

## Fails alone on

- AC2, and the actions checked against a second session.

## Premises that expire

- The frames' form is the one the owner will see first; T1 watches it, and the owner said they will
  iterate on it.

## Decided design

The milestone's "Decided design" of #style-action-target, #ancestor-frames, #split-at-caret and
#named-span-styles holds the shape.

## Acceptance criteria

AC2 is judged here, and the landing commits report on it.

## Harvest

- Write the app's heads of #style-action-target and #split-at-caret; rewrite
  `design@rootpoint-app@editor-overlay` for the frames and `design@rootpoint-app@styles-tab-edits-styles`
  for the span styles; update the texts the milestone's table lists for them, the restatement in
  `path@rootpoint-app@CLAUDE.md` included.
- Update `path@rootpoint-app@README.md` for the keys.
- Write T1, guarding the head of #ancestor-frames, and T4, guarding the head of #named-span-styles.
- The milestone document's own row: its "Harvest" section.
