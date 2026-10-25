# Slice: editing a formula in the window, and LyX's math keys

The fifth and last slice of `milestone@plans@math-support`. It lands
`thread@math-support@math-editing-model`, `thread@math-support@math-symbol-palette`,
`thread@math-support@math-keys`, `thread@math-support@keymap-table`, the command typing of
`thread@math-support@latex-command-set`, and the overlay's half of
`thread@math-support@empty-cell-placeholder`. It judges
`acceptance@math-support@ac-caret-on-formula` and `acceptance@math-support@ac-chords-in-browser`.

## Builds

- **The caret in a formula**: a caret position of a text gains, inside a math item, a path of cells
  and an offset in a row, held by the editor alone, by `design@rootpoint-app@caret-editor-state`;
  for an equation area, the same caret over its formula. The moves are LyX's, by the milestone's
  mapping table "A key in a formula, to what it does": Right and Left enter an atom at its first or
  last cell and leave it at its end, Up and Down move between a fraction's numerator and
  denominator and between a script and its nucleus, Tab moves to the next cell; leaving a math item
  at its end puts the caret after it in its text node, one key press, as at a span's edge.
- **Typing in a formula**: each change sends `math` for an inline item or `equation` for an equation
  area, the whole formula replaced, inside the stay's undo group, by `design@rootpoint-app@gesture-groups`.
  A letter or a digit inserts a symbol; `^` and `_` make a script of the previous atom; `{` opens a
  group; `\` opens a command name, extended by letters, with completion from the table of commands,
  closed by a space, which it swallows, or by any other character, which then acts; Space leaves the
  current cell; `&` sets an alignment point or moves to a matrix's next cell; Ctrl+Return starts a
  new row; all as LyX does. Backspace and Delete on an atom with cells select it by a first press and
  delete it by a second, as `design@rootpoint-app@delete-in-caret-node` does for a span; at the first
  or last position of a formula they do nothing; at a math item's edge in its text, they select the
  item, then delete it. A math item left empty when the caret leaves it is removed, as an empty span
  is unwrapped; an equation area keeps its formula, empty or not. The caret over an equation area's
  formula stands on the layout `area_formula` gives, as #equation-kind built it.
- **The overlay**: the caret in a formula, drawn at the height of its cell; the outline of every
  empty cell, on the placeholder box the renderer reserves; the frames of the atoms that hold the
  caret, as the frames of spans are drawn, taking no space, by `design@rootpoint-app@editor-overlay`.
- **The palette**: while a formula is edited, the Selection tab shows the symbols and structures of
  the table of commands that Libertinus Math has, in the categories relations, arrows, operators,
  Greek, delimiters and structures, with the rows and columns of a matrix; a click inserts at the
  caret, by commands. While a text is edited, the Selection tab has "Insert formula", the action of
  Ctrl+M; every action of the keymap table thus has a control, D3.
- **The keymap table**: a built-in table from key sequences to actions, in the app, holding Ctrl+M,
  Ctrl+Shift+M, Ctrl+Alt+N and LyX's Alt+M table as the milestone's mapping table "A LyX math key,
  to the action it runs" gives it, copied from LyX with its oddities; Alt+M arms the prefix, the
  status line shows it, the next key completes the sequence or cancels it; the keys of a sequence
  type no character, though egui-winit 0.37.0 sends a text event for a key held with Alt, which the
  table swallows; a binding with a modifier is matched exactly, since egui's `consume_key` ignores
  extra Shift and Alt; Alt+M C, W and Space are bound by D2's default; the existing shortcuts stay
  where they are, per `issue@rootpoint-app@configurable-keys`.
- **The browser**: the capture-phase listener of `design@rootpoint-app@editor-keys-kept-from-browser`
  prevents the default of Ctrl+M and of the Alt+M sequences while a text or a formula is edited, and
  of Ctrl+Shift+M and Ctrl+Alt+N while an area is selected and not edited, its flag widened from
  "a text is edited" to the states these keys act in; a binding whose key the browser keeps despite
  the listener is disabled in the browser build, on the owner's word of round 2, its action reached
  by its control.

## Steps

1. The caret in a formula, its moves and the overlay's marks, with AC3. Fails alone on: AC3.
2. Typing, commands and deletion, each change a command. Fails alone on: the actions' tests against a
   second session.
3. The palette. Fails alone on: its actions' tests.
4. The keymap table and LyX's keys, natively and in the browser, with AC6. Fails alone on: AC6, and
   a test per binding of the table.

## Claims

- **AC3**, `acceptance@math-support@ac-caret-on-formula`; shown to fail by a caret placed on the
  formula's box rather than its cell's.
- **AC6**, `acceptance@math-support@ac-chords-in-browser`; shown to fail by a listener that does not
  prevent Ctrl+M's default.
- **Each key of the table runs its action**: a test per binding, through `App::frame` with a
  synthetic key; shown to fail by a table with one binding swapped.
- **A typed command becomes its atom**: typing `\frac` then Space gives a fraction with the caret in
  its numerator, and `\leq` then a letter gives ≤ followed by that letter; shown to fail by a macro
  mode that closes on Space alone.

## Audit subjects

- `path@rootpoint-app@src/editing.rs`, `path@rootpoint-app@src/lib.rs` and its key handling,
  `path@rootpoint-app@src/web.rs` and `keep_editor_keys`, `path@rootpoint-app@src/panels.rs` and the
  Selection tab.
- `design@rootpoint-app@edit-in-place`, `design@rootpoint-app@boundary-caret-positions`,
  `design@rootpoint-app@selection-one-level`, `design@rootpoint-app@empty-span-unwrapped`,
  `design@rootpoint-app@split-at-caret`, `design@rootpoint-app@style-action-target`,
  `design@rootpoint-app@selection-keys`, `design@rootpoint-app@edit-entry`,
  `design@rootpoint-app@controls-act-at-the-caret`, `design@rootpoint-app@delete-in-caret-node`,
  `design@rootpoint-model@no-empty-kind`.
- egui-winit 0.37.0's text events with Alt, `path@elsewhere@egui-winit-0.37.0/src/lib.rs`, and egui's
  `consume_key`, for the keymap's matching.
- `tripwire@rootpoint-app@keys-confuse-with-the-outline`, `tripwire@rootpoint-app@frames-mislead`,
  `tripwire@rootpoint-app@empty-text-takes-the-keys`,
  `issue@rootpoint-app@abandoned-composition-undo-step`.
- LyX's `path@elsewhere@lib/bind/math.bind`, `Cursor::mathForward` and `InsetMathNest::interpretChar`,
  as `path@rootpoint@docs/grounding/math-typesetting.md` reads them.

## Fails alone on

- AC3 and AC6.

## Premises that expire

- The keys are fixed: `issue@rootpoint-app@configurable-keys` makes them configurable.

## Harvest

- Into `register@rootpoint-app@design`: heads for #math-editing-model, #math-symbol-palette,
  #math-keys with #keymap-table, by the test of one decision per head;
  `design@rootpoint-app@edit-in-place`, `design@rootpoint-app@boundary-caret-positions`,
  `design@rootpoint-app@editor-overlay` and `design@rootpoint-app@editor-keys-kept-from-browser` for
  their new members; `design@rootpoint-model@no-empty-kind`, whose sentence that the editor has one
  kind to enter for typing stops being true.
- T3, into the app's tripwires home, guarding the head of #math-keys.
- Report on AC3 and AC6.
