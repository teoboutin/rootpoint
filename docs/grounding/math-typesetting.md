# Math in LyX, Rust math typesetters, math fonts and command tables

## When, what, and from which sources

Run on 2026-10-25, as grounding for the design discussion on math support: inline math in a text,
and a numbered equation as a kind of content that text anywhere can reference. Three agents read
the sources; the session checked their load-bearing claims against their clones and took the
measurements of the last section itself. V marks a claim verified by reading or measuring, A one
assumed or from memory.

It asked:

1. Which math keys LyX binds by default, and how LyX models, numbers and references an equation.
2. Which Rust math typesetters exist, and whether one could serve as a library under rootpoint's
   own renderer, with a box per atom and per cell.
3. Whether the font crates rootpoint links parse the OpenType MATH table.
4. Whether OpenType MATH layout is linear in the font size.
5. Which math fonts are candidates, measured from their files.
6. Which tables from LaTeX command names to Unicode could be embedded.
7. How MathLive models a formula for editing, against LyX.

Sources:

- LyX, `git://git.lyx.org/lyx`, `master` at `e7ffd8d31564c08e3ccc426c898db1e9f305ebf4`
  (2026-10-25). `path@elsewhere@lib/bind/math.bind` is identical at tag 2.4.4; the math lines of
  `path@elsewhere@lib/bind/cua.bind` are the same there. The URLs `github.com/lyx/lyx` and
  `git.lyx.org/lyx.git` do not clone.
- The OpenType specification, "MATH — The mathematical typesetting table", learn.microsoft.com,
  updated 2024-05-31.
- Font crates: read-fonts 0.41.0 to 0.46.0, skrifa 0.44.0 and 0.49.0, ttf-parser 0.25.1, harfrust
  0.12.0, parley 0.12.0.
- Typesetters: ReTeX/ReX `7bd4efc` (2020-07-15) and its fork KenyC/ReX (2026-02-24); typst
  v0.15.1; formulary 0.3.0; latex-rust 3.0.1; RaTeX 0.1.15; mathtex 0.2.0; pulldown-latex 0.8.0;
  math-core 0.9.0; katex-rs 0.3.0; katex 0.4.6; waterui-math; iced_math 0.5.0; mathra 1.2.0.
- Fonts: Noto Sans Math 3.000 (notofonts/math release, 2024-06-05); Fira Math 0.3.4; STIX Two Math
  2.13 b171; Libertinus 7.051 (2024-09-26); Latin Modern Math 1.959 (CTAN `lm-math`); New Computer
  Modern 8.1.1 (CTAN, 2026-06-22); Open Sans, googlefonts/opensans `bd7e376` (2023-11-16).
- Command tables: unicode-math v0.8r (2023-08-13); KaTeX v0.19.1; MathJax v4.1.3; LyX
  `path@elsewhere@lib/symbols`; W3C xml-entities `efed20d` (2026-09-28); WHATWG `entities.json`;
  typst `codex` 0.3.0; unicodeit 0.2.1; mdwright-latex 0.1.3.
- MathLive v0.111.1, `8d148b4` (2026-10-09).

## 1. LyX's math keys and its math model

**Which bind files load (V).** The default bind file is cua (`path@elsewhere@src/KeyMap.cpp`,
`defaultBindfile = "cua"`). `cua.bind` includes `menus.bind`, `math.bind`, `latinkeys.bind`,
`cyrkeys.bind` and `greekkeys.bind`. `greekkeys.bind` maps the keysyms of a Greek keyboard to
`self-insert` and binds no math. On macOS the bind file is `mac.bind`, whose comment maps Alt to
Control (A: read in the comment, not tested).

**Entry chords (V, `cua.bind` lines 77 to 79):** Ctrl+M `math-mode` (inline), Ctrl+Shift+M
`math-display`, Ctrl+Alt+N `math-display` then `math-number-toggle`. No default key inserts a
label, a reference or a matrix: they are in the menus.

