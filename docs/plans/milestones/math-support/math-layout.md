# Slice: the formula, its LaTeX form, and its layout from a MATH table

The first slice of `milestone@plans@math-support`. It lands the math crate, with no content kind
and no item of a text using it yet: a formula's tree, its LaTeX subset read and written, the table
of commands, and the layout of a formula from Libertinus Math's font file into boxes, drawn by the
renderer. It lands the shape of `thread@math-support@math-storage-form`,
`thread@math-support@math-layout-engine`, `thread@math-support@math-font`,
`thread@math-support@latex-command-set`, the renderer's half of
`thread@math-support@empty-cell-placeholder`, the layout's half of
`thread@math-support@equation-rows` and `thread@math-support@equation-fit`, and the record of
`thread@math-support@math-glyph-fallback`. It judges `acceptance@math-support@ac-layout-looks-right`,
`acceptance@math-support@ac-fit-is-scaling` and `acceptance@math-support@ac-script-glyphs`.

## Builds

- The crate `path@rootpoint@crates/rootpoint-math/`, a member of the workspace, built for the
  wasm target by the gate `wasm`, whose list `WASM_CRATES` in `path@xtask@src/main.rs` gains it; its
  manifest declares `license = "MIT OR Apache-2.0"`, by `design@rootpoint@permissive-licence`,
  `publish = false`, as every crate of the workspace declares it, and
  its library root reads `ROOTPOINT_CHECKOUT`, by `path@xtask@CLAUDE.md`. It depends on no other
  crate of the workspace, and on ttf-parser 0.25.1 with its default features, among which
  `opentype-layout` exports the `math` module. It holds, by the milestone's "New names, in one
  place":
  - `Formula`, the tree: rows of atoms, an atom a symbol with its class, a fraction, a radical with
    an optional index, a script (nucleus, optional subscript, optional superscript), a delimited
    group (`\left…\right`), an accent, a big operator with its limits, a font change (`\mathbf`,
    `\mathrm`, `\mathcal`, `\mathbb`, `\mathit`), a `\text` run, a matrix of one of the environments
    matrix, pmatrix, bmatrix and cases, a space, a group, and an unknown command, which keeps its
    name and takes no argument;
  - `Formula::parse` and `Formula::to_latex`, the closed LaTeX subset of the milestone's mapping table
    "A LaTeX construct, to the atom it reads as", and back, with the errors of #math-storage-form's
    decided design. `to_latex` of `parse` of a text written by `to_latex` gives the same text;
  - `COMMANDS`, the table of commands, in `path@rootpoint-math@src/commands.rs`,
    generated, each command name with its character and its class: the names of W3C `unicode.xml`
    and LaTeX's aliases, by the milestone's decided design of #latex-command-set and its D6; the class from the character's `type`, by the milestone's mapping table "A
    `unicode.xml` type, to the class of an atom"; and only the characters Libertinus Math's cmap
    has. The W3C notice sits beside it, in `path@rootpoint-math@W3C-NOTICE.txt`;
  - the generator, the command `cargo x math-commands <unicode.xml> <font file>` of
    `path@xtask@src/main.rs`, which writes the two files; it is run again by hand when the font or
    the source changes, and xtask takes quick-xml 0.41 as a dependency for it, MIT, already in
    `path@rootpoint@Cargo.lock` through wayland-scanner, so the workspace gains no package;
  - `MathFont`, built from a font file's bytes, all of which it reads with ttf-parser: the cmap, the
    glyphs' advances and bounds, the MATH table, and the `ssty` lookups of GSUB;
  - `lay_out`, which takes a `Formula`, a `MathFont`, a style, display or text, and a size, and
    gives a `FormulaLayout`: a box per atom and per cell, each with its `CellPath`, its origin, width,
    ascent and descent, and the glyphs to draw with their positions, every value in the size's units.
    It follows the milestone's decided design of #math-layout-engine: styles, `ssty` forms chosen
    before measuring, the spacing between classes, the code points of the font changes, rows, `\text`
    runs, and device tables ignored. An empty cell gets a placeholder box half an em wide, by
    `thread@math-support@empty-cell-placeholder`, a default the slice's audit may revise.
- Libertinus Math 7.051, `LibertinusMath-Regular.otf` of the release archive `Libertinus-7.051.zip`
  of `github.com/alerque/libertinus`, `path@elsewhere@static/OTF/LibertinusMath-Regular.otf` in it,
  embedded in the render crate beside its `OFL.txt`, under
  `path@rootpoint-render@fonts/libertinus/`, and registered in `FontSet::new` with the built-in
  sans; the unit test `a_new_set_holds_the_built_in_family_alone` of
  `path@rootpoint-render@src/text.rs` changes in this slice to the built-in families, text and math.
