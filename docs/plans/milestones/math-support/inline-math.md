# Slice: math inside a text, and references to equations

The fourth slice of `milestone@plans@math-support`. It lands `thread@math-support@math-text-item`,
`thread@math-support@equation-reference-target`, `thread@math-support@removed-equation-references`,
`thread@math-support@reference-retarget`, `thread@math-support@math-edit-commands` and the text
tree's half of `thread@math-support@math-in-protocol-grammar`, through the model, the file, the
renderer, the protocol and the window's reference picker. The window edits no formula yet. It judges
`acceptance@math-support@ac-math-render-bound`.

## Builds

- **Model**: `TextItem::Math(Formula)` and `TextItem::Ref(NodeId)`, each one item of its text node,
  counting as one offset, by `design@rootpoint-model@text-node-kinds`; `Op::SetMath`, which replaces
  the formula at an offset of a text node and returns the former one;
  `Op::InsertText`'s counterpart for an item, as the audit finds the operations of
  `path@rootpoint-model@src/ops.rs` best extended to insert a math or a reference item; and
  `Op::RetargetReferences { from, to }`, whose inverse moves back exactly the references it moved.
  `apply` refuses `Op::Remove` of an equation area, `Op::SetContent` that turns one into another kind,
  and the removal of a slide holding one, while a text references it, with `OpError::Referenced`
  naming the texts; it refuses a reference, inserted or retargeted, to an area that holds no equation.
  `Op::DuplicateSlide` redirects a reference of the copied slide to an equation of that slide to the
  copy, and keeps every other reference's target.
- **Breach**: a reference to an area that holds no equation, from a file edited by hand or from an
  undo that brings back a text, is the cosmetic breach `Breach::DanglingReference`, drawn `(??)`,
  named by `Document::breaches`, the renderer's report and `render`'s reply, by
  `design@rootpoint-model@cosmetic-fallbacks`; no command creates one, by
  `design@rootpoint-protocol@no-new-breach`.
- **File**: `Math(r#"…"#)` and `Ref(<id>)` items, tagged, by `rejected@rootpoint-model@untagged-text-items`;
  `FORMAT_VERSION` 7, with no migration.
- **Renderer**: each math item laid out by the math crate at the size and in the colour its text node
  resolves to, and placed in the text's Parley layout as an `InlineBox` at its byte index, which the
  `TextMap` of `path@rootpoint-render@src/map.rs` gives it, as it gives each span's edge; a reference
  laid out as the characters of its `(n)`, in its node's style; `AreaText` carries, for each math
  item, its box on the layout and its `FormulaLayout`, so the window can place a caret in it.
- **Protocol**: the tree of `text` and `tree` takes `$…$` for a math item and `{ref n<id>}` for a
  reference, items of the tree beside quoted texts and spans; `insert <n|t> <offset> $…$` and
  `insert <n|t> <offset> {ref n<id>}` insert one; `math <n|t> <offset> $…$` replaces the formula at
  an offset; `retarget n<from> n<to>` replies the number of references it moved; the refusal of a
  removal names each referencing text by its area and the count.
- **Window**: "Insert reference" while a text is edited, a picker listing the presentation's
  numbered equations by their number and the start of their LaTeX, sending `insert … {ref …}`; on an
  equation area's Selection tab, "Referenced n times" and "Retarget all to…", the same picker,
  sending `retarget`; the status line's message when a removal is refused, as
  `thread@math-support@reference-retarget` gives it.

## Steps

1. The items in the model and the file, with the operations' property tests extended. Fails alone
   on: the fixture's round trip at version 7, and the refusals.
2. The renderer's inline boxes and references, with AC5. Fails alone on: the goldens of a text
   holding formulas and references, and AC5.
3. The protocol's items, commands and replies. Fails alone on: the parse of every printed command,
   and the property test of `design@rootpoint-protocol@no-new-breach` with the new commands.
4. The window's picker, the retarget control and the refusal's message. Fails alone on: the
   actions' tests.

## Claims

- **A removal is refused while referenced**: shown to fail by an `apply` that removes the area.
- **Retarget is one undo step that moves back only what it moved**: a test retargets n7 to n9 when n9
  already has references, undoes, and finds n9's own references kept; shown to fail by an inverse
  that retargets n9 to n7.
- **A duplicated slide's references follow the copy**: shown to fail by a duplicate that keeps every
  target.
- **The caret's map counts a math item as one offset**: `TextMap` maps the positions before and after
  a math item to the byte indices around its box; shown to fail by a map that skips the item.
- **AC5**, `acceptance@math-support@ac-math-render-bound`, measured by the ignored tests of
  `tripwire@rootpoint-render@render-within-bounds`, extended with a slide of formulas.

## Audit subjects

- `design@rootpoint-model@text-node-tree`, `design@rootpoint-model@text-tree-normal-form`,
  `design@rootpoint-model@stable-ids`, `design@rootpoint-model@cosmetic-fallbacks`,
  `design@rootpoint-model@validated-documents`, `design@rootpoint-render@area-text-layout`,
  `design@rootpoint-protocol@text-edit-commands`, `design@rootpoint-protocol@protocol-addressing`,
  `design@rootpoint-protocol@no-new-breach`, `rejected@rootpoint-protocol@whole-text-replacement`.
- Parley 0.12's `push_inline_box` and `InlineBox`, in `path@elsewhere@src/builder.rs` and
  `path@elsewhere@src/inline_box.rs`, and how a cursor moves past an inline box.
- `tripwire@rootpoint-protocol@one-line-tree-written-badly`, read against the new items of the tree;
  `tripwire@rootpoint-model@span-ids-in-the-way`, read against `Ref(<id>)` written by hand.
- `issue@rootpoint-app@styled-copy-paste`: a copy of a range holding a formula or a reference
  carries its characters alone today; the audit decides whether `range_text` writes `$…$`.

## Fails alone on

- The fixture's round trip at version 7, the refusals, and AC5.

## Premises that expire

- No formula is edited in the window: #math-editing gives the window its caret in a formula.

## Harvest

- Into `register@rootpoint-model@design`: `design@rootpoint-model@text-node-kinds` rewritten for the
  two kinds built; heads for #equation-reference-target and #removed-equation-references with
  #reference-retarget, by the test of one decision per head;
  `design@rootpoint-model@cosmetic-fallbacks` and `design@rootpoint-model@stable-ids` for their new
  members; `design@rootpoint-model@save-format` for version 7.
- Into `register@rootpoint-protocol@design`: `design@rootpoint-protocol@protocol-transport` and
  `design@rootpoint-protocol@text-edit-commands` for the items, `math` and `retarget`; a head for
  #math-edit-commands if it passes an entry test.
- Into `register@rootpoint-render@design`: `design@rootpoint-render@area-text-layout` for the
  inline boxes.
- T2, into the protocol's tripwires home, guarding `design@rootpoint-protocol@protocol-transport`,
  or folded into `tripwire@rootpoint-protocol@one-line-tree-written-badly`, which guards the same
  head, as an absorbed tripwire is.
- Report on AC5.
