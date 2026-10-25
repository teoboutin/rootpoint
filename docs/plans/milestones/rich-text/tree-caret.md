# Slice: the editor's caret over the text tree

The third slice of `milestone@plans@rich-text`. It lands `thread@rich-text@boundary-caret-positions`,
`thread@rich-text@selection-one-level` and the editor's half of
`thread@rich-text@text-edit-commands`. It judges `acceptance@rich-text@ac-caret-walk` and
`acceptance@rich-text@ac-composition-in-span`.

## Builds

- In `path@rootpoint-app@src/editing.rs`: `Editing` holds a caret position and an anchor in place
  of a Parley `Selection`, mapped to Parley's byte index through `AreaText`'s map to place, draw and
  move; the moves of the milestone's first mapping table; a click to the deepest node that owns the
  glyph.
- Typing, deletion, cut and paste, and the composition of the input method, each as the small
  commands of #span-commands, inside the stay's one undo group; a paste inserts plain text, D3.
- The selection bounded to one node, as the milestone's "Decided design" of #selection-one-level
  says.

## Claims

- **AC1**: the property test of `acceptance@rich-text@ac-caret-walk`, in
  `path@rootpoint-app@tests/editing.rs`; shown to fail by a map that skips the position inside a
  span's start.
- **AC3**: the test of `acceptance@rich-text@ac-composition-in-span`.
- **Typing goes into the caret's node**: at each of the two positions at a span's edge, typed text
  lands inside and outside the span.
- **A stay is one undo step**: a stay of typing, deletion and composition is undone by one `undo`,
  the existing test of `design@rootpoint-app@gesture-groups` extended.
- **Each action is its commands**: `path@rootpoint-app@tests/actions.rs` for typing and deletion.

## Audit subjects

- `path@rootpoint-app@src/editing.rs`, `path@rootpoint-app@src/view.rs`,
  `path@rootpoint-app@src/actions.rs`, `path@rootpoint-app@tests/editing.rs`,
  `path@rootpoint-app@tests/edit_ops.rs`.
- `design@rootpoint-app@edit-in-place`, `design@rootpoint-app@caret-editor-state`,
  `design@rootpoint-app@edit-entry`, `design@rootpoint-app@empty-preedit`,
  `design@rootpoint-app@gesture-groups`, `rejected@rootpoint-app@plain-editor`,
  `issue@rootpoint-app@abandoned-composition-undo-step`,
  `issue@rootpoint-app@editing-actions-outside-actions-rs`,
  `tripwire@rootpoint-app@keys-confuse-with-the-outline`,
  `tripwire@rootpoint-app@empty-text-takes-the-keys`.
- The rule for a byte index at which several edges meet, which the milestone's mapping table leaves
  to this audit.

## Fails alone on

- AC1 and AC3.

## Premises that expire

- No style action exists yet, so the only spans the editor meets come from a file or a command;
  #span-styling ends that.

## Decided design

The milestone's "Decided design" of #boundary-caret-positions, #selection-one-level and
#text-edit-commands holds the shape.

## Acceptance criteria

AC1 and AC3 are judged here, and the landing commits report on them.

## Harvest

- Write the app's heads of #boundary-caret-positions and #selection-one-level; rewrite
  `design@rootpoint-app@edit-in-place` and update the texts the milestone's table lists for it; add
  the caret's node to `design@rootpoint-app@caret-editor-state` as a member the owner's ruling
  admits.
- Open a `todo` issue for styled copy and paste, D3.