**The Alt+M table (V, `math.bind`, 122 bindings).** Structures and symbols: F `\frac`, S `\sqrt`,
X subscript, E superscript, U `\sum`, I `\int`, Y `\oint`, P `\partial`, R `\root`, 8 `\infty`,
M `math-mode`, D `math-display`, L limits, N `math-number-toggle`, Shift+N
`math-number-line-toggle`, O then 1 to 9 macro arguments, Space `math-space`, + `\pm`, = `\neq`.
Sub-tables: T then a key changes the hull type (N none, T dump, A align, I simple, D equation,
E eqnarray, M multline); C then a key edits columns, W then a key rows. Accents: `"` `\ddot`,
H `\hat`, `\` `\grave`, `/` `\acute`, `&` `\tilde`, `-` `\bar`, `.` `\dot`, Shift+V `\check`,
Shift+U `\breve`, V `\vec`, `_` `\underbar`, B `\overline`, `'` `\prime`. Delimiters: `(`, `[`,
`{`, `<` (langle rangle), `>` (rangle langle), `|`.

**Greek, Alt+M G then a letter (V, `math.bind` lines 152 to 196).** a α, b β, c χ, d δ, e ε,
f φ, g γ, h η, i ι, j ϕ (`\varphi`), k κ, l λ, m μ, n ν, o ω, p π, q ϑ, r ρ, s σ, t τ, u υ, v θ,
w ω, x ξ, y ψ, z ζ. Shift: D Δ, E ε (`\varepsilon`), F Φ, G Γ, I ι, J ϵ (`\epsilon`), L Λ, O Ω,
P Π, Q ϑ, R ϱ, S Σ, T ς, U Υ, V Θ, W Ω, X Ξ, Y Ψ. O and W both give ω; six Shift keys give
lowercase variants; A, B, C, H, K, M, N and Z have no Shift binding.

**Typing inside math (V, `InsetMathNest::interpretChar` of `path@elsewhere@src/mathed/InsetMathNest.cpp`).**
`\` starts macro mode; letters extend the name; any other character closes it, and a space closes
it and is swallowed. `^` and `_` make a script of the previous atom. `{ } & $ # %` insert their
own inset. Space leaves the current inset, and at the end of the formula the formula. Ctrl+M inside
math inserts `\text`. Ctrl+Shift+M toggles inline and display. Tab moves to the next cell. Math
autocorrection (`path@elsewhere@lib/autocorrect`, 314 rules such as `<` then `=` giving ≤) runs
only when `autocorrection_math` is set, which defaults to false (`LyXRC.h`).

**Model (V).** A formula is an `InsetMathHull`, a subclass of `InsetMathGrid`, whose types include
simple (inline), equation, eqnarray, align, multline and gather; simple is never numbered. Each row
carries a number state and an optional `InsetLabel`; a label is set with `label-insert` (default
name `eq:`), and setting one numbers its row. A reference is an `InsetRef`, shown `(n)` for
`eqref`. Numbers come from the document class's `equation` counter, stepped in document order;
`stdcounters.inc` gives it no `Within`, so it counts across the whole document. The `.lyx` file
stores a formula as LaTeX inside a Formula inset, and a reference as a separate command inset
naming the label.

**Completion and drawing (V).** Completion is offered in macro mode only; accepting it dispatches a
space. LyX draws math with its own painter; a LaTeX preview replaces it only when previews are on,
which they are not by default, and never while the formula is edited.

## 2. Rust math typesetters

**No candidate gives a box per atom and per cell tied back to the formula, except mathtex (V).**

