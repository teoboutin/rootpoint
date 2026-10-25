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
  the colour of the values it types with. An empty span that holds the caret, as a style action
  with no selection opens, covers no glyph: its frame is drawn around the caret, as tall as the caret
  and a few points wide, so the author sees the span the next letter goes into.
- The ancestor row in the side panel's Selection tab while editing.
- The span styles in the Styles tab, in their own section, sending `span-style add`, `set`, `rename`
  and `remove`. Each of the five values has a choice that sets none, which sends `<key>=inherit`,
  since every field of a span style is optional, #named-span-styles; the removal of a span style a
  span names opens the dialog of a style's removal, its replacements the other span styles and a
  custom copy, `design@rootpoint-model@style-removal`.
- In the Assets tab, a span style's use is a button that selects the span style in the Styles tab,
  as a style's use selects its style, where it was a label while no tab listed span styles.
- In the browser build, Ctrl+B, Ctrl+I and Ctrl+Space kept from the browser, while a text is
  edited, by the capture-phase listener of `design@rootpoint-app@editor-keys-kept-from-browser`;
  outside editing the editor does not consume them, and the browser keeps them.
- One slide of `path@rootpoint-protocol@sample/sample.ron` with a styled text, which teaches the
  keys.
- A styled span in `path@rootpoint-app@tests/data/mixed.ron`, so that the divider drag
  `tripwire@rootpoint-render@render-within-bounds` times draws a span, as #text-tree gave the
  renderer's mixed slide one.

## Steps

1. The style actions and "split here", with their checks against a second session. Fails alone on:
   the actions' tests.
2. The frames and the caret's style, with AC2, and the sample's slide, whose styled text AC2
   reads. Fails alone on: AC2.
3. The ancestor row and the span styles of the Styles tab. Fails alone on: their actions' tests.
4. The browser's keys. Fails alone on: the browser test of the keys.

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
  `design@rootpoint-app@actions-are-commands`: the actions of editing are in
  `path@rootpoint-app@src/actions.rs`, the caret's state and geometry in
  `path@rootpoint-app@src/editing.rs`.

## Fails alone on

- AC2, and the actions checked against a second session.

## Premises that expire

- The frames' form is the one the owner will see first; T1 watches it, and the owner said they would
  probably iterate on it once they see it live.

## Defaults awaiting the owner

### Q5, ruled: what the Selection tab's controls change while a text is edited

Ruled by the owner: "q5 go with A". Open at the audit, it stopped the slice until then; the thread
#controls-act-at-the-caret records the shape chosen. The milestone says that a control of the side panel, with
no selection, opens an empty span at the caret and, with one, wraps it, #style-action-target; and
that a click on a span of the ancestor row "shows its style in the panel's controls, where it is
changed", #ancestor-frames. It does not say how the one set of controls tells these targets apart,
nor whether the area's own style can be changed while editing, which the controls change outside
editing. Three shapes meet every binding criterion:

- **A, the default.** While editing, the controls are the style actions: they show the values the
  caret types with, or a selected span's, and send `wrap` at the caret or over the selection, or
  `span t<n>` when the selection is one whole span. The style menu lists the span styles. The
  alignment buttons send `text n<area>` with the alignment, since a span has none. A click on a
  span of the ancestor row selects that span whole, so the controls then show and change its
  style; a click on the row's first entry, the text itself, leaves editing, as Esc does, the area
  still selected, where the controls change the area's style as today. One meaning for each
  control at a time, and no new state; the cost is that changing the area's size or style name
  while editing takes one click out of the stay.
- **B.** The ancestor row chooses what the controls change: the caret, by default, for the style
  actions; a span, its style by `span t<n>`; or the text, the area's style by `text n<area>`, in
  the stay. Nothing leaves editing and no selection is made; the cost is a target the author reads
  before each click, held by the window, and a rule for when it returns to the caret, at each move
  of the caret.
