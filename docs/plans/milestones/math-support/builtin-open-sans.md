# Slice: Open Sans as the built-in text family, falling back to the math font

The second slice of `milestone@plans@math-support`. It lands
`thread@math-support@builtin-text-family`: Open Sans replaces Noto Sans as the one built-in text
family, and a text falls back to Libertinus Math for a character its font lacks. It judges
`acceptance@math-support@ac-arrows-fall-back`.

## Builds

- The four static styles of Open Sans's `noto-set` build, `OpenSans-Regular.ttf`, `-Bold`,
  `-Italic` and `-BoldItalic`, from googlefonts/opensans, `path@elsewhere@fonts/noto-set/ttf/`,
  embedded in the render crate with Open Sans's `OFL.txt`, under
  `path@rootpoint-render@fonts/open-sans/`; the Noto Sans files and their `OFL.txt` removed.
- `BUILTIN_SANS` in `path@rootpoint-render@src/text.rs` names Open Sans; `FontRef::Builtin("sans")`
  keeps its name, so no saved file changes.
- The font stack of `layout_text` ends with Libertinus Math after the built-in sans, so a character
  neither the text's font nor Open Sans has, as an arrow, is drawn from the math font; the report's
  `missing_glyphs` then names only what no built-in font has.
- Every golden image of the render and app crates written again, and every test that names Noto Sans
  rewritten: the comments and the font of `path@rootpoint-render@tests/edges.rs`, the ligature of
  `path@rootpoint-app@tests/editing.rs` (Open Sans's own ligatures to be read at the audit), and the
  font file `path@rootpoint-protocol@tests/pack.rs` packs.
- The README's licence section, which says the built-in fonts keep their own licence beside each
  file, read again against the new layout of `fonts/`.

## Steps

1. The fonts swapped and the goldens written again, with no fallback change. Fails alone on: the
   goldens, and the measured binary sizes.
2. The fallback to the math font, with AC7. Fails alone on: AC7.

## Claims

- **No system font is discovered**: the unit test `a_new_set_holds_the_built_in_family_alone` of
  `path@rootpoint-render@src/text.rs`, as #math-layout left it, pinning a new `FontSet` to the built-in
  families, Open Sans and Libertinus Math.
- **AC7**, `acceptance@math-support@ac-arrows-fall-back`; shown to fail by a stack without the math
  font.
- **Every saved file loads unchanged**: the fixture's round trip, unchanged since the family's name is
  kept.

## Audit subjects

- `design@rootpoint-render@builtin-noto-sans`, which this slice changes, and
  `tripwire@rootpoint-render@glyph-missing-everywhere`, whose concern the coverage measured in
  `path@rootpoint@docs/grounding/math-typesetting.md` answers.
- `path@rootpoint-render@src/text.rs`, `layout_text`, `BUILTIN_SANS`, `BUILTIN_FONTS`.
- Every file that `grep -rn "Noto"` finds in `crates/`.
- `tripwire@rootpoint-render@render-within-bounds`, whose re-entry this slice meets, as it changes
  the renderer, and whose test then draws in Open Sans.

## Fails alone on

- The goldens drawn in Open Sans, and AC7.

## Premises that expire

- Libertinus Math is built in, by #math-layout: the fallback names it.

## Harvest

- Into `register@rootpoint-render@design`: `design@rootpoint-render@builtin-noto-sans` rewritten in
  place for Open Sans, its slug renamed by the rule of `primer@design-heads`, with every reference to
  it rewritten, and the owner's choice attributed to the owner; the fallback to the math font stated
  in `design@rootpoint-render@font-sources` or in the head of the built-in family, by the test of one
  decision per head.
- Report on AC7.