| candidate | licence | MATH table | boxes | wasm32 | packages for wasm32 |
| --- | --- | --- | --- | --- | --- |
| ReX, ReTeX/ReX | README claims MIT and Apache; no LICENSE file | no: STIX2 metrics compiled in | `LayoutNode` tree, no node id | demo linked (A) | n/a, dead since 2020 |
| ReX fork KenyC/ReX (`rex` 0.1.2, git only) | MIT | yes, behind a `MathFont` trait (ttf-parser or pdf-rs backends) | `LayoutNode { node, width, height, depth }`, no node id | pure Rust (A) | 10, or 11 with ttf-parser |
| typst math, `typst-layout` 0.15.1 | Apache-2.0 | yes, through ttf-parser 0.25 and rustybuzz | private module; output is a `Frame` | cfgs exist; browser use A | 240 |
| formulary 0.3.0 | MIT OR Apache-2.0 | yes, own parser on read-fonts 0.43 | flat `Layout { width, ascent, descent, items }` | pure Rust (A) | 16 |
| latex-rust 3.0.1 | MIT OR Apache-2.0 | yes, own parser on skrifa 0.39 | nested `MathBox`, no node or cell id | no evidence (A) | 10 |
| RaTeX 0.1.15 | MIT | no: KaTeX metrics hard-coded | semantic `LayoutBox`, no source span | `ratex-wasm` crate and CI | 36 |
| mathtex 0.2.0 | MIT OR Apache-2.0; generated core under TeX-family licences | through XeTeX logic | `LayoutNode { id, origin, bounds, primary_source, kind }` | README and web demo | 19; XeTeX translated by c2rust, unsafe code, needs a TeX format file |

The packages were counted with `cargo tree --target wasm32-unknown-unknown -e normal` in a project
depending on each candidate. pulldown-latex, math-core and katex-rs parse LaTeX into MathML or
HTML and lay nothing out; katex and mathjax-svg-rs run JavaScript; waterui-math, iced_math and
mathra emit flat item lists. Parley 0.12, swash 0.2.10 and skrifa 0.44 have no math layout.

## 3. The MATH table in the font crates rootpoint links

- **read-fonts 0.41.0 and skrifa 0.44.0 parse no MATH table (V).** read-fonts 0.41 returns its raw
  bytes through `FontRef::table_data(Tag)`. read-fonts 0.45.0 (2026-10-03) and later have a full
  MATH module; parley 0.12.0, through fontique 0.12.0, and harfrust 0.12.0 require read-fonts
  `^0.41.0`, so using 0.45 now links two copies.
- **ttf-parser 0.25.1 parses the whole MATH table (V, `path@elsewhere@src/tables/math.rs`, 938
  lines):** every constant; italic corrections, top accent attachments, extended shapes and kerning;
  vertical and horizontal glyph constructions with their assemblies. `math::Table::parse(&[u8])` is
  public, so it reads the bytes read-fonts returns, with no ttf-parser `Face`. Licence MIT OR
  Apache-2.0; its one dependency, `core_maths`, is optional. It is not in rootpoint-render's tree
  today (`cargo tree -i ttf-parser --target wasm32-unknown-unknown` prints nothing).
- None of these applies the `ssty` substitution, the smaller forms a font gives for scripts. Every
  measured font has `ssty`. Whether harfrust or parley applies it to one run is A.

## 4. Linearity of the MATH data

**Every MATH value is in font design units or a percentage (V, the specification):** the
MathValueRecords and every constant, the two script scale-downs and
`radicalDegreeBottomRaisePercent` being percentages; the kern heights and values; the connector
lengths, overlaps and advances of the glyph constructions. The choice of a stretchy variant and the
assembly of parts compare design-unit measurements with a design-unit target, so they do not
depend on the size, and `ssty` depends on the script level only. So a formula laid out at size s
and scaled by k is the formula laid out at size k·s, with these exceptions:

- **device tables**, corrections at given pixel sizes, are not linear; the specification lets an
  engine ignore them. No measured font has one on a constant; STIX Two Math has them on 1 italic
  correction and 3 accent attachments (V);
- `spaceAfterScript`, if an engine replaces it by a fixed length;
- engine choices outside the font: snapping to pixels, a minimum rule thickness in pixels, hinting,
  optical sizes. No measured font is variable (no `fvar`, V).

MathKernInfo's algorithm is described wrongly by the specification, per Khaled Hosny on
notofonts/math#99 (V, the comment); this bears on correctness, not on linearity.

## 5. Math fonts

Measured with a script that parses the sfnt table directory and the cmap (V). Every font has a
MATH table, version 1.0, and 1000 units per em.