- **C.** Two groups of controls while editing: the area's style, as outside editing, and below it
  the style actions at the caret, which a click on a span of the row points at that span by
  selecting it, as in A. No mode, and the area's style is reached in the stay; the cost is the
  panel's controls shown twice, about ten rows of the side panel in place of five.

Each sends its commands inside the stay's group, `design@rootpoint-app@gesture-groups` as this
audit rewrites it. The discriminating fact is none the record holds: each serves
`goal@rootpoint@no-language-no-build`, and the choice weighs a click out of the stay against a mode
or the panel's height.

## Threads

### While a text is edited, the Selection tab's controls are the style actions, and the ancestor row selects a span `##controls-act-at-the-caret`

Put to the owner as Q5 by the session at this slice's design audit, with A as its default, against
B, a target the ancestor row chooses, and C, the area's controls beside the style actions. Closed
by the owner's word: "q5 go with A". Its shape is in "Decided design"; its harvest is the app's
design home, judged by the entry tests with #style-action-target.

## Decided design

The milestone's "Decided design" of #style-action-target, #ancestor-frames, #split-at-caret and
#named-span-styles holds the shape.

### #controls-act-at-the-caret

While a text is edited, the Selection tab's style controls show the values the caret types with,
or, when the selection is one whole span, that span's, and act as the style actions of the
milestone's table: `wrap` at the caret or over the selection, `span t<n>` for a selection that is
one whole span. The style menu lists the span styles. The alignment buttons send `text n<area>` with the alignment, since a span has none.
The ancestor row lists the text, then each span that holds the caret, outer to inner, or, while
one span is selected whole, that span and those that hold it; a click on a span selects it whole,
and a click on the text leaves editing, as Esc does, the area still selected. Every command goes into the stay's group. Nearest rivals: B and C of Q5; the owner chose
A.

## Acceptance criteria

AC2 is judged here, and the landing commits report on it.

## Harvest

- Write the app's heads of #style-action-target, of #controls-act-at-the-caret and of
  #split-at-caret's key and button, if the entry tests give them heads.
- Record the frames as members of `design@rootpoint-app@editor-overlay`, which its argument covers,
  directly, with the reason they take no space; judge the ancestor row by the entry tests, a
  comment at `path@rootpoint-app@src/panels.rs` if it earns no head.
- Rewrite `design@rootpoint-app@styles-tab-edits-styles` for the span styles; update the texts the
  milestone's table lists for these heads, the restatement in `path@rootpoint-app@CLAUDE.md`
  included.
- Rewrite `design@rootpoint-app@gesture-groups`, an action of the side panel on the edited text
  joining the stay, and `design@rootpoint-app@editor-keys-kept-from-browser`, its argument widened
  to every key the editor consumes and its members Ctrl+B, Ctrl+I, their Meta forms and
  Ctrl+Space while editing, as the slice's design audit closed them, each through
  `skill@knowledge-architect-decision-recording@reversal-check`, their rivals judged under
  `skill@knowledge-architect-decision-recording@losing-alternatives`: ending the stay first, an
  undo step of its own, and letting the browser act too. The doc comment of
  `rootpoint_app::keep_editor_keys` then points at the head for its argument. Update the texts the
  milestone's table lists for both.
- Rewrite `design@rootpoint-app@assets-tab`, whose span style's use is a button to the span style
  now, not a label.
- Update `path@rootpoint-app@README.md` for the keys, the frames and the two positions of an edge
  drawn apart, the Selection tab while editing, the span styles of the Styles tab and an asset's
  use by a span style; and `design@rootpoint-protocol@builtin-sample`, whose list of what the sample
  teaches gains the styled text.
- The losing alternatives of these threads, reserved space for the frames among them, which T1's
  response names, judged under `skill@knowledge-architect-decision-recording@losing-alternatives`.
- Write T1, guarding `design@rootpoint-app@editor-overlay`, with the frames recorded in it, and T4,
  guarding the model's head of #named-span-styles; cite T4 from
  `rejected@rootpoint-model@one-table-of-optional-styles`, which its response names.
- The milestone document's own row: its "Harvest" section.
