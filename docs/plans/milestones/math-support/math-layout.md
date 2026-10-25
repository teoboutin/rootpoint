# Slice: the formula, its LaTeX form, and its layout from a MATH table

The first slice of `milestone@plans@math-support`. It lands the math crate, with no content kind
and no item of a text using it yet: a formula's tree, its LaTeX subset read and written, the table
of commands, and the layout of a formula from Libertinus Math's MATH table into boxes, drawn by
the renderer. It lands the shape of `thread@math-support@math-storage-form`,
`thread@math-support@math-layout-engine`, `thread@math-support@math-font`,
`thread@math-support@latex-command-set`, the renderer's half of
`thread@math-support@empty-cell-placeholder`, the layout's half of
`thread@math-support@equation-rows` and `thread@math-support@equation-fit`, and the record of
`thread@math-support@math-glyph-fallback`. It judges `acceptance@math-support@ac-layout-looks-right`,
`acceptance@math-support@ac-fit-is-scaling` and `acceptance@math-support@ac-script-glyphs`.

## Builds

- The crate `planned@rootpoint@crates/rootpoint-math/`, a member of the workspace, built for the
  wasm target by the gate `wasm`, whose list in `path@xtask@src/main.rs` gains it; its manifest
  declares `license = "MIT OR Apache-2.0"`. It depends on no other crate of the workspace, and on
  ttf-parser 0.25.1 for the MATH table. It holds, by the milestone's "New names, in one place":
  - `Formula`, the tree: rows of atoms, an atom a symbol, a fraction, a radical with an optional
    index, a script (nucleus, optional subscript, optional superscript), a delimited group
    (`\left…\right`), an accent, a big operator with its limits, a font change (`\mathbf`,
    `\mathrm`, `\mathcal`, `\mathbb`), a `\text` run, a matrix of one of the environments matrix,
    pmatrix, bmatrix and cases, a space, and an unknown command, which keeps its name;
  - `Formula::parse` and `Formula::to_latex`, the closed LaTeX subset of the milestone's mapping
    table "A LaTeX construct, to the atom it reads as", and back. `to_latex` of `parse` of a text
    written by `to_latex` gives the same text;
  - the table of commands, generated from W3C `unicode.xml`, `<latex>` and `<mathlatex>` columns,
    and limited to the characters Libertinus Math has, with the W3C notice beside the generated
    file; and the generator, as a command of `path@xtask@src/main.rs` that reads a `unicode.xml`
    given as an argument, so it is run again by hand when the font or the source changes;
  - `MathFont`, built from a font file's bytes, its MATH table read with ttf-parser from the bytes
    read-fonts returns, per `issue@rootpoint-render@read-fonts-math-table`; and `lay_out`, which
    gives a `FormulaLayout`: a box per atom and per cell, each with its path in the formula, its
    origin, width, ascent and descent in font units scaled to a size, and the glyphs to draw with
    their positions. An empty cell gets a placeholder box of the font's em width divided by two,
    by `thread@math-support@empty-cell-placeholder`, a default the slice's audit may revise. Device
    tables are ignored. Rows are aligned at `&` and spaced by the font's constants.
- Libertinus Math 7.051, `LibertinusMath-Regular.otf`, embedded in the render crate beside its
  `OFL.txt`, under `planned@rootpoint-render@fonts/libertinus/`, so the two fonts' licence files do
  not collide.
- In the render crate, `draw_formula`, which draws a `FormulaLayout` with vello_cpu through the
  slide's transform, glyphs unhinted, applying the font's `ssty` forms to scripts through harfrust
  or Parley's shaping, whichever the audit finds can shape one run with a feature.
- A test that pins the 44 commands of KaTeX that Libertinus Math lacks, listed in
  `path@rootpoint@docs/grounding/math-typesetting.md`, against the font's character map, so a
  change of the font fails it.
- An image test of the formulas of `acceptance@math-support@ac-layout-looks-right`, written as
  PNG files for the owner to compare.

## Steps

1. The crate, empty of layout: `Formula`, `parse` and `to_latex`, the generated table, and the
   crate in the gate `wasm`. Measured: the size of the wasm binary of the app before and after.
   Fails alone on: the round trip of the subset's fixtures.