| font | file (bytes) | glyphs | MATH (bytes) | licence |
| --- | --- | --- | --- | --- |
| Noto Sans Math 3.000, unhinted OTF | 479,308 | 5014 | 20,426 | OFL 1.1 |
| Fira Math 0.3.4 | 179,840 | 2094 | 6,912 | OFL 1.1 |
| STIX Two Math 2.13 OTF | 838,652 | 6760 | 27,408 | OFL 1.1 |
| Libertinus Math 7.051 | 588,848 | 4463 | 10,202 | OFL 1.1 |
| Latin Modern Math 1.959 | 733,736 | 4802 | 25,280 | GUST Font License |
| New Computer Modern Math 4.0 | 1,306,268 | 8603 | 41,170 | GUST Font License |
| New Computer Modern Sans Math 4.0 | 1,183,540 | 8937 | 38,970 | GUST Font License |

| font | math Greek | Operators 2200–22FF | Arrows 2190–21FF | Supp. Ops 2A00–2AFF | Math Alnum | scripts (%) | math kerning |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Noto Sans Math | 290/290 | 256/256 | 112/112 | 256/256 | 994/994 | 60 / 50 | none |
| Fira Math | 174/290 | 158/256 | 112/112 | 3/256 | 438/994 | 72 / 58 | none |
| STIX Two Math | 290/290 | 256/256 | 103/112 | 256/256 | 994/994 | 70 / 55 | 233 glyphs |
| Libertinus Math | 278/290 | 220/256 | 112/112 | 23/256 | 982/994 | 80 / 60 | none |
| Latin Modern Math | 290/290 | 237/256 | 79/112 | 26/256 | 945/994 | 70 / 50 | none |
| New Computer Modern Math | 290/290 | 256/256 | 112/112 | 256/256 | 994/994 | 70 / 50 | 668 glyphs |