- In the render crate, `draw_formula`, crate-private, in `path@rootpoint-render@src/formula.rs`, which draws a `FormulaLayout` with vello_cpu
  through the slide's transform, glyphs unhinted, in a colour; a glyph the font lacks, glyph 0, is
  reported in `Report::missing_glyphs` with the area that holds the formula, and an unknown command
  in a new field `Report::unknown_commands`. No content shows a formula before #equation-kind, which
  calls `draw_formula` from `render_slide`, and lists the field in the `render` reply of
  `path@rootpoint-protocol@src/session.rs` and in `App::status` of `path@rootpoint-app@src/lib.rs`.
- A test that pins the missing set both ways, so any change of the font or of its version fails it,
  as the owner approved in round 8: of the math commands of KaTeX v0.19.1 that map to one character,
  a fixture of the test, `path@rootpoint-math@tests/data/katex-symbols.txt`,
  generated from KaTeX's `path@elsewhere@src/symbols.ts` as section 4 of the survey counts them, with
  KaTeX's MIT notice beside it, the commands whose character Libertinus Math's cmap lacks are exactly
  the 44 of that section, and no entry of `COMMANDS` holds the character of one of them. The
  fixture holds 457 commands, counted at the audit by the rule of section 4, which counted 458.
- An ignored test of `path@rootpoint-render@src/formula.rs`, a unit test since `draw_formula` is
  crate-private, that draws the reference formulas into PNG files in the directory
  `ROOTPOINT_MATH_IMAGES` names, for the owner's comparison of AC1.

## Steps

1. The crate: `Formula`, `parse` and `to_latex`, the generator and `COMMANDS`, and the crate in the
   gate `wasm`; Libertinus Math in the render crate, with its licence. Fails alone on: the round trip
   of the property test of the subset.
2. `MathFont`: constants, italic corrections, accent attachments, variants and assemblies read from
   Libertinus Math. Fails alone on: the values the test reads against those the survey measured.
3. `lay_out` and `draw_formula`, with AC2 and AC4 and the pinned missing set. Measured: the size of
   the browser build's `rootpoint.wasm`, which `cargo x web` writes, before the slice and after this
   step, once the renderer links the math crate; and the 4K render of
   `tripwire@rootpoint-render@render-within-bounds`, whose re-entry the slice meets. Fails alone on:
   AC2, AC4.
4. The image test of AC1, and the owner's comparison. Fails alone on: AC1.

## Claims

- **The subset round-trips**: a property test generates formulas, writes them with `to_latex`,
  reads them back with `parse`, and compares; shown to fail by a writer that drops a brace around a
  script of two atoms.
- **An unknown command survives**: `\foo{x}` reads as an unknown atom followed by a group, and writes
  back as `\foo{x}`; shown to fail by a parser that refuses it.
- **A double script does not parse**: `x^a^b` is an error; shown to fail by a parser that keeps the
  second superscript.
- **The MATH table reads as the survey measured**: the script scale-downs read 80 and 60 per cent,
  and the vertical variants of `(` 13, its assembly 3 parts, as section 2 of the survey measured them
  for Libertinus Math; shown to fail by reading the table at a wrong offset.
- **AC2**, `acceptance@math-support@ac-fit-is-scaling`; shown to fail by a layout that adds a
  constant spacing in slide units.
- **AC4**, `acceptance@math-support@ac-script-glyphs`; shown to fail by laying scripts out with the
  base glyph.
- **A command means what it means in LaTeX**: a test reads from `COMMANDS` the characters of
  `\alpha`, `\epsilon` ϵ, `\varepsilon` ε, `\phi` ϕ, `\varphi` φ, `\le`, `\neq`, `\to` and
  `\hbar` ℏ; shown to fail by a generator that reads the `<latex>` column first, D6.
- **The missing set is pinned both ways**: shown to fail by adding to the expected set a command whose
  character the font has, as `\alpha`, and by removing one of the 44 from it.

## Fixtures

The reference formulas, in the subset, which AC1 and AC2 lay out; the first two are equations (61)
and (62) of page 60 of the owner's thesis, as the font sample page of the discussion set them:

