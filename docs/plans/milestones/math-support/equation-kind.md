# Slice: the equation, a kind of content, numbered, in the file, the protocol and the window

The third slice of `milestone@plans@math-support`. It lands
`thread@math-support@equation-content-kind`, `thread@math-support@equation-rows`,
`thread@math-support@equation-numbering`, `thread@math-support@equation-options` and
`thread@math-support@equation-fit` through the model, the file, the renderer, the protocol and the
window, and the `$…$` literal of `thread@math-support@math-in-protocol-grammar` for the equation's
command. No text item holds math yet, and no reference exists.

## Builds

- **Model**: `Content::Equation(EquationContent)`, with `formula: Formula`, `numbered: bool`,
  `size: EquationSize` (`Fit` or `AtMost(f64)`, slide units) and `colour: Rgba`, by the milestone's
  "New names, in one place". `Op::SetContent` sets it, as it sets a text or an image.
  `Document::equation_numbers` computes the number of each numbered equation: slides in their order,
  then the areas of a slide in the depth-first order of its tree. A new equation is numbered, sized
  `Fit`, in the colour of the Body style of the table, or black.
- **File**: `FORMAT_VERSION` 6, with no migration, per `design@rootpoint@no-compat-in-initial-dev`;
  `save_ron` sets ron's `escape_strings(false)`, so a formula reads
  `Equation(formula: r#"…"#, numbered: true, size: Fit, colour: …)`; the four `.ron` files of the
  tree written again, `find crates -name '*.ron'` listing them at the slice's start.
- **Renderer**: an equation area draws its formula centred in the area less the padding of
  `design@rootpoint-render@text-padding`, at the scale of its size mode, the number `(n)` at the
  area's right edge on the formula's axis, in the built-in sans at the Body style's size; `Fit`
  takes the largest scale at which the formula and the number fit.
- **Protocol**: `equation <area> $<formula>$ [numbered=yes|no] [size=fit|<n>] [colour=<c>]`,
  setting an area's content to an equation or changing one; the lexer of
  `path@rootpoint-protocol@src/command.rs` reads `$…$` as a word of its own, its content raw up to
  the first `$` that `\` does not escape; `tree` prints an equation area with its formula, its
  options and its number. A printed `Command` parses back to itself.
- **Window**: "Equation" under "Content" in the node menu, `design@rootpoint-app@node-menu`, and a
  button of the side panel's Selection tab, sending `equation` with an empty formula; the Selection
  tab of an equation area shows its options, numbered, the size mode and the colour; the outline's
  row of an equation area shows its number and the start of its LaTeX, `design@rootpoint-app@tree-outline`.
  Editing the formula in the window comes with #math-editing; until then it is changed by the
  protocol.

## Steps

1. The model's kind and the file at version 6, with the property tests of the operations extended
   to it. Fails alone on: the fixture's round trip.
2. The renderer's equation, with the two size modes and the number. Fails alone on: the goldens of
   an equation area at two sizes of area.
3. The protocol's `equation`, its literal and `tree`. Fails alone on: the parse of every printed
   command, and the property test of `design@rootpoint-protocol@no-new-breach`.
4. The window's entries and options, each an action checked against a second session. Fails alone
   on: the actions' tests.

## Claims

- **Numbers follow the order**: a test numbers the equations of a document of three slides,
  then moves a slide and swaps two areas, and reads the numbers again; shown to fail by numbering in
  the order of the ids.
- **A divider drag changes no number**: the same test drags a divider between two equations of
  one split; shown to fail by numbering by position on the slide.
- **The literal is raw**: `equation n7 $\frac{a}{b}$` parses with the formula `\frac{a}{b}`, and
  prints back as typed; shown to fail by a lexer that reads `\f` as an escape.
- **Each window action is its commands**: `path@rootpoint-app@tests/actions.rs`.

## Audit subjects

- `design@rootpoint-model@no-empty-kind`, `design@rootpoint-model@inline-content-assets`,
  `design@rootpoint-model@style-kinds`, `design@rootpoint-model@new-area-style`,
  `design@rootpoint-model@save-format`, `design@rootpoint-model@validated-documents`,
  `design@rootpoint-render@text-padding`, `design@rootpoint-protocol@protocol-transport`,
  `design@rootpoint-app@node-menu`, `design@rootpoint-app@tree-outline`,
  `design@rootpoint-app@no-discard-confirmation`.
- `tripwire@rootpoint-render@render-within-bounds`, read against an equation area.
- The `.ron` files of the tree, and the tests that hold the file as text, as
  `path@rootpoint-protocol@tests/breaches.rs` and `path@rootpoint-model@tests/invalid.rs` did at
  the previous change of the format.

## Fails alone on

- The fixture's round trip at version 6, and the numbering test.

## Premises that expire

- No reference exists: the removal of an equation area is not refused until #inline-math lands the
  references and `thread@math-support@removed-equation-references`.

## Harvest

- Into `register@rootpoint-model@design`: heads for #equation-content-kind, #equation-numbering and
  #equation-options with #equation-fit, by the test of one decision per head;
  `design@rootpoint-model@inline-content-assets` and `design@rootpoint-model@style-kinds` rewritten
  for the third kind; `design@rootpoint-model@save-format` for version 6 and `escape_strings`.
- Into `register@rootpoint-protocol@design`: `design@rootpoint-protocol@protocol-transport` for the
  `$…$` literal.
- Into `register@rootpoint-app@design`: `design@rootpoint-app@node-menu`, its new member recorded
  as the owner approved it; `design@rootpoint-app@edit-entry`, whose sentence "every area that holds
  no image holds a text" stops being true.
- T1, into the model's tripwires home, guarding the head of #equation-options.