Every font stretches `( [ { |` and `√` by variants and an assembly; ∑ has two sizes and no
assembly in any of them. Noto Sans Math's open issues include small scripts (#98), heavy `ssty`
forms (#58), no kerning (#99) and a single weight (#67, #105); its U+0305 extender has
`endConnectorLength 0` in the 3.000 release, which an engine must tolerate (#94, fixed in the
source only).

**What the owner's thesis uses (V).** `github.com/teoboutin/thesis` at `53baab6`: `pdffonts` of
page 60 of `main.pdf`, a page with no figure, lists Linux Libertine for the text and CMMI12, CMMI8,
CMR12, CMSY10, CMEX10 and esint10 for the math, which is Computer Modern; `main.lyx` has
`\font_math "auto"`, and the preamble loads `\usepackage{libertine}` alone. The defense slides,
`defense/defense.pdf`, set the text in Open Sans and the math in Computer Modern.

## 6. Tables from LaTeX command names to Unicode

| source | licence | entries | class per entry |
| --- | --- | --- | --- |
| unicode-math `unicode-math-table.tex` | LPPL 1.3c or later | 2448 | yes |
| KaTeX `path@elsewhere@src/symbols.ts` | MIT | 490 math-mode `\name` entries | yes |
| MathJax `BaseMappings.ts`, `AmsMappings.ts` | Apache-2.0 | 256 and 248 | no, from the operator dictionary |
| LyX `lib/symbols` | GPL v2 or later | 1602 lines, 632 with a code point | yes, for symbols |
| W3C `unicode.xml` | W3C Software Notice and License | 2334 `<latex>` names starting with `\`, 2614 `<mathlatex>` | partly: Unicode MathClass and a TeX-like `type` |
| WHATWG `entities.json` | CC BY 4.0, code BSD-3 | 2231 HTML entity names | no |
| typst `codex` | Apache-2.0 | 1223, typst names only | no |
| unicodeit | `MIT AND LPPL-1.3c` | 4257 pairs | no |
| pulldown-latex | MIT | 570 match arms | by constructor |
| MathLive `symbols.ts` | MIT | 501 | per block |

unicode-math's table is the upstream of ReX's and of `unicode.xml`'s `mathlatex
set="unicode-math"` column (V, file headers).

**Commands Libertinus Math has no glyph for (V).** Of the 458 math commands of KaTeX that map to
one character, Libertinus Math lacks 44: `\lessapprox` ⪅, `\gtrapprox` ⪆, `\lnapprox` ⪉,
`\gnapprox` ⪊, `\eqslantless` ⪕, `\eqslantgtr` ⪖, `\lesseqqgtr` ⪋, `\gtreqqless` ⪌,
`\precapprox` ⪷, `\succapprox` ⪸, `\precnapprox` ⪹, `\succnapprox` ⪺, `\subseteqq` ⫅,
`\supseteqq` ⫆, `\subsetneqq` ⫋, `\supsetneqq` ⫌, `\Subset` ⋐, `\Supset` ⋑, `\bumpeq` ≏,
`\Bumpeq` ≎, `\pitchfork` ⋔, `\multimap` ⊸, `\Cap` ⋒, `\Cup` ⋓, `\doublecap` ⋒, `\doublecup` ⋓,
`\barwedge` ⊼, `\veebar` ⊻, `\doublebarwedge` ⩞, `\divideontimes` ⋇, `\leftthreetimes` ⋋,
`\rightthreetimes` ⋌, `\intercal` ⊺, `\diamond` ⋄, `\triangleleft` ◃, `\triangleright` ▹,
`\bigcirc` ◯, `\blacklozenge` ⧫, `\checkmark` ✓, `\maltese` ✠, `\diagup` ╱, `\diagdown` ╲,
`\Finv` Ⅎ, `\Game` ⅁. By the same count Latin Modern Math lacks 29 and Noto Sans Math 22. Over the
2448 commands of unicode-math's table, Libertinus Math lacks 787; STIX Two Math and New Computer
Modern Math lack none.

## 7. MathLive against LyX

MathLive's `Atom` has fixed branches (body, above, below, subscript, superscript) and `[row, col]`
cells, each starting with a sentinel atom. Its caret is one integer offset over a post-order walk of
the tree, where LyX keeps a stack of `(idx, pos)` slices. Scripts hang as branches on the previous
atom, where LyX wraps it in an `InsetMathScript`. Empty slots are `\placeholder{}` atoms stored in
the LaTeX, where LyX draws a box and stores nothing. `\` opens a group that closes on Space, Enter
or Tab and is parsed whole, arguments included, where LyX closes the name on the first non-letter.
About 259 inline shortcuts (`alpha`, `>=`) are undone to the typed keys by Backspace. Up and down
move by rendered x positions.

## Measurements of the session

- **ron 0.12.2 writes raw strings (V).** In a probe crate, `PrettyConfig::escape_strings(false)`
  writes `Math(r#"\frac{a}{b} + \alpha"#)` where the default writes `"\\frac{a}{b} + \\alpha"`, and
  both forms read back equal. The setting applies to every string of the file: a string holding a
  quote or a newline is written as a raw string with a real line break.
- **Open Sans against Noto Sans (V).** The `path@elsewhere@fonts/noto-set/ttf/` build of Open Sans maps 2882 code
  points against Noto Sans's 2840 and covers Latin Extended-B, Vietnamese, Greek, Cyrillic and
  General Punctuation as Noto Sans does, plus Hebrew (45/112) and 13 math operators; its four
  static styles weigh 2,159,352 bytes against 2,092,724. The `path@elsewhere@fonts/ttf/` build maps 1010 code
  points. Neither Open Sans nor Noto Sans has an arrow of U+2190–21FF.

## What transfers, and the property each depends on

- **Writing the layout on the MATH table** transfers from typst, formulary, latex-rust and ReX, which
  all do it. It depends on a MATH parser that builds for wasm32, which ttf-parser 0.25.1 is, and on
  applying `ssty` through shaping (A).
- **Borrowing a box-tree engine** depends on adding node and cell identity to its boxes, which none
  has but mathtex.
- **Fitting an equation to its area by one layout and one multiplication** depends on ignoring device
  tables, snapping nothing, unhinted glyphs and a non-variable font.
- **The command table from `unicode.xml`** depends on its W3C licence notice travelling with the
  generated table.
- **LyX's empty-cell model** keeps placeholders out of the stored LaTeX; MathLive's stores them.