```text
\frac{\partial f_i}{\partial t} + \mathbf{c}_i \cdot \nabla_{\mathbf{x}} f_i = \Omega(f_i)
\int_{-\infty}^{\infty} P_n(x)\,\bar{\omega}(x)\,\mathrm{d}x = \sum_{i=1}^{N} w_i P_n(x_i)
f(x) &= (x+1)^2 \\ &= x^2 + 2x + 1 \\ &\leq \sqrt{\frac{a^2+b^2}{2}}
R(\theta) = \begin{pmatrix} \cos\theta & -\sin\theta \\ \sin\theta & \cos\theta \end{pmatrix}
|x| = \begin{cases} x & \text{if } x \geq 0 \\ -x & \text{otherwise} \end{cases}
```

## Audit subjects

- `path@rootpoint@docs/grounding/math-typesetting.md`, section 2, and section 4 for the missing set,
  before any code.
- The OpenType MATH specification, MathConstants, MathGlyphInfo and MathVariants, for every
  constant the layout uses; The TeXbook, chapter 18 and appendix G, and the Unicode block
  Mathematical Alphanumeric Symbols, for the milestone's mapping tables written from memory.
- ttf-parser 0.25.1, `path@elsewhere@src/tables/math.rs` and its GSUB, for the `ssty` lookups.
- `design@rootpoint@crate-existence-rule`, `design@rootpoint@gui-only-in-app` and
  `design@rootpoint@wasm-from-the-start`, which the math crate changes; `design@xtask@browser-tests-gated`,
  whose gate `wasm-test` runs the app's tests `actions` and `web` alone, for whether the math crate's
  tests run in a browser too.
- `path@rootpoint-render@src/text.rs`, `BUILTIN_FONTS` and `FontSet::new`, where the math font is
  registered; `tripwire@rootpoint-render@glyph-missing-everywhere`, read against the missing set;
  `tripwire@rootpoint-render@render-within-bounds`, whose re-entry this slice meets, as it changes
  the renderer.

## Fails alone on

- The round trip of the subset, and the MATH values read against the survey's measurements.
- AC2 and AC4; AC1 is the owner's judgement.

## Premises that expire

- The math font is Libertinus Math: the pinned missing set fails when it changes, and T4 of the
  milestone's premortem, written as a tripwire at this slice's harvest, watches whether a talk needs
  one of them.
- ttf-parser parses the font while read-fonts 0.41 cannot parse its MATH table:
  `issue@rootpoint-render@read-fonts-math-table` waits for parley to move.

## Harvest

- Into `register@rootpoint@design`: `design@rootpoint@crate-existence-rule`, its list of crates and
  its sentence on the model's serialization; `design@rootpoint@gui-only-in-app`, its graph of
  dependencies; `design@rootpoint@wasm-from-the-start`, the crates its gate builds.
- A new Component: the manifest declares `rootpoint-math`, with its README, its `CLAUDE.md`, and its
  goals, design, rejected, tripwires and issues homes, under
  `path@rootpoint-math@docs/`. Its goals are drafted under
  `skill@knowledge-architect-goal-setting` and ruled by the owner, goal by goal, before the harvest
  merges. Heads, by `skill@knowledge-architect-decision-recording`: #math-storage-form's tree and
  LaTeX subset; #math-layout-engine, with its rivals in `rejected` entries where they pass the tests;
  #latex-command-set's table and its source, citing `design@rootpoint@permissive-licence` for the GPL
  table it rules out, with unicode-math's table and KaTeX's in `rejected` entries, each carrying its
  own reason: the renaming condition LPPL 1.3c would add to the binary, which that head does not rule
  out, and the measured count of KaTeX's entries; #equation-rows; #empty-cell-placeholder.
- `issue@rootpoint-render@read-fonts-math-table` moved to the math crate's issue home, and its text to
  what the slice built.
- Into `register@rootpoint-render@design`: a head for #math-font, which carries the 44 commands
  Libertinus Math lacks and cites the test that pins them, on the owner's word that the set be
  recorded; a head for #math-glyph-fallback, a head of absence, no fallback math font, citing a
  `rejected` entry for STIX Two Math as a fallback, refuted by its measured weight; and
  `design@rootpoint-render@font-sources`, whose built-in fonts gain the math font.
- `issue@rootpoint@thorough-testing`: its count of the files that hold a property test gains the
  math crate's.
- T4, into the render crate's tripwires home, guarding the head of #math-glyph-fallback, as
  `skill@knowledge-architect-decision-recording@ruled-tripwires` says.
- Report on AC1, AC2 and AC4.