2. `MathFont` and the MATH table: constants, italic corrections, accent attachments, variants and
   assemblies read from Libertinus Math. Fails alone on: the values the test reads against those
   the survey's script measured.
3. `lay_out` and `draw_formula`, with AC2 and AC4 and the pinned missing set. Fails alone on: AC2,
   AC4.
4. The image test of AC1, and the owner's comparison. Fails alone on: AC1.

## Claims

- **The subset round-trips**: a property test generates formulas, writes them with `to_latex`,
  reads them back with `parse`, and compares; shown to fail by a writer that drops a brace around a
  script of two atoms.
- **An unknown command survives**: `\foo{x}` reads as an unknown atom and writes back as `\foo{x}`;
  shown to fail by a parser that refuses it.
- **The MATH table reads as the survey measured**: the script scale-downs read 80 and 60 per cent,
  and the test reads the vertical variants of `(` as 13 and its assembly as 3 parts; shown to fail
  by reading the table at a wrong offset.
- **AC2**, `acceptance@math-support@ac-fit-is-scaling`; shown to fail by a layout that adds a
  constant spacing in slide units.
- **AC4**, `acceptance@math-support@ac-script-glyphs`; shown to fail by drawing scripts with the
  base glyph.
- **The missing set is pinned**: the test lists the 44 commands; shown to fail by a font file with
  one more glyph mapped, built for the test by editing the cmap of a copy.

## Fixtures

The formulas of AC1, written in the subset: (61) and (62) of the owner's thesis, page 60, as the
sample pages of the discussion set them; the aligned derivation; the rotation matrix and the
`cases` of the font sample page.

## Audit subjects

- `path@rootpoint@docs/grounding/math-typesetting.md`, sections 3, 4 and 5, before any code.
- The OpenType MATH specification, MathConstants, MathGlyphInfo and MathVariants, for every
  constant the layout uses.
- ttf-parser 0.25.1, `path@elsewhere@src/tables/math.rs`; and whether harfrust 0.12 or Parley 0.12
  can shape one run with the feature `ssty`.
- `design@rootpoint@crate-existence-rule` and `design@rootpoint@gui-only-in-app`, whose lists of
  crates gain `rootpoint-math`; `design@rootpoint@wasm-from-the-start`.
- `path@rootpoint-render@src/text.rs`, `BUILTIN_FONTS` and `FontSet::new`, where the math font is
  registered too.

## Fails alone on

- The round trip of the subset, and the MATH values read against the survey's measurements.
- AC2 and AC4.

## Premises that expire

- The math font is Libertinus Math: the pinned missing set fails when it changes, and
  T4 of the milestone's premortem, written as a tripwire at this slice's harvest, watches whether a
  talk needs one of them.
- ttf-parser parses the MATH table while read-fonts 0.41 cannot:
  `issue@rootpoint-render@read-fonts-math-table` waits for parley to move.

## Harvest

- Into `register@rootpoint@design`: `design@rootpoint@crate-existence-rule` and
  `design@rootpoint@gui-only-in-app` name `rootpoint-math` among the crates the rule gives.
- A new design home, `planned@rootpoint@crates/rootpoint-math/docs/design.md`, with the Component's other required
  documents, its README, its `CLAUDE.md`, and its goals, rejected, tripwires and issues homes, the
  goals written under `skill@knowledge-architect-goal-setting`; and the manifest declares the
  Component. Heads, by `skill@knowledge-architect-decision-recording`: #math-storage-form's tree and
  LaTeX subset; #math-layout-engine, with its rivals in `rejected` entries where they pass the
  tests; #latex-command-set's table and its source.
- Into `register@rootpoint-render@design`: a head for #math-font, which carries the 44 commands
  Libertinus Math lacks and cites the test that pins them, on the owner's word that the set be
  recorded; `design@rootpoint-render@font-sources`, whose built-in fonts gain the math font.
- T4, into the render crate's tripwires home, guarding the head of #math-font, as
  `skill@knowledge-architect-decision-recording@ruled-tripwires` says.
- Report on AC1, AC2 and AC4.
